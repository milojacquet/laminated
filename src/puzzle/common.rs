use enum_map::{Enum, EnumMap};
use std::iter::zip;

use crate::util::*;

pub trait RaySystem
where
    Self: 'static + Sized + Eq + Copy + Enum + enum_map::EnumArray<u8> + enum_map::EnumArray<Self>,
{
    /// Returns a list of rays that make up the vector. Should
    /// return the same order for each axis.
    fn get_axis(&self) -> Vec<Self>;
    /// Turns the ray system one unit about ray's axis and returns the new ray
    /// that occupies self's direction.
    /// Should return the same value for any ray with the same axis.
    fn turn_one(&self, ray: &Self) -> Self;
    /// Turns the ray system about ray's axis and returns the new ray
    /// that occupies self's direction.
    /// Should return the same value for any ray with the same axis.
    fn turn(&self, (ray, order): &(Self, i8)) -> Self {
        let mut turned = *self;
        for _ in 0..order.rem_euclid(ray.order()) {
            turned = turned.turn_one(&ray);
        }
        turned
    }
    /// Gets the order of the turn
    fn order(&self) -> i8;
    /// Returns a list of rays, each one of which is the first ray of its axis.
    const AXIS_HEADS: &'static [Self];
    /// Hamiltonian cycle for symmetry group
    const CYCLE: &'static [(Self, i8)];
}

/// A single piece of an abstract laminated puzzle.
#[derive(Debug)]
pub struct Piece<Ray>
where
    Ray: RaySystem,
{
    /// For each ray, the layer index on that ray (solved position).
    pub layers: EnumMap<Ray, u8>,
    /// For each ray, a tuple of the ray currently occupying
    /// that direction and its layer parameter.
    pub orientation: EnumMap<Ray, Ray>,
}

impl<Ray: RaySystem> Piece<Ray> {
    pub fn make_solved(axis_layers: Vec<&[u8]>) -> Self {
        let mut layers = EnumMap::from_fn(|_ray| 0);
        for (axh, axl) in zip(Ray::AXIS_HEADS, axis_layers) {
            for (ray, &layer) in zip(axh.get_axis(), axl) {
                layers[ray] = layer;
            }
        }

        Self::make_solved_from_layers(layers)
    }

    pub fn make_solved_from_layers(layers: EnumMap<Ray, u8>) -> Self {
        Self {
            layers,
            orientation: EnumMap::from_fn(|ray| ray),
        }
    }

    pub fn is_solved(&self) -> bool {
        self.orientation.iter().all(|(pos, &cur)| pos == cur)
    }

    /*pub fb grip_on_axis_enummap(layers: &EnumMap<Ray, u8>, ray: &Ray) -> Vec<u8>{
        ray.get_axis()
            .iter()
            .map(|&r| self.layers[self.orientation[r]])
            .collect()
    }*/

    pub fn grip_on_axis(&self, ray: &Ray) -> Vec<u8> {
        ray.get_axis()
            .iter()
            .map(|&r| self.layers[self.orientation[r]])
            .collect()
    }

    pub fn grip_on_axis_solved(&self, ray: &Ray) -> Vec<u8> {
        ray.get_axis().iter().map(|&r| self.layers[r]).collect()
    }

    pub fn twist(&mut self, (ray, order): (Ray, i8), grip: &[u8]) {
        if &self.grip_on_axis(&ray)[..] == grip {
            let new_orientation = EnumMap::from_fn(|r:Ray| self.orientation[r.turn(&(ray, order))]);
            self.orientation = new_orientation;
            /*for (_, cur) in self.orientation.iter_mut() {
                *cur = cur.turn(&(ray, order));
            }*/
        }
    }

    pub fn oriented_layers(&self) -> EnumMap<Ray, u8> {
        EnumMap::from_fn(|ray| self.layers[self.orientation[ray]])
    }
}

/// Abstract laminated puzzle.
#[derive(Debug)]
pub struct Puzzle<'a, Ray: RaySystem> {
    pub grips: Vec<&'a [u8]>,
    pub pieces: Vec<Piece<Ray>>,
    /// number at index i is the index of the piece that occupies position i
    pub permutation: Vec<usize>,
}

impl<'a, Ray: RaySystem> Puzzle<'a, Ray> {
    pub fn piece_count(&self) -> usize {
        self.grips.len().pow(Ray::AXIS_HEADS.len() as u32)
    }

    pub fn make_solved(grips: Vec<&'a [u8]>) -> Self {
        let mut new = Self {
            grips: grips.clone(),
            permutation: Vec::new(),
            pieces: Vec::new(),
        };
        let piece_count: usize = new.grips.len().pow(Ray::AXIS_HEADS.len() as u32);
        new.pieces = (0..piece_count)
            .map(|i| new.index_to_solved_piece(i))
            .collect();
        new.permutation = (0..piece_count).collect();
        return new;
    }

    pub fn is_solved(&self) -> bool {
        self.pieces.iter().all(|piece| piece.is_solved())
    }

    pub fn twist(&mut self, (ray, order): (Ray, i8), grip: &[u8]) {
        if grip == self.grips[0] {
            // we cannot move the core
            let other_layers: Vec<_> = self.grips.iter().skip(1).copied().collect();
            for new_grip in other_layers {
                self.twist((ray, order), new_grip);
            }
        } else {
            for i in 0..self.pieces.len() {
                self.pieces[i].twist((ray, order), grip);
                //self.permutation[i] = self.piece_to_index(&self.pieces[i]);
                let piece_index = self.piece_to_index(&self.pieces[i]);
                self.permutation[piece_index] = i;
            }
        }
    }

    /// Returns a new solved piece whose index is the provided usize.
    pub fn index_to_solved_piece(&self, i: usize) -> Piece<Ray> {
        let grip_count: usize = self.grips.len();
        Piece::make_solved(
            (0..Ray::AXIS_HEADS.len())
                .map(|j| self.grips[i / grip_count.pow(j as u32) % grip_count])
                .collect(),
        )
    }

    /// Gets the index of the current position of the piece.
    pub fn piece_to_index(&self, piece: &Piece<Ray>) -> usize {
        Ray::AXIS_HEADS
            .iter()
            .enumerate()
            .map(|(j, r)| {
                self.grips
                    .iter()
                    .position(|gr| &&piece.grip_on_axis(r)[..] == gr)
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
            .map(|(j, r)| {
                self.grips
                    .iter()
                    .position(|gr| &&piece.grip_on_axis_solved(r)[..] == gr)
                    .expect("grips should all exist because the piece should be valid")
                    * self.grips.len().pow(j as u32)
            })
            .sum()
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
        for ray in Ray::AXIS_HEADS {
            for ray2 in ray.get_axis() {
                for ray3 in enum_iter::<Ray>() {
                    assert_eq!(
                        ray3.turn_one(&ray),
                        ray3.turn_one(&ray2),
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
        for ray in Ray::AXIS_HEADS {
            for ray2s in enum_iter::<Ray>().iter().combinations(2) {
                assert!(
                    ray2s[0].turn_one(ray) != ray2s[1].turn_one(ray),
                    "{:?} and {:?} turn the same under {:?}",
                    ray2s[0],
                    ray2s[1],
                    ray
                );
            }
        }
    }

    /*// not technically the ray system
    fn index_piece_inverses<Ray: RaySystem + std::fmt::Debug>(){

    }*/

    pub fn validate_ray_system<Ray: RaySystem + std::fmt::Debug>() {
        axes_all_same_order::<Ray>();
        axis_heads_all_heads::<Ray>();
        turns_consistent_axis::<Ray>();
        turns_permutations::<Ray>();
    }
}
