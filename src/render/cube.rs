use crate::puzzle::common::*;
use crate::render::common::*;
use crate::CubeRay;
use crate::NUMBER_KEYS;
use enum_map::enum_map;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::iter;
use three_d::*;
use CubeRay::*;

const SUPER_START: f32 = 0.75;

impl ConcreteRaySystem for CubeRay {
    type Conjugate = ();

    fn axis_to_transform(&(ray, order): &(Self, i8), _conjugate: Self::Conjugate) -> Mat4 {
        match ray {
            U | D => Mat4::from_angle_z(Rad(PI / 2.0 * (order as f32))),
            R | L => Mat4::from_angle_x(Rad(PI / 2.0 * (order as f32))),
            F | B => Mat4::from_angle_y(Rad(-PI / 2.0 * (order as f32))),
        }
    }

    fn axis_to_vec(&self, _conjugate: Self::Conjugate) -> Vec3 {
        match self {
            U | D => Vec3::new(0.0, 0.0, 1.0),
            R | L => Vec3::new(1.0, 0.0, 0.0),
            F | B => Vec3::new(0.0, -1.0, 0.0),
        }
    }

    fn ray_to_color(&self) -> Srgba {
        match self {
            U => Srgba::WHITE,
            F => Srgba::RED,
            R => Srgba::BLUE,
            B => Srgba::new_opaque(255, 128, 0),
            L => Srgba::GREEN,
            D => Srgba::new_opaque(255, 255, 0),
        }
    }
}

/*pub fn weird_puzzle_seeds() -> Vec<ViewportSeed<CubeRay>> {
    let mut corner_mesh = CpuMesh::square();
    corner_mesh
        .transform(
            &(Mat4::from_translation(vec3(2.0 / 3.0, 2.0 / 3.0, 1.0))
                * Mat4::from_scale(1.0 / 3.0)),
        )
        .expect("the matrix should be invertible i made it");
    let mut edge_mesh = CpuMesh::square();
    edge_mesh
        .transform(
            &(Mat4::from_translation(vec3(2.0 / 3.0, 0.0, 1.0)) * Mat4::from_scale(1.0 / 3.0)),
        )
        .expect("the matrix should be invertible i made it");
    vec![ViewportSeed {
        abstract_viewport: AbstractViewport {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        conjugate: (),
        stickers: vec![
            StickerSeed {
                layers: enum_map! {U=>1,R=>1,B=>1,D=>-1,L=>-1,F=>-1,},
                face: U,
                color: U,
                cpu_mesh: corner_mesh,
            },
            StickerSeed {
                layers: enum_map! {U=>1,R=>1,B=>0,D=>-1,L=>-1,F=>0,},
                face: U,
                color: U,
                cpu_mesh: edge_mesh,
            },
            StickerSeed {
                layers: enum_map! {U=>1,R=>0,B=>0,D=>-1,L=>0,F=>0,},
                face: U,
                color: R,
                cpu_mesh: CpuMesh {
                    positions: Positions::F32(vec![
                        Vec3::new(0.2, -0.2, 1.0),
                        Vec3::new(1.0 / 3.0, -1.0 / 3.0, 1.0),
                        Vec3::new(1.0 / 3.0, 1.0 / 3.0, 1.0),
                        Vec3::new(0.2, 0.2, 1.0),
                    ]),
                    indices: Indices::U8(vec![0, 1, 2, 2, 3, 0]),
                    ..Default::default()
                },
            },
            StickerSeed {
                layers: enum_map! {U=>1,R=>0,B=>0,D=>-1,L=>0,F=>0,},
                face: U,
                color: U,
                cpu_mesh: CpuMesh {
                    positions: Positions::F32(vec![
                        Vec3::new(0.2, -0.2, 1.0),
                        Vec3::new(0.2, 0.2, 1.0),
                        Vec3::new(0.0, 0.0, 1.0),
                    ]),
                    indices: Indices::None,
                    ..Default::default()
                },
            },
        ],
    }]
}*/

pub fn nnn_seeds<'a>(order: i8) -> PuzzleSeed<CubeRay> {
    use CubeRay::*;

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
                default_layers: vec![vec![n, -n]],
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
}
