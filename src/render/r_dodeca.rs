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

impl RDodecaRay {
    /// Maps self from conjugate to BinaryConjugate::Id.
    /// This is an abstract inverse of ray_to_vec.
    fn unconjugate(&self, conjugate: <Self as ConcreteRaySystem>::Conjugate) -> Self {
        match conjugate {
            BinaryConjugate::Id => *self,
            BinaryConjugate::Conj => Self(self.0, self.1, -self.2),
        }
    }

    /// Maps self from conjugate to BinaryConjugate::Id.
    /// This is an abstract version of ray_to_vec.
    fn conjugate(&self, conjugate: <Self as ConcreteRaySystem>::Conjugate) -> Self {
        match conjugate {
            BinaryConjugate::Id => *self,
            BinaryConjugate::Conj => Self(self.0, self.1, -self.2),
        }
    }
}

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
        // in the conjugate, (a, ±₁, ±₂) -> - ±₁a⁺ + ∓₂a⁺⁺
        let sign = match conjugate {
            BinaryConjugate::Id => 1.0,
            BinaryConjugate::Conj => -1.0,
        };

        ((self.0 + BasisDiff::D1).to_vec() * self.1.to_f32()
            + sign * (self.0 + BasisDiff::D2).to_vec() * self.2.to_f32())
        .normalize()
    }

    fn default_colors() -> enum_map::EnumMap<Self, color::Color> {
        use crate::puzzle::dodeca::{name, DodecaRay};

        let default_dodeca = DodecaRay::default_colors();
        enum_map::EnumMap::from_fn(|ray: Self| {
            default_dodeca[DodecaRay(ray.0, ray.1, ray.2).turn((name::U, 2))]
        })
    }

    fn ray_to_color(prefs: &Preferences) -> &enum_map::EnumMap<Self, color::Color> {
        &prefs.colors.r_dodeca
    }

    fn ray_to_color_mut(prefs: &mut Preferences) -> &mut enum_map::EnumMap<Self, color::Color> {
        &mut prefs.colors.r_dodeca
    }
}

const CORE_SIZE: f32 = 0.4;

pub fn little_chop_seeds(_prefs: &ConcretePuzzlePreferences) -> PuzzleSeed<RDodecaRay> {
    use crate::puzzle::r_dodeca::name::*;

    let grips: Vec<Vec<i8>> = vec![vec![-1, 1], vec![1, -1]];

    let mut viewports: Vec<ViewportSeed<RDodecaRay>> = vec![];

    let key_layers = vec![
        HashMap::from([(NUMBER_KEYS[0], vec![1, -1]), (NUMBER_KEYS[1], vec![-1, 1])]),
        HashMap::from([(NUMBER_KEYS[0], vec![-1, 1]), (NUMBER_KEYS[1], vec![1, -1])]),
    ];

    for conj in enum_iter::<BinaryConjugate>() {
        /*let make_grips = |grips: Vec<RDodecaRay>| {
            enum_map::EnumMap::from_fn(|ray: RDodecaRay| {
                if grips.contains(&ray.unconjugate(conj)) {
                    1
                } else if grips.contains(&ray.unconjugate(conj).opposite()) {
                    -1
                } else {
                    0
                }
            })
        };*/

        {
            let abstract_viewport = AbstractViewport {
                x: -CORE_SIZE,
                y: match conj {
                    BinaryConjugate::Id => 2.0 - CORE_SIZE,
                    BinaryConjugate::Conj => 0.0,
                },
                width: CORE_SIZE,
                height: CORE_SIZE,
            };

            let layers = enum_map! {FU=>1,BU=>1,UR=>1,UL=>1,RF=>1,LF=>1,BD=>-1,FD=>-1,DL=>-1,DR=>-1,LB=>-1,RB=>-1,};
            viewports.push(ViewportSeed {
                abstract_viewport,
                conjugate: conj,
                stickers: vec![StickerSeed {
                    layers,
                    face: RB.conjugate(conj),
                    color: RB.conjugate(conj),
                    vertices: vec![
                        Vec3::new(1.0, 0.0, 0.0) * CORE_SIZE,
                        Vec3::new(0.5, 0.5, -0.5) * CORE_SIZE,
                        Vec3::new(0.5, 0.5, 0.5) * CORE_SIZE,
                    ],
                    options: StickerOptions {
                        core: true,
                        parity: conj == BinaryConjugate::Conj,
                        ..Default::default()
                    },
                }],
                key_layers: vec![HashMap::new(), HashMap::new()],
            });
        }

        /*let abstract_viewport = AbstractViewport {
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
                options: Default::default(),
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
                options: Default::default(),
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
                options: Default::default(),
            });
        }

        viewports.push(ViewportSeed {
            abstract_viewport,
            conjugate: conj,
            stickers,
            key_layers: key_layers.clone(),
        });
        */
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
    fn validate_concrete_ray_system_r_dodeca() {
        validate_concrete_ray_system::<RDodecaRay>()
    }
}
