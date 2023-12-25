use crate::enum_iter;
use crate::preferences::ConcretePuzzlePreferences;
use crate::preferences::Preferences;
use crate::puzzle::common::RaySystem;
use crate::puzzle::common::{Basis, BasisDiff, Sign};
use crate::puzzle::r_dodeca::RDodecaRay;
use crate::render::common::*;
use crate::NUMBER_KEYS;
use enum_map::enum_map;
use std::collections::HashMap;
use std::f32::consts::PI;

use crate::util::{color, Vec3};
use cgmath::InnerSpace;

/*impl RDodecaRay {
    /// Maps self from conjugate to BinaryConjugate::Id.
    /// This is an abstract inverse of ray_to_vec.
    fn unconjugate(&self, conjugate: <Self as ConcreteRaySystem>::Conjugate) -> Self {
        match conjugate {
            BinaryConjugate::Id => *self,
            BinaryConjugate::Conj => {
                // in the conjugate, (X, ±₁, ±₂) -> -±₁Y + φ ±₂Z -> -±₁Z + φ ±₂Y
                //                   (Y, ±₁, ±₂) -> -±₁Z + φ ±₂X -> -±₁Y + φ ±₂X
                //                   (Z, ±₁, ±₂) -> -±₁X + φ ±₂Y -> -±₁X + φ ±₂Z
                let basis = match self.0 {
                    Basis::X => Basis::X,
                    Basis::Y => Basis::Z,
                    Basis::Z => Basis::Y,
                };
                DodecaRay(basis, -self.2, self.1)
            }
        }
    }

    /// Maps self from conjugate to BinaryConjugate::Id.
    /// This is an abstract version of ray_to_vec.
    fn conjugate(&self, conjugate: <Self as ConcreteRaySystem>::Conjugate) -> Self {
        match conjugate {
            BinaryConjugate::Id => *self,
            BinaryConjugate::Conj => {
                // in the conjugate, (X, ±₁, ±₂) -> -±₁Y + φ ±₂Z -> -±₁Z + φ ±₂Y
                //                   (Y, ±₁, ±₂) -> -±₁Z + φ ±₂X -> -±₁Y + φ ±₂X
                //                   (Z, ±₁, ±₂) -> -±₁X + φ ±₂Y -> -±₁X + φ ±₂Z
                let basis = match self.0 {
                    Basis::X => Basis::X,
                    Basis::Y => Basis::Z,
                    Basis::Z => Basis::Y,
                };
                DodecaRay(basis, self.2, -self.1)
            }
        }
    }

    fn opposite(&self) -> Self {
        Self(self.0, -self.1, -self.2)
    }
}*/

impl ConcreteRaySystem for RDodecaRay {
    type Conjugate = BinaryConjugate;

    fn order_conjugate(_conjugate: Self::Conjugate) -> i8 {
        1
    }

    fn turn_to_concrete((ray, order): (Self, i8), conjugate: Self::Conjugate) -> ConcreteTurn {
        match conjugate {
            BinaryConjugate::Id => ConcreteTurn::Rotation(
                ray.axis_to_vec(conjugate),
                order as f32 * 2.0 * PI / Self::order() as f32,
            ),
            BinaryConjugate::Conj => {
                ConcreteTurn::Reflection(Self(ray.0, ray.1, -ray.2).axis_to_vec(conjugate))
            }
        }
    }

    fn ray_to_vec(&self, conjugate: Self::Conjugate) -> Vec3 {
        // in the conjugate, (a, ±₁, ±₂) -> - ±₁a⁺ + ±₂a⁺⁺
        let sign = match conjugate {
            BinaryConjugate::Id => 1.0,
            BinaryConjugate::Conj => -1.0,
        };

        (sign * (self.0 + BasisDiff::D1).to_vec() * self.1.to_f32()
            + (self.0 + BasisDiff::D2).to_vec() * self.2.to_f32())
        .normalize()
    }

    fn default_colors() -> enum_map::EnumMap<Self, color::Color> {
        use crate::puzzle::dodeca::{name, DodecaRay};

        let default_dodeca = DodecaRay::default_colors();
        enum_map::EnumMap::from_fn(|ray: Self| {
            default_dodeca[DodecaRay(ray.0, ray.1, ray.2).turn((name::U, 0))]
        })
    }

    fn ray_to_color(prefs: &Preferences) -> &enum_map::EnumMap<Self, color::Color> {
        &prefs.colors.r_dodeca
    }

    fn ray_to_color_mut(prefs: &mut Preferences) -> &mut enum_map::EnumMap<Self, color::Color> {
        &mut prefs.colors.r_dodeca
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::common::concrete_ray_system_tests::validate_concrete_ray_system;

    #[test]
    fn validate_concrete_ray_system_r_dodeca() {
        validate_concrete_ray_system::<RDodecaRay>()
    }
}
