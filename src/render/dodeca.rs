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

const SQ5: f32 = 2.2360679774997896964;
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
                    * (-(self.0 + BasisDiff::D1).to_vec() * self.1.to_f32()
                        + PHI * (self.0 + BasisDiff::D2).to_vec() * self.2.to_f32())
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

const SCALE_CIRCUMRAD: f32 = 0.5;
const SUPER_START: f32 = 0.85;

/*fn bary(c1: f32, c2: f32, c3: f32, c4: f32, c5: f32) -> Vec3 {
    use crate::puzzle::dodeca::name::PB;

    let mat = DodecaRay::axis_to_transform((PB, 1), BinaryConjugate::Id);
    let v1 = Vec3::new((3.0 * SQ5 - 5.0) / 2.0, SQ5, 0.0) * SCALE_CIRCUMRAD;
    let v2 = (mat * v1.extend(1.0)).truncate();
    let v3 = (mat * v1.extend(1.0)).truncate();
    let v4 = (mat * v1.extend(1.0)).truncate();
    let v5 = (mat * v1.extend(1.0)).truncate();
}*/

pub fn pentultimate_seeds(_prefs: &ConcretePuzzlePreferences) -> PuzzleSeed<DodecaRay> {
    use crate::puzzle::dodeca::name::*;

    let grips: Vec<Vec<i8>> = vec![vec![-1, 1], vec![1, -1]];

    let mut viewports: Vec<ViewportSeed<DodecaRay>> = vec![];

    let key_layers = vec![
        HashMap::from([(NUMBER_KEYS[0], vec![1, -1]), (NUMBER_KEYS[1], vec![-1, 1])]),
        HashMap::from([(NUMBER_KEYS[0], vec![-1, 1]), (NUMBER_KEYS[1], vec![1, -1])]),
    ];

    let abstract_viewport = AbstractViewport {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    let mut stickers: Vec<StickerSeed<DodecaRay>> = vec![];

    let v1 = Vec3::new(SQ5 / 2.0, (5.0 + SQ5) / 4.0, (5.0 - SQ5) / 4.0) * SCALE_CIRCUMRAD;
    let v2 = Vec3::new(0.0, SQ5, 0.0) * SCALE_CIRCUMRAD;
    let vcen = Vec3::new(0.0, PHI, 1.0) * SCALE_CIRCUMRAD;

    {
        let layers: enum_map::EnumMap<DodecaRay, i8> =
            enum_map! {PB=>1,BL=>1,BR=>1,PL=>1,PR=>1,PD=>1,F=>-1,DR=>-1,DL=>-1,R=>-1,L=>-1,U=>-1};

        // center
        stickers.push(StickerSeed {
            layers,
            face: PB,
            color: PB,
            vertices: vec![v1, v2, vcen],
        });
    }

    {
        let layers: enum_map::EnumMap<DodecaRay, i8> =
            enum_map! {PB=>1,DR=>1,BR=>1,PL=>1,PR=>1,PD=>1,F=>-1,BL=>-1,DL=>-1,R=>-1,L=>-1,U=>-1};

        // corner
        stickers.push(StickerSeed {
            layers,
            face: PB,
            color: PB,
            vertices: vec![
                Vec3::new((3.0 * SQ5 - 5.0) / 2.0, SQ5, 0.0) * SCALE_CIRCUMRAD,
                v2,
                v1,
            ],
        });
    }

    viewports.push(ViewportSeed {
        abstract_viewport,
        conjugate: BinaryConjugate::Id,
        stickers,
        key_layers: key_layers.clone(),
    });

    PuzzleSeed {
        grips,
        viewports,
        key_layers: key_layers.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::common::concrete_ray_system_tests::validate_concrete_ray_system;

    #[test]
    fn validate_concrete_ray_system_dodeca() {
        validate_concrete_ray_system::<DodecaRay>()
    }
}
