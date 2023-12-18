use crate::util::color::Color;
use enum_map::{Enum, EnumMap};
use rand;
use std::fmt;
use std::iter::zip;
use std::ops::{Add, Mul, Neg, Sub};

use crate::util::*;

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    X,
    Y,
    Z,
}

impl Basis {
    pub fn to_vec(&self) -> Vec3 {
        match self {
            Basis::X => Vec3::new(1.0, 0.0, 0.0),
            Basis::Y => Vec3::new(0.0, 1.0, 0.0),
            Basis::Z => Vec3::new(0.0, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub enum BasisDiff {
    D0,
    D1,
    D2,
}

impl Add<BasisDiff> for Basis {
    type Output = Self;
    fn add(self, diff: BasisDiff) -> Self {
        match (self, diff) {
            (_, BasisDiff::D0) => self,
            (Basis::X, BasisDiff::D1) => Basis::Y,
            (Basis::Y, BasisDiff::D1) => Basis::Z,
            (Basis::Z, BasisDiff::D1) => Basis::X,
            (Basis::X, BasisDiff::D2) => Basis::Z,
            (Basis::Y, BasisDiff::D2) => Basis::X,
            (Basis::Z, BasisDiff::D2) => Basis::Y,
        }
    }
}

impl Add for BasisDiff {
    type Output = Self;
    fn add(self, diff: BasisDiff) -> Self {
        match (self, diff) {
            (_, BasisDiff::D0) => self,
            (BasisDiff::D0, _) => self,
            (BasisDiff::D1, BasisDiff::D1) => BasisDiff::D2,
            (BasisDiff::D1, BasisDiff::D2) => BasisDiff::D0,
            (BasisDiff::D2, BasisDiff::D1) => BasisDiff::D0,
            (BasisDiff::D2, BasisDiff::D2) => BasisDiff::D1,
        }
    }
}

impl Sub<BasisDiff> for Basis {
    type Output = Self;
    fn sub(self, diff: BasisDiff) -> Self {
        self + (-diff)
    }
}

impl Sub for Basis {
    type Output = BasisDiff;
    fn sub(self, basis: Self) -> BasisDiff {
        match (self, basis) {
            (Basis::X, Basis::X) => BasisDiff::D0,
            (Basis::Y, Basis::Y) => BasisDiff::D0,
            (Basis::Z, Basis::Z) => BasisDiff::D0,
            (Basis::Y, Basis::X) => BasisDiff::D1,
            (Basis::Z, Basis::Y) => BasisDiff::D1,
            (Basis::X, Basis::Z) => BasisDiff::D1,
            (Basis::Z, Basis::X) => BasisDiff::D2,
            (Basis::X, Basis::Y) => BasisDiff::D2,
            (Basis::Y, Basis::Z) => BasisDiff::D2,
        }
    }
}

impl Neg for BasisDiff {
    type Output = BasisDiff;
    fn neg(self) -> Self {
        match self {
            BasisDiff::D0 => BasisDiff::D0,
            BasisDiff::D1 => BasisDiff::D2,
            BasisDiff::D2 => BasisDiff::D1,
        }
    }
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Pos,
    Neg,
}

impl Mul for Sign {
    type Output = Sign;
    fn mul(self, sign: Self) -> Self {
        if self == sign {
            Sign::Pos
        } else {
            Sign::Neg
        }
    }
}

impl Neg for Sign {
    type Output = Sign;
    fn neg(self) -> Self {
        match self {
            Sign::Pos => Sign::Neg,
            Sign::Neg => Sign::Pos,
        }
    }
}

impl Sign {
    pub fn to_f32(&self) -> f32 {
        match self {
            Self::Pos => 1.0,
            Self::Neg => -1.0,
        }
    }
}

/// A set of rays that align with the turns of the puzzle.
/// Changing the order of them will change the log files.
pub trait RaySystem
where
    Self: 'static
        + Sized
        + Eq
        + Copy
        + Clone
        + Enum
        + std::fmt::Debug
        + enum_map::EnumArray<i8>
        + enum_map::EnumArray<Self>
        + enum_map::EnumArray<Color>
        + fmt::Display,
{
    /// Returns a list of rays that make up the vector. Should
    /// return the same order for each axis.
    fn get_axis(&self) -> Vec<Self>;
    /// Turns the ray system one unit clockwise about ray's axis and returns the new ray
    /// that occupies self's direction.
    /// Should return the same value for any ray with the same axis.
    fn turn_one(&self, ray: Self) -> Self;
    /// Turns the ray system about ray's axis and returns the new ray
    /// that occupies self's direction.
    /// Should return the same value for any ray with the same axis.
    fn turn(&self, ray_order: (Self, i8)) -> Self {
        let (ray, order) = ray_order;
        let mut turned = *self;
        for _ in 0..order.rem_euclid(ray.order()) {
            turned = turned.turn_one(ray);
        }
        turned
    }
    /// Gets the order of the turn
    fn order(&self) -> i8;
    /// Returns a list of rays, each one of which is the first ray of its axis.
    const AXIS_HEADS: &'static [Self];
    /// Hamiltonian cycle for symmetry group
    const CYCLE: &'static [(Self, i8)];

    /// Uniform selection
    fn choose<R: rand::Rng>(rng: &mut R) -> Self {
        use rand::seq::SliceRandom;
        *enum_iter::<Self>()
            .collect::<Vec<_>>()
            .choose(rng)
            .expect("ray system should not be empty")
    }

    // Short name for ray
    fn name(&self) -> String;

    fn from_name(name: &str) -> Option<Self> {
        enum_iter::<Self>().find(|ray: &Self| ray.name() == name)
    }
}

/// A single piece of an abstract laminated puzzle.
#[derive(Debug)]
pub struct Piece<Ray>
where
    Ray: RaySystem,
{
    /// For each ray, the layer index on that ray (solved position). It does not change.
    pub layers: EnumMap<Ray, i8>,
    /// For each ray, a tuple of the ray currently occupying
    /// that direction and its layer parameter. It changes when the piece moves.
    pub orientation: EnumMap<Ray, Ray>,
}

impl<Ray: RaySystem> Piece<Ray> {
    pub fn make_solved(axis_layers: Vec<Vec<i8>>) -> Self {
        let mut layers = EnumMap::from_fn(|_ray| 0);
        for (axh, axl) in zip(Ray::AXIS_HEADS, axis_layers) {
            for (ray, layer) in zip(axh.get_axis(), axl) {
                layers[ray] = layer;
            }
        }

        Self::make_solved_from_layers(layers)
    }

    pub fn make_solved_from_layers(layers: EnumMap<Ray, i8>) -> Self {
        Self {
            layers,
            orientation: EnumMap::from_fn(|ray| ray),
        }
    }

    /// WARNING: this method checks if the piece is in the solved orientation!
    /// The puzzle is considered solved if all its pieces are in the same orientation.
    pub fn is_solved(&self) -> bool {
        self.orientation.iter().all(|(pos, &cur)| pos == cur)
    }

    pub fn grip_on_axis(&self, ray: Ray) -> Vec<i8> {
        ray.get_axis()
            .iter()
            .map(|&r| self.layers[self.orientation[r]])
            .collect()
    }

    pub fn grip_on_axis_solved(&self, ray: Ray) -> Vec<i8> {
        ray.get_axis().iter().map(|&r| self.layers[r]).collect()
    }

    pub fn twist(&mut self, (ray, order): (Ray, i8), grip: &[i8]) -> bool {
        if &self.grip_on_axis(ray)[..] == grip {
            let new_orientation = EnumMap::from_fn(|r: Ray| self.orientation[r.turn((ray, order))]);
            self.orientation = new_orientation;
            /*for (_, cur) in self.orientation.iter_mut() {
                *cur = cur.turn(&(ray, order));
            }*/
            return true;
        }
        false
    }

    pub fn oriented_layers(&self) -> EnumMap<Ray, i8> {
        EnumMap::from_fn(|ray| self.layers[self.orientation[ray]])
    }
}

impl<Ray: RaySystem + fmt::Display> fmt::Display for Piece<Ray> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first_iter = true;
        write!(f, "[")?;
        for (ray, layer) in &self.layers {
            if !first_iter {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", ray, layer)?;
            first_iter = false;
        }
        write!(f, "]")?;
        Ok(())
    }
}

/// Abstract laminated puzzle.
/// I assume the pieces are always in order of their layers.
#[derive(Debug)]
pub struct Puzzle<Ray: RaySystem> {
    pub grips: Vec<Vec<i8>>,
    pub pieces: Vec<Piece<Ray>>,
}

impl<Ray: RaySystem> Puzzle<Ray> {
    pub fn piece_count(&self) -> usize {
        self.grips.len().pow(Ray::AXIS_HEADS.len() as u32)
    }

    pub fn make_solved(grips: Vec<Vec<i8>>) -> Self {
        let mut new = Self {
            grips: grips.clone(),
            pieces: Vec::new(),
        };
        new.pieces = (0..new.piece_count())
            .map(|i| new.index_to_solved_piece(i))
            .collect();
        new
    }

    /// Checks whether the puzzle is solved, i.e. whether all pieces
    /// are in the same orientation.
    pub fn is_solved(&self) -> bool {
        self.pieces
            .iter()
            .skip(1)
            .all(|piece| piece.orientation == self.pieces[0].orientation)
    }

    /// Applies the twist to the puzzle.
    pub fn twist(&mut self, (ray, order): (Ray, i8), grip: &[i8]) {
        for i in 0..self.pieces.len() {
            let _piece_twisted = self.pieces[i].twist((ray, order), grip);
        }
    }

    /// Returns a new solved piece whose index is the provided usize.
    pub fn index_to_solved_piece(&self, i: usize) -> Piece<Ray> {
        let grip_count: usize = self.grips.len();
        Piece::make_solved(
            (0..Ray::AXIS_HEADS.len())
                .map(|j| self.grips[i / grip_count.pow(j as u32) % grip_count].clone())
                .collect(),
        )
    }

    /// Gets the index of the current position of the piece.
    pub fn piece_to_index(&self, piece: &Piece<Ray>) -> usize {
        Ray::AXIS_HEADS
            .iter()
            .enumerate()
            .map(|(j, &r)| {
                self.grips
                    .iter()
                    .position(|gr| &piece.grip_on_axis(r)[..] == gr)
                    .expect("grips should all exist because the piece should be valid")
                    * self.grips.len().pow(j as u32)
            })
            .sum()
    }

    /// Gets the index of the solved position of the piece.
    pub fn piece_to_index_solved(&self, piece: &Piece<Ray>) -> usize {
        Ray::AXIS_HEADS
            .iter()
            .enumerate()
            .map(|(j, &r)| {
                self.grips
                    .iter()
                    .position(|gr| &piece.grip_on_axis_solved(r)[..] == gr)
                    .expect("grips should all exist because the piece should be valid")
                    * self.grips.len().pow(j as u32)
            })
            .sum()
    }

    pub fn permutation(&self) -> Vec<usize> {
        let mut permutation = vec![0; self.piece_count()];
        for i in 0..self.pieces.len() {
            let piece_index = self.piece_to_index(&self.pieces[i]);
            permutation[piece_index] = i;
        }
        permutation
    }

    pub fn orientations(&self) -> Vec<EnumMap<Ray, Ray>> {
        self.pieces
            .iter()
            .map(|piece| enum_map_clone(&piece.orientation))
            .collect()
    }

    pub fn set_orientations(&mut self, oris: &[EnumMap<Ray, Ray>]) {
        for (piece, ori) in self.pieces.iter_mut().zip(oris) {
            piece.orientation = enum_map_clone(ori);
        }
    }

    pub fn scramble(&mut self) {
        use rand::seq::SliceRandom;
        use rand::Rng;

        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let ray = Ray::choose(&mut rng);
            let grip = self.grips[..]
                .choose(&mut rng)
                .expect("ray system should not be empty")
                .to_vec();
            self.twist((ray, rng.gen_range(0..ray.order())), &grip[..]);
        }
    }
}

pub mod ray_system_tests {
    use super::*;
    use itertools::Itertools;

    fn axes_all_same_order<Ray: RaySystem + std::fmt::Debug>() {
        for ray in enum_iter::<Ray>() {
            let axis = ray.get_axis().clone();
            for ray2 in &axis {
                let axis2 = ray2.get_axis().clone();
                assert_eq!(
                    axis, axis2,
                    "rays {:?} and {:?} have different axes",
                    ray, ray2
                );
            }
        }
    }

    fn axis_heads_all_heads<Ray: RaySystem + std::fmt::Debug>() {
        for ray in Ray::AXIS_HEADS {
            let axis = ray.get_axis();
            assert_eq!(ray, &axis[0], "ray {:?} is not an axis head", ray);
        }
    }

    fn turns_consistent_axis<Ray: RaySystem + std::fmt::Debug>() {
        for &ray in Ray::AXIS_HEADS {
            for ray2 in ray.get_axis() {
                for ray3 in enum_iter::<Ray>() {
                    assert_eq!(
                        ray3.turn_one(ray),
                        ray3.turn_one(ray2),
                        "{:?} turned differently under {:?} and {:?}",
                        ray3,
                        ray,
                        ray2
                    );
                }
            }
        }
    }

    fn turns_permutations<Ray: RaySystem + std::fmt::Debug>() {
        for &ray in Ray::AXIS_HEADS {
            for ray2s in enum_iter::<Ray>().combinations(2) {
                assert!(
                    ray2s[0].turn_one(ray) != ray2s[1].turn_one(ray),
                    "{:?} and {:?} both turn to {:?} under {:?}",
                    ray2s[0],
                    ray2s[1],
                    ray2s[0].turn_one(ray),
                    ray
                );
            }
        }
    }

    fn turns_have_correct_order<Ray: RaySystem + std::fmt::Debug>() {
        for &ray in Ray::AXIS_HEADS {
            for ray2 in enum_iter::<Ray>() {
                let r = ray2.turn((ray, ray.order()));
                assert!(
                    ray2 == r,
                    "{:?} does not turn with order divisible by {:?} under {:?}",
                    ray2,
                    ray.order(),
                    ray
                );
            }
        }
    }

    pub fn validate_ray_system<Ray: RaySystem + std::fmt::Debug>() {
        axes_all_same_order::<Ray>();
        axis_heads_all_heads::<Ray>();
        turns_consistent_axis::<Ray>();
        turns_permutations::<Ray>();
        turns_have_correct_order::<Ray>();
    }
}
