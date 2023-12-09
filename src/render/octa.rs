use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::{Basis, Sign};
use crate::puzzle::octa::OctaRay;
use crate::render::common::*;
use crate::NUMBER_KEYS;
use enum_map::enum_map;
use std::collections::HashMap;
use std::f32::consts::PI;

use three_d::*;

const SUPER_START: f32 = 0.75;

impl ConcreteRaySystem for OctaRay {
    type Conjugate = ();

    fn axis_to_transform((ray, order): (Self, i8), conjugate: Self::Conjugate) -> Mat4 {
        Mat4::from_axis_angle(
            ray.axis_to_vec(conjugate),
            Rad(order as f32 * 2.0 * PI / 3.0),
        )
    }

    fn ray_to_vec(&self, _conjugate: Self::Conjugate) -> Vec3 {
        (Basis::X.to_vec() * self.0.to_f32()
            + Basis::Y.to_vec() * self.1.to_f32()
            + Basis::Z.to_vec() * self.2.to_f32())
        .normalize()
    }

    fn ray_to_color(&self) -> Srgba {
        match self {
            OctaRay(Sign::Pos, Sign::Neg, Sign::Pos) => Srgba::WHITE,
            OctaRay(Sign::Pos, Sign::Neg, Sign::Neg) => Srgba::GREEN,
            OctaRay(Sign::Neg, Sign::Neg, Sign::Pos) => Srgba::RED,
            OctaRay(Sign::Neg, Sign::Neg, Sign::Neg) => Srgba::new_opaque(0, 128, 0),
            OctaRay(Sign::Pos, Sign::Pos, Sign::Pos) => Srgba::BLUE,
            OctaRay(Sign::Pos, Sign::Pos, Sign::Neg) => Srgba::new_opaque(255, 128, 0),
            OctaRay(Sign::Neg, Sign::Pos, Sign::Pos) => Srgba::new_opaque(128, 0, 255),
            OctaRay(Sign::Neg, Sign::Pos, Sign::Neg) => Srgba::new_opaque(255, 255, 0),
        }
    }
}

pub fn core_seeds() -> PuzzleSeed<OctaRay> {
    #[allow(non_snake_case)]
    let r_BU = OctaRay::from_name("BU").unwrap();
    #[allow(non_snake_case)]
    let r_U = OctaRay::from_name("U").unwrap();
    let grips: Vec<Vec<i8>> = vec![vec![0, 0]];

    let viewports = vec![{
        let abstract_viewport = AbstractViewport {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };

        let mut stickers = vec![];

        stickers.push(StickerSeed {
            layers: enum_map::EnumMap::from_fn(|_ray| 0),
            face: r_BU,
            color: r_BU,
            cpu_mesh: polygon(vec![
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 1.0, 1.0) / 3.0,
            ]),
        });

        stickers.push(StickerSeed {
            layers: enum_map::EnumMap::from_fn(|_ray| 0),
            face: r_U,
            color: r_U,
            cpu_mesh: polygon(vec![
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, -1.0, 1.0) / 3.0,
            ]),
        });

        ViewportSeed {
            abstract_viewport,
            conjugate: (),
            stickers,
            default_layers: vec![vec![0, 0]],
        }
    }];

    let key_layers = vec![HashMap::from_iter(vec![]), HashMap::from_iter(vec![])];

    PuzzleSeed {
        grips,
        viewports,
        key_layers,
    }
}

/*
pub fn nnn_seeds<'a>(order: i8) -> PuzzleSeed<CubeRay> {
    use crate::puzzle::cube::name::*;

    let mut current_width = 0.0;

    let grips: Vec<Vec<i8>> = (-order + 1..=order - 1)
        .step_by(2)
        .map(|k| vec![k, -k])
        .collect();

    let viewports = (1 + (order & 1)..=order - 1)
        .step_by(2)
        .map(|n| {
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
                            cpu_mesh: polygon(vec![
                                cv(-1.0, -1.0),
                                cv(1.0, -1.0),
                                cv(1.0, 1.0),
                                cv(-1.0, 1.0),
                            ]),
                        });
                    } else if j == i {
                        // x-center
                        stickers.push(StickerSeed {
                            layers,
                            face: U,
                            color: U,
                            cpu_mesh: polygon(vec![
                                cv(-1.0, -1.0),
                                cv(SUPER_START, -1.0),
                                cv(SUPER_START, SUPER_START),
                                cv(-1.0, SUPER_START),
                            ]),
                        });
                        stickers.push(StickerSeed {
                            layers,
                            face: U,
                            color: R,
                            cpu_mesh: polygon(vec![
                                cv(SUPER_START, -1.0),
                                cv(1.0, -1.0),
                                cv(1.0, 1.0),
                                cv(SUPER_START, SUPER_START),
                            ]),
                        });
                        stickers.push(StickerSeed {
                            layers,
                            face: U,
                            color: B,
                            cpu_mesh: polygon(vec![
                                cv(-1.0, SUPER_START),
                                cv(SUPER_START, SUPER_START),
                                cv(1.0, 1.0),
                                cv(-1.0, 1.0),
                            ]),
                        });
                    } else {
                        // t-center or oblique
                        stickers.push(StickerSeed {
                            layers,
                            face: U,
                            color: U,
                            cpu_mesh: polygon(vec![
                                cv(-1.0, -1.0),
                                cv(SUPER_START, -1.0),
                                cv(SUPER_START, 1.0),
                                cv(-1.0, 1.0),
                            ]),
                        });
                        stickers.push(StickerSeed {
                            layers,
                            face: U,
                            color: R,
                            cpu_mesh: polygon(vec![
                                cv(SUPER_START, -1.0),
                                cv(1.0, -1.0),
                                cv(1.0, 1.0),
                                cv(SUPER_START, 1.0),
                            ]),
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
                    cpu_mesh: polygon(vec![
                        Vec3::new(0.0, 0.0, 1.0) * cube_scale,
                        Vec3::new(SUPER_START * si, -SUPER_START * si, 1.0) * cube_scale,
                        Vec3::new(SUPER_START * si, SUPER_START * si, 1.0) * cube_scale,
                    ]),
                });
                stickers.push(StickerSeed {
                    layers,
                    face: U,
                    color: R,
                    cpu_mesh: polygon(vec![
                        Vec3::new(SUPER_START * si, -SUPER_START * si, 1.0) * cube_scale,
                        Vec3::new(si, -si, 1.0) * cube_scale,
                        Vec3::new(si, si, 1.0) * cube_scale,
                        Vec3::new(SUPER_START * si, SUPER_START * si, 1.0) * cube_scale,
                    ]),
                });
            }

            ViewportSeed {
                abstract_viewport,
                conjugate: (),
                stickers,
                default_layers: vec![vec![n, -n], vec![-n, n]],
            }
        })
        .collect();

    let key_layers = vec![
        HashMap::from_iter(NUMBER_KEYS.into_iter().zip(grips.iter().rev().cloned())),
        HashMap::from_iter(NUMBER_KEYS.into_iter().zip(grips.iter().cloned())),
    ];

    PuzzleSeed {
        grips,
        viewports,
        key_layers,
    }
}*/
