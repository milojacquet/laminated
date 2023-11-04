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
                        ray3.turn(&ray),
                        ray3.turn(&ray2),
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
                    ray2s[0].turn(ray) != ray2s[1].turn(ray),
                    "{:?} and {:?} turn the same under {:?}",
                    ray2s[0],
                    ray2s[1],
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
    }
}
