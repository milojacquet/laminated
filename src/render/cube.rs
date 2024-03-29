use crate::preferences::ConcretePuzzlePreferences;
use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::CubeRay;
use crate::puzzle::cube::{Basis, Sign};
use crate::render::common::*;
use crate::NUMBER_KEYS;
use enum_map::enum_map;
use std::collections::HashMap;
use std::f32::consts::PI;

use crate::preferences::Preferences;
use crate::util::{color, Vec3};

const SUPER_START: f32 = 0.75;

impl ConcreteRaySystem for CubeRay {
    type Conjugate = ();

    fn ray_to_vec(&self, _conjugate: Self::Conjugate) -> Vec3 {
        self.0.to_vec() * self.1.to_f32()
    }

    fn axis_to_vec(&self, _conjugate: Self::Conjugate) -> Vec3 {
        self.0.to_vec()
    }

    fn default_colors() -> enum_map::EnumMap<Self, color::Color> {
        enum_map! {
            Self(Basis::Y, Sign::Pos) => color::ORANGE,
            Self(Basis::Z, Sign::Pos) => color::WHITE,
            Self(Basis::X, Sign::Pos) => color::BLUE,
            Self(Basis::Z, Sign::Neg) => color::YELLOW,
            Self(Basis::X, Sign::Neg) => color::GREEN,
            Self(Basis::Y, Sign::Neg) => color::RED,
        }
    }

    fn ray_to_color(prefs: &Preferences) -> &enum_map::EnumMap<Self, color::Color> {
        &prefs.colors.cube
    }

    fn ray_to_color_mut(prefs: &mut Preferences) -> &mut enum_map::EnumMap<Self, color::Color> {
        &mut prefs.colors.cube
    }
}

pub fn nnn_seeds(order: i8, _prefs: &ConcretePuzzlePreferences) -> PuzzleSeed<CubeRay> {
    use crate::puzzle::cube::name::*;

    let mut current_width = 0.0;

    let grips: Vec<Vec<i8>> = (-order + 1..=order - 1)
        .step_by(2)
        .map(|k| vec![k, -k])
        .collect();

    let mut viewports: Vec<ViewportSeed<CubeRay>> = vec![];

    for n in (1 + (order & 1)..=order - 1).step_by(2) {
        // n = outer layer of the puzzle in this viewport
        // convenience
        let si = 1.0 / (n + 1) as f32;
        let cube_scale = (n as f32 + 1.0) / (order as f32);

        let abstract_viewport = AbstractViewport {
            x: current_width, //((n - 1) / 2) as f32,
            y: 0.0,
            //width: 1.0 * ((n + 1) as f32) / (order as f32),
            width: 0.7 * ((n + 1) as f32) / (order as f32) + 0.3,
            height: 1.0,
        };

        current_width += abstract_viewport.width;

        let mut stickers = vec![];

        for i in (2 - (n & 1)..=n).step_by(2) {
            for j in (2 - i..=i).step_by(2) {
                //dbg!(n, i, j);
                let layers = enum_map! {U=>n,R=>i,B=>j,D=>-n,L=>-i,F=>-j,};
                let cv = |x: f32, y: f32| {
                    Vec3::new((i as f32 + x) * si, (j as f32 + y) * si, 1.0) * cube_scale
                };

                if i == n {
                    // corner or edge
                    stickers.push(StickerSeed {
                        layers,
                        face: U,
                        color: U,
                        vertices: vec![cv(-1.0, -1.0), cv(1.0, -1.0), cv(1.0, 1.0), cv(-1.0, 1.0)],
                        options: Default::default(),
                    });
                } else if j == i {
                    // x-center
                    stickers.push(StickerSeed {
                        layers,
                        face: U,
                        color: U,
                        vertices: vec![
                            cv(-1.0, -1.0),
                            cv(SUPER_START, -1.0),
                            cv(SUPER_START, SUPER_START),
                            cv(-1.0, SUPER_START),
                        ],
                        options: Default::default(),
                    });
                    stickers.push(StickerSeed {
                        layers,
                        face: U,
                        color: R,
                        vertices: vec![
                            cv(SUPER_START, -1.0),
                            cv(1.0, -1.0),
                            cv(1.0, 1.0),
                            cv(SUPER_START, SUPER_START),
                        ],
                        options: Default::default(),
                    });
                    stickers.push(StickerSeed {
                        layers,
                        face: U,
                        color: B,
                        vertices: vec![
                            cv(-1.0, SUPER_START),
                            cv(SUPER_START, SUPER_START),
                            cv(1.0, 1.0),
                            cv(-1.0, 1.0),
                        ],
                        options: Default::default(),
                    });
                } else {
                    // t-center or oblique
                    stickers.push(StickerSeed {
                        layers,
                        face: U,
                        color: U,
                        vertices: vec![
                            cv(-1.0, -1.0),
                            cv(SUPER_START, -1.0),
                            cv(SUPER_START, 1.0),
                            cv(-1.0, 1.0),
                        ],
                        options: Default::default(),
                    });
                    stickers.push(StickerSeed {
                        layers,
                        face: U,
                        color: R,
                        vertices: vec![
                            cv(SUPER_START, -1.0),
                            cv(1.0, -1.0),
                            cv(1.0, 1.0),
                            cv(SUPER_START, 1.0),
                        ],
                        options: Default::default(),
                    });
                }
            }
        }
        if order & 1 == 1 {
            let layers = enum_map! {U=>n,R=>0,B=>0,D=>-n,L=>0,F=>0,};
            stickers.push(StickerSeed {
                layers,
                face: U,
                color: U,
                vertices: vec![
                    Vec3::new(0.0, 0.0, 1.0) * cube_scale,
                    Vec3::new(SUPER_START * si, -SUPER_START * si, 1.0) * cube_scale,
                    Vec3::new(SUPER_START * si, SUPER_START * si, 1.0) * cube_scale,
                ],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers,
                face: U,
                color: R,
                vertices: vec![
                    Vec3::new(SUPER_START * si, -SUPER_START * si, 1.0) * cube_scale,
                    Vec3::new(si, -si, 1.0) * cube_scale,
                    Vec3::new(si, si, 1.0) * cube_scale,
                    Vec3::new(SUPER_START * si, SUPER_START * si, 1.0) * cube_scale,
                ],
                options: Default::default(),
            });
        }

        let key_layers = vec![
            HashMap::from_iter((-n..=n).rev().step_by(2).map(|nn| {
                let layer = (n - nn) / 2;
                (NUMBER_KEYS[layer as usize], vec![nn, -nn])
            })),
            HashMap::from_iter((-n..=n).step_by(2).map(|nn| {
                let layer = (n + nn) / 2;
                (NUMBER_KEYS[layer as usize], vec![nn, -nn])
            })),
        ];

        viewports.push(ViewportSeed {
            abstract_viewport,
            conjugate: (),
            stickers,
            key_layers,
        });
    }

    let key_layers = vec![
        HashMap::from_iter(NUMBER_KEYS.into_iter().zip(grips.iter().rev().cloned())),
        HashMap::from_iter(NUMBER_KEYS.into_iter().zip(grips.iter().cloned())),
    ];

    PuzzleSeed {
        grips,
        viewports,
        key_layers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::common::concrete_ray_system_tests::validate_concrete_ray_system;

    #[test]
    fn validate_concrete_ray_system_cube() {
        validate_concrete_ray_system::<CubeRay>()
    }
}
