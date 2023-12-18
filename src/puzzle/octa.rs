use enum_map::Enum;
use std::fmt;

use crate::puzzle::common::RaySystem;
pub use crate::puzzle::common::{Basis, BasisDiff, Sign};

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub struct OctaRay(pub Sign, pub Sign, pub Sign);

impl OctaRay {
    fn opposite(&self) -> Self {
        OctaRay(-self.0, -self.1, -self.2)
    }

    pub fn tet_sign(&self) -> Sign {
        self.0 * self.1 * self.2
    }
}

impl RaySystem for OctaRay {
    fn get_axis(&self) -> Vec<Self> {
        if self.tet_sign() == Sign::Pos {
            vec![*self, self.opposite()]
        } else {
            vec![self.opposite(), *self]
        }
    }

    fn turn_one(&self, axis: Self) -> Self {
        // an axis head
        let axis = axis.get_axis()[0];
        OctaRay(axis.1 * self.2, axis.2 * self.0, axis.0 * self.1)
    }

    fn order() -> i8 {
        3
    }

    const AXIS_HEADS: &'static [Self] = &[
        OctaRay(Sign::Pos, Sign::Pos, Sign::Pos),
        OctaRay(Sign::Pos, Sign::Neg, Sign::Neg),
        OctaRay(Sign::Neg, Sign::Pos, Sign::Neg),
        OctaRay(Sign::Neg, Sign::Neg, Sign::Pos),
    ];

    #[rustfmt::skip]
    const CYCLE: &'static [(Self, i8)] = {
        #[allow(non_snake_case)]
        let r_BU = OctaRay(Sign::Pos, Sign::Pos, Sign::Pos);
        #[allow(non_snake_case)]
        let r_R = OctaRay(Sign::Pos, Sign::Neg, Sign::Neg);

        &[
            (r_BU, 1), (r_BU, 1), (r_R, 1),
            (r_BU, 1), (r_BU, 1), (r_R, 2),
            (r_BU, 1), (r_BU, 1), (r_R, 2),
            (r_BU, 1), (r_BU, 1), //(__, 2),
        ]
    };

    fn name(&self) -> String {
        match self {
            OctaRay(Sign::Pos, Sign::Neg, Sign::Pos) => "U",
            OctaRay(Sign::Pos, Sign::Neg, Sign::Neg) => "R",
            OctaRay(Sign::Neg, Sign::Neg, Sign::Pos) => "L",
            OctaRay(Sign::Neg, Sign::Neg, Sign::Neg) => "F",
            OctaRay(Sign::Pos, Sign::Pos, Sign::Pos) => "BU",
            OctaRay(Sign::Pos, Sign::Pos, Sign::Neg) => "BR",
            OctaRay(Sign::Neg, Sign::Pos, Sign::Pos) => "BL",
            OctaRay(Sign::Neg, Sign::Pos, Sign::Neg) => "D",
        }
        .to_string()
    }
}

pub mod name {
    use super::*;

    pub const U: OctaRay = OctaRay(Sign::Pos, Sign::Neg, Sign::Pos);
    pub const R: OctaRay = OctaRay(Sign::Pos, Sign::Neg, Sign::Neg);
    pub const L: OctaRay = OctaRay(Sign::Neg, Sign::Neg, Sign::Pos);
    pub const F: OctaRay = OctaRay(Sign::Neg, Sign::Neg, Sign::Neg);
    pub const BU: OctaRay = OctaRay(Sign::Pos, Sign::Pos, Sign::Pos);
    pub const BR: OctaRay = OctaRay(Sign::Pos, Sign::Pos, Sign::Neg);
    pub const BL: OctaRay = OctaRay(Sign::Neg, Sign::Pos, Sign::Pos);
    pub const D: OctaRay = OctaRay(Sign::Neg, Sign::Pos, Sign::Neg);
}

impl fmt::Display for OctaRay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::common::ray_system_tests::validate_ray_system;

    #[test]
    fn validate_ray_system_octa() {
        validate_ray_system::<OctaRay>()
    }
}
