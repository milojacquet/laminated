use enum_map::Enum;

use crate::puzzle::common::RaySystem;
pub use crate::puzzle::common::{Basis, BasisDiff, Sign};

/// Binary numbering of axes; false if UFR, true if DBL.
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
        /*match axis {
            OctaRay(true, true, true) | OctaRay(false, false, false) => {
                OctaRay(self.1, self.2, self.0)
            }
            OctaRay(true, true, false) | OctaRay(false, false, true) => {
                OctaRay(!self.2, self.0, !self.1)
            }
            OctaRay(true, false, true) | OctaRay(false, true, false) => {
                OctaRay(self.2, !self.0, !self.1)
            }
            OctaRay(true, false, false) | OctaRay(false, true, true) => {
                OctaRay(!self.1, self.2, !self.0)
            }
        }*/
        // an axis head
        let axis = axis.get_axis()[0];
        if axis == OctaRay(Sign::Pos, Sign::Pos, Sign::Pos) {
            OctaRay(self.1, self.2, self.0)
        } else {
            OctaRay(axis.2 * self.1, axis.0 * self.2, axis.1 * self.0)
        }
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
    const CYCLE: &'static [(Self, i8)] = {
        &[
            /*(U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1), (F, 3),
            (U, 1), (U, 1), (U, 1), (F, 3),
            (U, 1), (U, 1), (U, 1), (F, 1),
            (U, 1), (U, 1), (U, 1),*/
        ]
    };
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
