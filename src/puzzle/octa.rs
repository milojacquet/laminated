use enum_map::Enum;
use std::fmt;

use crate::puzzle::common::RaySystem;
pub use crate::puzzle::common::{Basis, BasisDiff, Sign};

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub struct OctaRay(Sign, Sign, Sign);

impl OctaRay {
    fn opposite(&self) -> Self {
        OctaRay(-self.0, -self.1, -self.2)
    }

    fn tet_sign(&self) -> Sign {
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
        OctaRay(axis.2 * self.1, axis.0 * self.2, axis.1 * self.0)
    }

    fn order(&self) -> i8 {
        3
    }

    const AXIS_HEADS: &'static [Self] = &[
        OctaRay(Sign::Pos, Sign::Pos, Sign::Pos),
        OctaRay(Sign::Pos, Sign::Neg, Sign::Neg),
        OctaRay(Sign::Neg, Sign::Pos, Sign::Neg),
        OctaRay(Sign::Neg, Sign::Neg, Sign::Pos),
    ];

    #[rustfmt::skip]
    const CYCLE: &'static [(Self, i8)] = {todo!()
    };

    fn name(&self) -> String {
        todo!()
    }
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
