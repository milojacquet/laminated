use enum_map::Enum;
use std::fmt;

use crate::puzzle::common::RaySystem;
pub use crate::puzzle::common::{Basis, BasisDiff, Sign};

/// +X: R, +Y: U, +Z: F
#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub struct CubeRay(pub Basis, pub Sign);

impl RaySystem for CubeRay {
    fn get_axis(&self) -> Vec<Self> {
        vec![CubeRay(self.0, Sign::Pos), CubeRay(self.0, Sign::Neg)]
    }

    fn turn_one(&self, axis: Self) -> Self {
        match axis.0 - self.0 {
            BasisDiff::D0 => *self,
            BasisDiff::D1 => CubeRay(self.0 + BasisDiff::D2, -self.1),
            BasisDiff::D2 => CubeRay(self.0 + BasisDiff::D1, self.1),
        }
    }

    fn order(&self) -> i8 {
        4
    }

    //const AXIS_HEADS: &'static [Self] = &[CubeRay::U, CubeRay::F, CubeRay::R];
    const AXIS_HEADS: &'static [Self] = &[
        CubeRay(Basis::X, Sign::Pos),
        CubeRay(Basis::Y, Sign::Pos),
        CubeRay(Basis::Z, Sign::Pos),
    ];

    #[rustfmt::skip]
    const CYCLE: &'static [(Self, i8)] = {
        use name::*;
        // 3u 3u 3u 3f 3u 3u 3u 3f 3u 3u 3u 3f' 3u 3u 3u 3f' 3u 3u 3u 3f 3u' 3u' 3u' 3f'
        &[
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1), (F, 3),
            (U, 1), (U, 1), (U, 1), (F, 3),
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 3), (U, 3), (U, 3), //(F, 3),
        ]
    };

    fn name(&self) -> String {
        match self {
            CubeRay(Basis::X, Sign::Pos) => "R".to_string(),
            CubeRay(Basis::Y, Sign::Pos) => "B".to_string(),
            CubeRay(Basis::Z, Sign::Pos) => "U".to_string(),
            CubeRay(Basis::X, Sign::Neg) => "L".to_string(),
            CubeRay(Basis::Y, Sign::Neg) => "F".to_string(),
            CubeRay(Basis::Z, Sign::Neg) => "D".to_string(),
        }
    }
}

impl fmt::Display for CubeRay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Sorry everyone, I'm using z-up like a normal person
pub mod name {
    use super::*;

    pub const R: CubeRay = CubeRay(Basis::X, Sign::Pos);
    pub const B: CubeRay = CubeRay(Basis::Y, Sign::Pos);
    pub const U: CubeRay = CubeRay(Basis::Z, Sign::Pos);
    pub const L: CubeRay = CubeRay(Basis::X, Sign::Neg);
    pub const F: CubeRay = CubeRay(Basis::Y, Sign::Neg);
    pub const D: CubeRay = CubeRay(Basis::Z, Sign::Neg);
}

#[cfg(test)]
mod tests {
    use super::name::*;
    use super::*;
    use crate::puzzle::common::ray_system_tests::validate_ray_system;
    use crate::puzzle::common::Puzzle;

    #[test]
    fn validate_ray_system_cube() {
        validate_ray_system::<CubeRay>()
    }

    /// Applies one turn and asserts that it is unsolved.
    #[test]
    fn one_turn() {
        let mut puzzle = Puzzle::make_solved(vec![vec![0, 0], vec![1, 0], vec![0, 1]]);
        puzzle.twist((R, 1), &[1, 0]);
        assert!(!puzzle.is_solved());
    }

    /// Applies (R U R' U')6 to the 3x3x3 and asserts that it is solved.
    #[test]
    fn six_sexy() {
        let mut puzzle = Puzzle::make_solved(vec![vec![0, 0], vec![1, 0], vec![0, 1]]);
        for _ in 0..6 {
            puzzle.twist((R, 1), &[1, 0]);
            puzzle.twist((R, 1), &[1, 0]);
            puzzle.twist((R, -1), &[1, 0]);
            puzzle.twist((R, -1), &[1, 0]);
        }
        assert!(puzzle.is_solved());
    }
}
