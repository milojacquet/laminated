use enum_map::Enum;

use crate::puzzle::common::RaySystem;

/// Binary numbering of axes; false if UFR, true if DBL.
#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub struct OctaRay(bool, bool, bool);

impl RaySystem for OctaRay {
    fn get_axis(&self) -> Vec<Self> {
        match self {
            OctaRay(true, b1, b2) => vec![OctaRay(true, *b1, *b2), OctaRay(false, !*b1, !*b2)],
            OctaRay(false, b1, b2) => vec![OctaRay(true, !*b1, !*b2), OctaRay(false, *b1, *b2)],
        }
    }

    fn turn_one(&self, axis: Self) -> Self {
        match axis {
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
        }
    }

    fn order(&self) -> i8 {
        3
    }

    const AXIS_HEADS: &'static [Self] = &[
        OctaRay(true, true, true),
        OctaRay(true, true, false),
        OctaRay(true, false, true),
        OctaRay(true, false, false),
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
