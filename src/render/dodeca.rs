use crate::preferences::ConcretePuzzlePreferences;
use crate::preferences::Preferences;
use crate::puzzle::common::Basis;
use crate::puzzle::common::BasisDiff;
use crate::puzzle::dodeca::DodecaRay;
use crate::render::common::*;
use crate::NUMBER_KEYS;
use enum_map::enum_map;
use std::collections::HashMap;
use std::f32::consts::PI;

use crate::util::{color, Mat3, Mat4, Vec3};
use cgmath::{InnerSpace, Rad};
use enum_map::Enum;

const PHI: f32 = 1.6180339887498948482; //0.5 * (1.0 + 5.0_f32.sqrt());

impl ConcreteRaySystem for DodecaRay {
    type Conjugate = BinaryConjugate;

    fn axis_to_transform((ray, order): (Self, i8), conjugate: Self::Conjugate) -> Mat4 {
        let multiplier = match conjugate {
            BinaryConjugate::Id => 1.0,
            BinaryConjugate::Conj => 2.0,
        };
        Mat4::from_axis_angle(
            ray.axis_to_vec(conjugate),
            Rad(order as f32 * 2.0 * PI * multiplier / 5.0),
        )
    }

    fn ray_to_vec(&self, conjugate: Self::Conjugate) -> Vec3 {
        // in the conjugate, (X, ±₁, ±₂) -> - ±₁a⁺ + φ ±₂a⁺⁺
        // however, we have to rotate 90° around an axis for the rays to line up
        // this will make puzzle construction easier

        match conjugate {
            BinaryConjugate::Id => (PHI * (self.0 + BasisDiff::D1).to_vec() * self.1.to_f32()
                + (self.0 + BasisDiff::D2).to_vec() * self.2.to_f32())
            .normalize(),
            BinaryConjugate::Conj => {
                Mat3::from_angle_x(Rad(PI / 2.0))
                    * (-(self.0 + BasisDiff::D2).to_vec() * self.1.to_f32()
                        + PHI * (self.0 + BasisDiff::D1).to_vec() * self.2.to_f32())
                    .normalize()
            }
        }
    }

    fn ray_to_color(prefs: &Preferences) -> &enum_map::EnumMap<Self, color::Color> {
        &prefs.colors.dodeca
    }

    fn ray_to_color_mut(prefs: &mut Preferences) -> &mut enum_map::EnumMap<Self, color::Color> {
        &mut prefs.colors.dodeca
    }
}

pub fn pentultimate_seeds(prefs: &ConcretePuzzlePreferences) -> PuzzleSeed<DodecaRay> {
    todo!()
}
