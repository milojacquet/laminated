use enum_map::Enum;
use std::fmt;

use crate::puzzle::common::RaySystem;
pub use crate::puzzle::common::{Basis, BasisDiff, Sign};

/// DodecaRay(a, ±₁, ±₂) => φ ±₁a⁺ + ±₂a⁺⁺
#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub struct DodecaRay(pub Basis, pub Sign, pub Sign);

impl RaySystem for DodecaRay {
    fn get_axis(&self) -> Vec<Self> {
        vec![
            DodecaRay(self.0, Sign::Pos, self.1 * self.2),
            DodecaRay(self.0, Sign::Neg, -self.1 * self.2),
        ]
    }

    fn turn_one(&self, axis: Self) -> Self {
        // rotation around [0, φ, 1] by 2π/5:
        // 1/2 [[1/φ, -1 , φ  ],
        //      [1  , φ  , 1/φ],
        //      [-φ , 1/φ, 1  ]]
        let axis = axis.get_axis()[0];
        let transform = match (self.0 - axis.0, self.1 * axis.1 * self.2 * axis.2) {
            (BasisDiff::D0, Sign::Pos) => (BasisDiff::D0, Sign::Pos, Sign::Pos),
            (BasisDiff::D0, Sign::Neg) => (BasisDiff::D2, Sign::Neg, Sign::Neg),
            (BasisDiff::D1, Sign::Pos) => (BasisDiff::D1, Sign::Pos, Sign::Pos),
            (BasisDiff::D1, Sign::Neg) => (BasisDiff::D0, Sign::Pos, Sign::Neg),
            (BasisDiff::D2, Sign::Pos) => (BasisDiff::D1, Sign::Pos, Sign::Neg),
            (BasisDiff::D2, Sign::Neg) => (BasisDiff::D2, Sign::Neg, Sign::Neg),
        };
        //println!("{self:?} {axis:?} {transform:?}");
        DodecaRay(
            self.0 + transform.0,
            self.1 * transform.1,
            self.2 * transform.2,
        )
    }

    fn order(&self) -> i8 {
        5
    }

    const AXIS_HEADS: &'static [Self] = &[
        DodecaRay(Basis::X, Sign::Pos, Sign::Pos),
        DodecaRay(Basis::X, Sign::Pos, Sign::Neg),
        DodecaRay(Basis::Y, Sign::Pos, Sign::Pos),
        DodecaRay(Basis::Y, Sign::Pos, Sign::Neg),
        DodecaRay(Basis::Z, Sign::Pos, Sign::Pos),
        DodecaRay(Basis::Z, Sign::Pos, Sign::Neg),
    ];

    #[rustfmt::skip]
    const CYCLE: &'static [(Self, i8)] = {
        &[]
    };

    fn name(&self) -> String {
        match self {
            DodecaRay(Basis::X, Sign::Pos, Sign::Pos) => "PB",
            DodecaRay(Basis::X, Sign::Pos, Sign::Neg) => "PD",
            DodecaRay(Basis::X, Sign::Neg, Sign::Pos) => "U",
            DodecaRay(Basis::X, Sign::Neg, Sign::Neg) => "F",
            DodecaRay(Basis::Y, Sign::Pos, Sign::Pos) => "BL",
            DodecaRay(Basis::Y, Sign::Pos, Sign::Neg) => "BR",
            DodecaRay(Basis::Y, Sign::Neg, Sign::Pos) => "DL",
            DodecaRay(Basis::Y, Sign::Neg, Sign::Neg) => "DR",
            DodecaRay(Basis::Z, Sign::Pos, Sign::Pos) => "PR",
            DodecaRay(Basis::Z, Sign::Pos, Sign::Neg) => "R",
            DodecaRay(Basis::Z, Sign::Neg, Sign::Pos) => "PL",
            DodecaRay(Basis::Z, Sign::Neg, Sign::Neg) => "L",
        }
        .to_string()
    }
}

pub mod name {
    use super::*;

    pub const PB: DodecaRay = DodecaRay(Basis::X, Sign::Pos, Sign::Pos);
    pub const PD: DodecaRay = DodecaRay(Basis::X, Sign::Pos, Sign::Neg);
    pub const U: DodecaRay = DodecaRay(Basis::X, Sign::Neg, Sign::Pos);
    pub const F: DodecaRay = DodecaRay(Basis::X, Sign::Neg, Sign::Neg);
    pub const BL: DodecaRay = DodecaRay(Basis::Y, Sign::Pos, Sign::Pos);
    pub const BR: DodecaRay = DodecaRay(Basis::Y, Sign::Pos, Sign::Neg);
    pub const DL: DodecaRay = DodecaRay(Basis::Y, Sign::Neg, Sign::Pos);
    pub const DR: DodecaRay = DodecaRay(Basis::Y, Sign::Neg, Sign::Neg);
    pub const PR: DodecaRay = DodecaRay(Basis::Z, Sign::Pos, Sign::Pos);
    pub const R: DodecaRay = DodecaRay(Basis::Z, Sign::Pos, Sign::Neg);
    pub const PL: DodecaRay = DodecaRay(Basis::Z, Sign::Neg, Sign::Pos);
    pub const L: DodecaRay = DodecaRay(Basis::Z, Sign::Neg, Sign::Neg);
}

impl fmt::Display for DodecaRay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::common::ray_system_tests::validate_ray_system;

    #[test]
    fn validate_ray_system_dodeca() {
        validate_ray_system::<DodecaRay>()
    }
}
