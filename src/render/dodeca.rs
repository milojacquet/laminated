use crate::enum_iter;
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

const SQ5: f32 = 2.2360679774997896964;
const PHI: f32 = 1.6180339887498948482; //0.5 * (1.0 + 5.0_f32.sqrt());

impl DodecaRay {
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
}

impl ConcreteRaySystem for DodecaRay {
    type Conjugate = BinaryConjugate;

    fn order_conjugate(conjugate: Self::Conjugate) -> i8 {
        match conjugate {
            BinaryConjugate::Id => 1,
            BinaryConjugate::Conj => -2,
        }
    }

    fn ray_to_vec(&self, conjugate: Self::Conjugate) -> Vec3 {
        // in the conjugate, (a, ±₁, ±₂) -> - ±₁a⁺ + φ ±₂a⁺⁺
        // however, we will swap Y and Z to make puzzle construction easier

        match conjugate {
            BinaryConjugate::Id => (PHI * (self.0 + BasisDiff::D1).to_vec() * self.1.to_f32()
                + (self.0 + BasisDiff::D2).to_vec() * self.2.to_f32())
            .normalize(),
            BinaryConjugate::Conj => {
                let vec = (-(self.0 + BasisDiff::D1).to_vec() * self.1.to_f32()
                    + PHI * (self.0 + BasisDiff::D2).to_vec() * self.2.to_f32())
                .normalize();
                Vec3::new(vec.x, vec.z, vec.y)
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

const SCALE_CIRCUMRAD: f32 = 0.7;
const SUPER_START: f32 = 0.7;

fn bary(c1: f32, c2: f32, c3: f32, c4: f32, c5: f32) -> Vec3 {
    use crate::puzzle::dodeca::name::PB;

    let mat = DodecaRay::turn_to_transform((PB, 1), BinaryConjugate::Id);
    let v1 = Vec3::new((3.0 * SQ5 - 5.0) / 2.0, SQ5, 0.0) * SCALE_CIRCUMRAD;
    let v2 = (mat * v1.extend(1.0)).truncate();
    let v3 = (mat * v2.extend(1.0)).truncate();
    let v4 = (mat * v3.extend(1.0)).truncate();
    let v5 = (mat * v4.extend(1.0)).truncate();
    (c1 * v1 + c2 * v2 + c3 * v3 + c4 * v4 + c5 * v5) / (c1 + c2 + c3 + c4 + c5)
}

pub fn pentultimate_seeds(_prefs: &ConcretePuzzlePreferences) -> PuzzleSeed<DodecaRay> {
    use crate::puzzle::dodeca::name::*;

    let grips: Vec<Vec<i8>> = vec![vec![-1, 1], vec![1, -1]];

    let mut viewports: Vec<ViewportSeed<DodecaRay>> = vec![];

    let key_layers = vec![
        HashMap::from([(NUMBER_KEYS[0], vec![1, -1]), (NUMBER_KEYS[1], vec![-1, 1])]),
        HashMap::from([(NUMBER_KEYS[0], vec![-1, 1]), (NUMBER_KEYS[1], vec![1, -1])]),
    ];

    for conj in enum_iter::<BinaryConjugate>() {
        let x = match conj {
            BinaryConjugate::Id => 0.0,
            BinaryConjugate::Conj => 1.0,
        };

        let abstract_viewport = AbstractViewport {
            x,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };

        let mut stickers: Vec<StickerSeed<DodecaRay>> = vec![];

        {
            let layers: enum_map::EnumMap<DodecaRay, i8> =
                enum_map::EnumMap::from_fn(|ray: DodecaRay| match ray.unconjugate(conj) {
                    PB => 1,
                    BL => 1,
                    BR => 1,
                    PL => 1,
                    PR => 1,
                    PD => 1,
                    F => -1,
                    DR => -1,
                    DL => -1,
                    R => -1,
                    L => -1,
                    U => -1,
                });

            // center
            stickers.push(StickerSeed {
                layers,
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    bary(SUPER_START, 1.0, 1.0 - SUPER_START, 0.0, 0.0),
                    bary(1.0 - SUPER_START, 1.0, SUPER_START, 0.0, 0.0),
                    bary(0.0, SUPER_START, 1.0, 1.0 - SUPER_START, 0.0),
                    bary(1.0, 1.0, 1.0, 1.0, 1.0),
                ],
            });
            stickers.push(StickerSeed {
                layers,
                face: PB.conjugate(conj),
                color: PL.conjugate(conj),
                vertices: vec![
                    bary(0.0, SUPER_START, 1.0, 1.0 - SUPER_START, 0.0),
                    bary(1.0 - SUPER_START, 1.0, SUPER_START, 0.0, 0.0),
                    bary(0.0, 1.0, 1.0, 0.0, 0.0),
                ],
            });
        }

        {
            let layers: enum_map::EnumMap<DodecaRay, i8> =
                enum_map::EnumMap::from_fn(|ray: DodecaRay| match ray.unconjugate(conj) {
                    PB => 1,
                    BL => 1,
                    DL => 1,
                    PL => 1,
                    PR => 1,
                    PD => 1,
                    F => -1,
                    DR => -1,
                    BR => -1,
                    R => -1,
                    L => -1,
                    U => -1,
                });

            // corner
            stickers.push(StickerSeed {
                layers,
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    bary(0.0, 1.0, 1.0, 0.0, 0.0),
                    bary(1.0, 1.0, 0.0, 0.0, 0.0),
                    bary(0.0, 1.0, 0.0, 0.0, 0.0),
                ],
            });
        }

        viewports.push(ViewportSeed {
            abstract_viewport,
            conjugate: conj,
            stickers,
            key_layers: key_layers.clone(),
        });
    }

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
