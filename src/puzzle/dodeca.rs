use enum_map::Enum;
use std::fmt;

use crate::puzzle::common::RaySystem;
pub use crate::puzzle::common::{Basis, BasisDiff, Sign};

/// DodecaRay(a, ±₁, ±₂) => φ ±₁a⁺ + ±₂a⁺⁺
#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub struct DodecaRay(pub Basis, pub Sign, pub Sign);

impl DodecaRay {
    fn get_flip(&self, basis: Basis) -> Sign {
        match basis - self.0 {
            BasisDiff::D0 => self.1 * self.2,
            BasisDiff::D1 => self.1,
            BasisDiff::D2 => self.2,
        }
    }

    fn flip_by_basis(&self, basis: Basis, sign: Sign) -> Self {
        match basis - self.0 {
            BasisDiff::D0 => *self,
            BasisDiff::D1 => Self(self.0, sign * self.1, self.2),
            BasisDiff::D2 => Self(self.0, self.1, sign * self.2),
        }
    }

    fn flip_by(&self, axis: Self) -> Self {
        let flip_x = axis.get_flip(Basis::X);
        let flip_y = axis.get_flip(Basis::Y);
        let flip_z = axis.get_flip(Basis::Z);
        self.flip_by_basis(Basis::X, flip_x)
            .flip_by_basis(Basis::Y, flip_y)
            .flip_by_basis(Basis::Z, flip_z)
    }

    fn cycle_to_x(&self, basis: Basis) -> Self {
        Self(self.0 + (Basis::X - basis), self.1, self.2)
    }

    fn cycle_from_x(&self, basis: Basis) -> Self {
        Self(self.0 - (Basis::X - basis), self.1, self.2)
    }
}

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
        // not actually used
        let axis = axis.get_axis()[0];

        // absolute coordinates: (Basis, Sign, Sign)
        // relative coordinates: that for which axis is (Basis::X, Sign::Pos, Sign::Pos)
        // to convert axis to relative: flip x, y, z so signs are both Pos, then cycle
        let rel_ray = self.flip_by(axis); //.cycle_to_x(axis.0);

        let transform = match (rel_ray.0 - axis.0, rel_ray.1 * rel_ray.2) {
            (BasisDiff::D0, Sign::Pos) => (BasisDiff::D0, Sign::Pos, Sign::Pos),
            (BasisDiff::D0, Sign::Neg) => (BasisDiff::D2, Sign::Neg, Sign::Neg),
            (BasisDiff::D1, Sign::Pos) => (BasisDiff::D1, Sign::Pos, Sign::Pos),
            (BasisDiff::D1, Sign::Neg) => (BasisDiff::D0, Sign::Pos, Sign::Neg),
            (BasisDiff::D2, Sign::Pos) => (BasisDiff::D1, Sign::Pos, Sign::Neg),
            (BasisDiff::D2, Sign::Neg) => (BasisDiff::D2, Sign::Neg, Sign::Neg),
        };
        //println!("{self:?} {axis:?} {transform:?}");
        DodecaRay(
            rel_ray.0 + transform.0,
            rel_ray.1 * transform.1,
            rel_ray.2 * transform.2,
        )
        //.cycle_from_x(axis.0)
        .flip_by(axis)
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
        use crate::puzzle::dodeca::name::*;

        // originaly this was with U,R,L but U and L are not axis heads
        // so it uses BR,R,BL instead
        // i'm not going to update the line comments

        // also i had to flip the signs of BR and BL but not R for some reason

        &[
            /* up U  front F  */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), (R, 1),
            /* up L  front DL */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), (R, 4),
            /* up F  front L  */ (BR, 1), (BR, 1), (BR, 1), (BR, 1), (R, 4),
            /* up R  front F  */ (BR, 1), (BR, 1), (BR, 1), (BR, 1), (R, 4),
            /* up BR front R  */ (BR, 1), (BR, 1), (BR, 1), (BR, 1), (R, 4),
            /* up BL front BR */ (BR, 1), (BR, 1), (BR, 1), (BR, 1), (R, 1),
            /* up PB front PD */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), (R, 4),
            /* up PL front PB */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), (BL, 4),
            /* up DL front PL */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), (BL, 4),
            /* up DR front DL */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), (BL, 4),
            /* up PR front DR */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), (R, 4),
            /* up PD front PR */ (BR, 4), (BR, 4), (BR, 4), (BR, 4), // (_, 1),
        ]
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
