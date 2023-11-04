use enum_map::{Enum, EnumMap};
use std::iter::zip;

pub trait RaySystem
where
    Self: 'static + Sized + Eq + Copy + Enum + enum_map::EnumArray<u8> + enum_map::EnumArray<Self>,
{
    /// Returns a list of rays that make up the vector. Should
    /// return the same order for each axis.
    fn get_axis(&self) -> Vec<Self>;
    /// Turns the ray system about ray's axis and returns the new ray
    /// that occupies self's direction.
    /// Should return the same value for any ray with the same axis.
    fn turn(&self, ray: &Self) -> Self;
    /// Gets the order of the turn
    fn order(&self) -> i8;
    /// Returns a list of rays, each one of which is the first ray of its axis.
    const AXIS_HEADS: &'static [Self];
    /// Hamiltonian cycle for symmetry group
    const CYCLE: &'static [(Self, i8)];
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub enum CubeRay {
    U,
    D,
    F,
    B,
    R,
    L,
}

impl RaySystem for CubeRay {
    fn get_axis(&self) -> Vec<Self> {
        use CubeRay::*;

        match self {
            U | D => vec![U, D],
            F | B => vec![F, B],
            R | L => vec![R, L],
        }
    }

    //fn axis_heads

    fn turn(&self, axis: &Self) -> Self {
        use CubeRay::*;

        match (axis, self) {
            (U | D, U) => U,
            (U | D, D) => D,
            (U | D, F) => R,
            (U | D, B) => L,
            (U | D, R) => B,
            (U | D, L) => F,

            (F | B, U) => L,
            (F | B, D) => R,
            (F | B, F) => F,
            (F | B, B) => B,
            (F | B, R) => U,
            (F | B, L) => D,

            (R | L, U) => F,
            (R | L, D) => B,
            (R | L, F) => D,
            (R | L, B) => U,
            (R | L, R) => R,
            (R | L, L) => L,
        }
    }

    fn order(&self) -> i8 {
        4
    }

    const AXIS_HEADS: &'static [Self] = &[CubeRay::U, CubeRay::F, CubeRay::R];

    #[rustfmt::skip]
    const CYCLE: &'static [(Self, i8)] = {
        use CubeRay::*;
        // 3u 3u 3u 3f 3u 3u 3u 3f 3u 3u 3u 3f' 3u 3u 3u 3f' 3u 3u 3u 3f 3u 3u 3u
        &[
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1), (F, 3),
            (U, 1), (U, 1), (U, 1), (F, 3),
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1),
        ]
    };
}

/// A single piece of an abstract laminated puzzle.
#[derive(Debug)]
pub struct Piece<Ray>
where
    Ray: RaySystem,
{
    /// For each ray, the layer index on that ray (solved position).
    layers: EnumMap<Ray, u8>,
    /// For each ray, a tuple of the ray currently occupying
    /// that direction and its layer parameter.
    orientation: EnumMap<Ray, Ray>,
}

impl<Ray: RaySystem> Piece<Ray> {
    fn make_solved(axis_layers: Vec<&[u8]>) -> Self {
        let mut layers = EnumMap::from_fn(|_ray| 0);
        for (axh, axl) in zip(Ray::AXIS_HEADS, axis_layers) {
            for (ray, &layer) in zip(axh.get_axis(), axl) {
                layers[ray] = layer;
            }
        }

        Self {
            layers,
            orientation: EnumMap::from_fn(|ray| ray),
        }
    }

    fn is_solved(&self) -> bool {
        self.orientation.iter().all(|(pos, &cur)| pos == cur)
    }

    fn twist(&mut self, (ray, order): (Ray, i8), grip: &[u8]) {
        let axis = ray.get_axis();
        let order = if order < 0 {
            ray.order() + order
        } else {
            order
        };
        if zip(axis, grip).all(|(r, &i)| self.layers[self.orientation[r]] == i) {
            for _ in 0..order {
                for (_, cur) in self.orientation.iter_mut() {
                    *cur = cur.turn(&ray);
                }
            }
        }
    }
}

/// Abstract laminated puzzle.
#[derive(Debug)]
pub struct Puzzle<'a, Ray: RaySystem> {
    pub grips: Vec<&'a [u8]>,
    pub pieces: Vec<Piece<Ray>>,
}

impl<'a, Ray: RaySystem> Puzzle<'a, Ray> {
    pub fn make_solved(grips: Vec<&'a [u8]>) -> Self {
        let grip_count: usize = grips.len();
        let piece_count: usize = grip_count.pow(Ray::AXIS_HEADS.len().try_into().unwrap());
        let pieces = (0..piece_count)
            .map(|i| {
                Piece::make_solved(
                    (0..Ray::AXIS_HEADS.len())
                        .map(|j| grips[i / grip_count.pow(j.try_into().unwrap()) % grip_count])
                        .collect(),
                )
            })
            .collect();
        Self { grips, pieces }
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
            for piece in &mut self.pieces {
                piece.twist((ray, order), grip);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Applies one turn and asserts that it is unsolved..
    #[test]
    fn one_turn() {
        use CubeRay::*;

        let mut puzzle = Puzzle::make_solved(vec![&[0, 0], &[1, 0], &[0, 1]]);
        puzzle.twist((R, 1), &[1, 0]);
        assert!(!puzzle.is_solved());
    }

    /// Applies (R U R' U')6 to the 3x3x3 and asserts that it is solved.
    #[test]
    fn six_sexy() {
        use CubeRay::*;

        let mut puzzle = Puzzle::make_solved(vec![&[0, 0], &[1, 0], &[0, 1]]);
        for _ in 0..6 {
            puzzle.twist((R, 1), &[1, 0]);
            puzzle.twist((R, 1), &[1, 0]);
            puzzle.twist((R, -1), &[1, 0]);
            puzzle.twist((R, -1), &[1, 0]);
        }
        assert!(puzzle.is_solved());
    }
}
