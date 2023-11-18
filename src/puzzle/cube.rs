use enum_map::Enum;

use crate::puzzle::common::RaySystem;

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

    fn turn_one(&self, axis: Self) -> Self {
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
}

#[cfg(test)]
mod tests {
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
        use CubeRay::*;

        let mut puzzle = Puzzle::make_solved(vec![vec![0, 0], vec![1, 0], vec![0, 1]]);
        puzzle.twist((R, 1), &[1, 0]);
        assert!(!puzzle.is_solved());
    }

    /// Applies (R U R' U')6 to the 3x3x3 and asserts that it is solved.
    #[test]
    fn six_sexy() {
        use CubeRay::*;

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
