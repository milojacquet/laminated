use crate::puzzle::cube::CubeRay;
use enum_map::Enum;
use std::fmt;

use crate::puzzle::common::RaySystem;
pub use crate::puzzle::common::{Basis, BasisDiff, Sign};

/// RDodecaRay(a, ±₁, ±₂) => ±₁a⁺ + ±₂a⁺⁺
#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub struct RDodecaRay(pub Basis, pub Sign, pub Sign);

impl RaySystem for RDodecaRay {
    fn get_axis(&self) -> Vec<Self> {
        vec![
            Self(self.0, Sign::Pos, self.1 * self.2),
            Self(self.0, Sign::Neg, -self.1 * self.2),
        ]
    }

    fn turn_one(&self, axis: Self) -> Self {
        let axis = axis.get_axis()[0];

        match self.0 - axis.0 {
            BasisDiff::D0 => Self(self.0, axis.1 * axis.2 * self.2, axis.1 * axis.2 * self.1),
            BasisDiff::D1 => Self(self.0 + BasisDiff::D1, -self.2, axis.1 * axis.2 * self.1),
            BasisDiff::D2 => Self(self.0 + BasisDiff::D2, axis.1 * axis.2 * self.2, -self.1),
        }
    }

    fn order() -> i8 {
        2
    }

    const AXIS_HEADS: &'static [Self] = &[
        Self(Basis::X, Sign::Pos, Sign::Pos),
        Self(Basis::X, Sign::Pos, Sign::Neg),
        Self(Basis::Y, Sign::Pos, Sign::Pos),
        Self(Basis::Y, Sign::Pos, Sign::Neg),
        Self(Basis::Z, Sign::Pos, Sign::Pos),
        Self(Basis::Z, Sign::Pos, Sign::Neg),
    ];

    #[rustfmt::skip]
    const CYCLE: &'static [(Self, i8)] = {
        use crate::puzzle::r_dodeca::name::*;

        &[

        ]
    };

    fn name(&self) -> String {
        format!(
            "{}{}",
            CubeRay(self.0 + BasisDiff::D1, self.1),
            CubeRay(self.0 + BasisDiff::D2, self.2)
        )
    }
}

#[allow(dead_code)]
pub mod name {
    use super::*;
    use crate::puzzle::cube::name as c;

    const fn add_cube_rays(c1: CubeRay, c2: CubeRay) -> RDodecaRay {
        //assert!(c2.0 - c1.0 == BasisDiff::D1);
        // that but constant
        let basis = match (c1.0, c2.0) {
            (Basis::X, Basis::Y) => Basis::Z,
            (Basis::Y, Basis::Z) => Basis::X,
            (Basis::Z, Basis::X) => Basis::Y,
            _ => panic!("invalid cube sum"),
        };
        RDodecaRay(basis, c1.1, c2.1)
    }

    const RF: RDodecaRay = add_cube_rays(c::R, c::F);
    const RB: RDodecaRay = add_cube_rays(c::R, c::B);
    const LF: RDodecaRay = add_cube_rays(c::L, c::F);
    const LB: RDodecaRay = add_cube_rays(c::L, c::B);
    const FU: RDodecaRay = add_cube_rays(c::F, c::U);
    const FD: RDodecaRay = add_cube_rays(c::F, c::D);
    const BU: RDodecaRay = add_cube_rays(c::B, c::U);
    const BD: RDodecaRay = add_cube_rays(c::B, c::D);
    const UR: RDodecaRay = add_cube_rays(c::U, c::R);
    const UL: RDodecaRay = add_cube_rays(c::U, c::L);
    const DR: RDodecaRay = add_cube_rays(c::D, c::R);
    const DL: RDodecaRay = add_cube_rays(c::D, c::L);
}

impl fmt::Display for RDodecaRay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::common::ray_system_tests::validate_ray_system;

    #[test]
    fn validate_ray_system_r_dodeca() {
        validate_ray_system::<RDodecaRay>()
    }
}
