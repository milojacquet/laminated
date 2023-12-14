use crate::preferences::Preferences;
use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::{Basis, Sign};
use crate::puzzle::octa::OctaRay;
use crate::render::common::*;
use crate::NUMBER_KEYS;
use enum_map::enum_map;
use std::collections::HashMap;
use std::f32::consts::PI;

use crate::util::{color, Mat4, Vec3};
use cgmath::{InnerSpace, Rad};

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

    fn ray_to_color(prefs: &Preferences) -> enum_map::EnumMap<Self, color::Color> {
        prefs.colors.octa
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
            vertices: vec![
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 1.0, 1.0) / 3.0,
            ],
        });

        stickers.push(StickerSeed {
            layers: enum_map::EnumMap::from_fn(|_ray| 0),
            face: r_U,
            color: r_U,
            vertices: vec![
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, -1.0, 1.0) / 3.0,
            ],
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

const SUPER_SIDE_RATIO: f32 = 0.6; // ratio of super sticker side to trapezoid short side
const CENTER_INRAD_RATIO: f32 = (1.0 + SUPER_SIDE_RATIO) / 3.0; // ratio between inradius of center and height of trapezoid
const FULL_SCALE: f32 = 1.8;

fn fto_inrad(order: i8) -> f32 {
    // assume the cut_width_on_axis is 1 for simplicity
    // it gets divided out anyway
    3.0 * (order as f32 - 2.0 + 2.0 * CENTER_INRAD_RATIO)
    //1.0
}

fn cut_depth(order: i8, cut: i8) -> f32 {
    // (order - 2 + 2 CIR) width = 2 circrad/3
    // => width = 2 circrad/(3 (order - 2 + 2 CIR))
    let half_width = fto_inrad(order) / (3.0 * (order as f32 - 2.0 + 2.0 * CENTER_INRAD_RATIO));
    return half_width * cut as f32;
}

fn cut_width_on_axis(order: i8) -> f32 {
    1.0 / (3.0 * (order as f32 - 2.0 + 2.0 * CENTER_INRAD_RATIO))
}

/// more convenient version of cut_depth that ranges from 0 to 1
fn cut_depth_on_axis(order: i8, cut: i8) -> f32 {
    // (order - 2 + 2 CIR) width = 1/3
    // => width = 1/(3 (order - 2 + 2 CIR))
    let half_width = cut_width_on_axis(order) / 2.0;
    return 0.5 + half_width * cut as f32;
}

pub fn fto_seeds<'a>(order: i8) -> PuzzleSeed<OctaRay> {
    use crate::puzzle::octa::name::*;

    let grips: Vec<Vec<i8>> = (-order + 1..=order - 1)
        .step_by(2)
        .map(|k| vec![k, -k])
        .collect();

    /*
    The laminated [order]-layer FTO corresponds to a subgroup of the n^4 hypercube.
    The 4d coordinates of a FTO piece are the layer indices of the axis heads.
    Each viewport contains pieces whose maximum coordinate is n,
    and whose minimum coordinate is m.
    */

    let mut viewports: Vec<ViewportSeed<OctaRay>> = vec![];

    // for n in (-order + 3..=order - 1).step_by(2) {
    // for m in (-order + 1..=n - 2).step_by(2) {

    let mut current_x = 0.0;

    for n_plus_m in (-2 * order + 4..=2 * order - 4).step_by(2) {
        let max_s_order = order - n_plus_m.abs() / 2;
        let current_width = fto_inrad(max_s_order) / fto_inrad(order);
        let mut current_y = 0.0;

        for n_minus_m in
            (((n_plus_m & 2) ^ ((order & 1) * 2)) + 2..=order * 2 - 2 - n_plus_m.abs()).step_by(4)
        {
            let n = (n_plus_m + n_minus_m) / 2;
            let m = (n_plus_m - n_minus_m) / 2;

            /*
            suppose inradius of octahedron is r
            range of cut depths will be between ± r/3
            if trapezoid height is h and local order is w, and CENTER_INRAD_RATIO is ρ,
            r/3 = 2ρ+h(w-2)  =>  r = 6ρ + 3h(w-2) ?????

            octahedron planes: x+y+z = r
            legal cuts: x+y+z = d where -r/3 < d < r/3

            */

            let s_order: i8 = (n - m) / 2 + 1;
            let circrad = fto_inrad(s_order) / fto_inrad(order) * FULL_SCALE;
            // side length of super sticker
            let sd = cut_width_on_axis(s_order) * SUPER_SIDE_RATIO;

            let current_height = fto_inrad(s_order) / fto_inrad(order);

            let abstract_viewport = AbstractViewport {
                x: current_x,
                y: current_y,
                //width: 1.0 * ((n + 1) as f32) / (order as f32),
                width: current_width,
                height: current_height,
            };

            //println!("{m} {n} {abstract_viewport:?}");

            let cd = |l| cut_depth_on_axis(s_order, l);

            let mut stickers = vec![];

            for i in (m..=n).step_by(2) {
                for j in (m..=m + n - i).step_by(2) {
                    let flip_ray = |flip: bool, ray: OctaRay| {
                        if flip {
                            OctaRay(-ray.0, ray.2, ray.1)
                        } else {
                            ray
                        }
                    };
                    let flip_vec = |flip: bool, x: f32, y, z| {
                        if flip {
                            Vec3::new(-x, z, y)
                        } else {
                            Vec3::new(x, y, z)
                        }
                    };

                    for flip in crate::util::enum_iter::<bool>() {
                        let fr = |ray| flip_ray(flip, ray);
                        let fv = |x, y, z| flip_vec(flip, x, y, z);

                        let layers: enum_map::EnumMap<OctaRay, i8>;
                        //layers = enum_map! {BU => n,R => m,L => j,D => i,F => -n,BL => -m,BR => -j,U => -i,};

                        // None: not extending
                        // Some(true): extending this face
                        // Some(false): extending the other face
                        let extend_opt: Option<bool>;
                        if n_plus_m.abs() != 2 * order - 4 || order == 2 {
                            extend_opt = None;
                        } else {
                            extend_opt = Some((n_plus_m == -2 * order + 4) ^ flip);
                        }

                        if flip {
                            layers = enum_map! {BU => n,R => m,L => n+m-j,D => n+m-i,F => -n,BL => -m,BR => -n-m+j,U => -n-m+i,};
                        } else {
                            layers = enum_map! {BU => n,R => m,L => j,D => i,F => -n,BL => -m,BR => -j,U => -i,};
                        }
                        /*layers = enum_map::EnumMap::from_fn(|ray| match fr(ray){
                            BU => n,R => m,L => j,D => i,F => -n,BL => -m,BR => -j,U => -i,
                        }*if flip{1} else {1}/*+if ray.tet_sign()==Sign::Neg{-n-m}else{n+m}*/);*/

                        // all pieces are on the BU face (or the BL face in the second invocation)

                        // local layer indices: these pretend that m = -s_order+1 and n = s_order-1
                        let il = i + s_order - 1 - n;
                        let jl = j + s_order - 1 - n;

                        if s_order == 1 {
                            // core/anticore
                            // this doesn't need to be rendered because its orientation is determined by the other pieces
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                vertices: vec![
                                    fv(1.0, 0.0, 0.0) * circrad,
                                    fv(0.0, 1.0, 0.0) * circrad,
                                    fv(0.0, 0.0, 1.0) * circrad,
                                ],
                            });
                        } else if i == m && j == n {
                            // corner
                            match extend_opt {
                                None => {
                                    stickers.push(StickerSeed {
                                        layers,
                                        face: fr(BU),
                                        color: fr(BU),
                                        vertices: vec![
                                            fv(cd(il + 1), 0.0, cd(jl - 1)) * circrad,
                                            fv(0.0, cd(il + 1), cd(jl - 1)) * circrad,
                                            fv(0.0, 0.0, 1.0) * circrad,
                                        ],
                                    });
                                }
                                Some(false) => {
                                    stickers.push(StickerSeed {
                                        layers,
                                        face: fr(BU),
                                        color: fr(BU),
                                        vertices: vec![
                                            fv(0.0 + sd, 0.0 - sd, 1.0) * circrad,
                                            fv(0.5, 0.0 - sd, 0.5 + sd) * circrad,
                                            fv(0.5, 0.0, 0.5) * circrad,
                                            fv(0.0, 0.5, 0.5) * circrad,
                                            fv(0.0 - sd, 0.5, 0.5 + sd) * circrad,
                                            fv(0.0 - sd, 0.0 + sd, 1.0) * circrad,
                                        ],
                                    });
                                }
                                Some(true) => {
                                    stickers.push(StickerSeed {
                                        layers,
                                        face: fr(BU),
                                        color: fr(BU),
                                        vertices: vec![
                                            fv(0.5, 0.0 + sd, 0.5 + sd) * circrad,
                                            fv(0.0 + sd, 0.5, 0.5 + sd) * circrad,
                                            fv(0.0 + sd, 0.0 + sd, 1.0) * circrad,
                                        ],
                                    });
                                }
                            }
                        } else if i == n && j == m {
                            // another corner
                            // we don't need to render this one
                        } else if i == m && j == m {
                            // BU center
                            if extend_opt == Some(true) {
                                // cd(il+1) == 0.5

                                stickers.push(StickerSeed {
                                    layers,
                                    face: fr(BU),
                                    color: fr(BU),
                                    vertices: vec![
                                        fv(0.0 + sd, 0.5 + sd, 0.5) * circrad,
                                        fv(0.0 + sd, 0.5, 0.5 + sd) * circrad,
                                        fv(0.5, 0.0 + sd, 0.5 + sd) * circrad,
                                        fv(1.0 + 2.0 * sd, 1.0 + 2.0 * sd, 1.0 + 2.0 * sd) / 3.0
                                            * circrad,
                                    ],
                                });
                                stickers.push(StickerSeed {
                                    layers,
                                    face: fr(BL), // it's bent
                                    color: fr(BL),
                                    vertices: vec![
                                        fv(sd, 0.5, 0.5 + sd) * circrad,
                                        fv(sd, 0.5 + sd, 0.5) * circrad,
                                        fv(0.0, 0.5, 0.5) * circrad,
                                    ],
                                });
                            } else {
                                stickers.push(StickerSeed {
                                    layers,
                                    face: fr(BU),
                                    color: fr(BU),
                                    vertices: vec![
                                        fv(
                                            1.0 - 2.0 * cd(il + 1) + sd,
                                            cd(il + 1),
                                            cd(il + 1) - sd,
                                        ) * circrad,
                                        fv(
                                            1.0 - 2.0 * cd(il + 1) + sd,
                                            cd(il + 1) - sd,
                                            cd(il + 1),
                                        ) * circrad,
                                        fv(
                                            cd(il + 1) - sd,
                                            1.0 - 2.0 * cd(il + 1) + sd,
                                            cd(il + 1),
                                        ) * circrad,
                                        fv(1.0, 1.0, 1.0) / 3.0 * circrad,
                                    ],
                                });
                                stickers.push(StickerSeed {
                                    layers,
                                    face: fr(BU),
                                    color: fr(BL),
                                    vertices: vec![
                                        fv(
                                            1.0 - 2.0 * cd(il + 1) + sd,
                                            cd(il + 1) - sd,
                                            cd(il + 1),
                                        ) * circrad,
                                        fv(
                                            1.0 - 2.0 * cd(il + 1) + sd,
                                            cd(il + 1),
                                            cd(il + 1) - sd,
                                        ) * circrad,
                                        fv(1.0 - 2.0 * cd(il + 1), cd(il + 1), cd(il + 1))
                                            * circrad,
                                    ],
                                });
                            }
                        } else if i == m {
                            // trapezoid (x-center) near L
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                vertices: vec![
                                    fv(1.0 - cd(il + 1) - cd(jl - 1), cd(il + 1), cd(jl - 1))
                                        * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1),
                                        cd(jl + 1) - sd,
                                    ) * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1) - sd,
                                        cd(jl + 1),
                                    ) * circrad,
                                    fv(
                                        (1.0 - cd(jl + 1)) / 2.0,
                                        (1.0 - cd(jl + 1)) / 2.0,
                                        cd(jl + 1),
                                    ) * circrad,
                                    fv(
                                        (1.0 - cd(jl - 1)) / 2.0,
                                        (1.0 - cd(jl - 1)) / 2.0,
                                        cd(jl - 1),
                                    ) * circrad,
                                ],
                            });
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BL),
                                vertices: vec![
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1) - sd,
                                        cd(jl + 1),
                                    ) * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1),
                                        cd(jl + 1) - sd,
                                    ) * circrad,
                                    fv(1.0 - cd(il + 1) - cd(jl + 1), cd(il + 1), cd(jl + 1))
                                        * circrad,
                                ],
                            });
                        } else if j == m {
                            // trapezoid (x-center) near D
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                vertices: vec![
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1),
                                        cd(jl + 1) - sd,
                                    ) * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1) - sd,
                                        cd(jl + 1),
                                    ) * circrad,
                                    fv(1.0 - cd(il - 1) - cd(jl + 1), cd(il - 1), cd(jl + 1))
                                        * circrad,
                                    fv(
                                        (1.0 - cd(il - 1)) / 2.0,
                                        cd(il - 1),
                                        (1.0 - cd(il - 1)) / 2.0,
                                    ) * circrad,
                                    fv(
                                        (1.0 - cd(il + 1)) / 2.0,
                                        cd(il + 1),
                                        (1.0 - cd(il + 1)) / 2.0,
                                    ) * circrad,
                                ],
                            });
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BL),
                                vertices: vec![
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1) - sd,
                                        cd(jl + 1),
                                    ) * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1),
                                        cd(jl + 1) - sd,
                                    ) * circrad,
                                    fv(1.0 - cd(il + 1) - cd(jl + 1), cd(il + 1), cd(jl + 1))
                                        * circrad,
                                ],
                            });
                        } else if i + j == m + n {
                            // edge or wing
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                vertices: vec![
                                    fv(1.0 - cd(il - 1) - cd(jl - 1), cd(il - 1), cd(jl - 1))
                                        * circrad,
                                    fv(1.0 - cd(il + 1) - cd(jl - 1), cd(il + 1), cd(jl - 1))
                                        * circrad,
                                    fv(1.0 - cd(il - 1) - cd(jl + 1), cd(il - 1), cd(jl + 1))
                                        * circrad,
                                ],
                            });
                        } else {
                            // rhombus
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                vertices: vec![
                                    fv(1.0 - cd(il - 1) - cd(jl - 1), cd(il - 1), cd(jl - 1))
                                        * circrad,
                                    fv(1.0 - cd(il + 1) - cd(jl - 1), cd(il + 1), cd(jl - 1))
                                        * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1),
                                        cd(jl + 1) - sd,
                                    ) * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1) - sd,
                                        cd(jl + 1),
                                    ) * circrad,
                                    fv(1.0 - cd(il - 1) - cd(jl + 1), cd(il - 1), cd(jl + 1))
                                        * circrad,
                                ],
                            });
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BL),
                                vertices: vec![
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1) - sd,
                                        cd(jl + 1),
                                    ) * circrad,
                                    fv(
                                        1.0 - cd(il + 1) - cd(jl + 1) + sd,
                                        cd(il + 1),
                                        cd(jl + 1) - sd,
                                    ) * circrad,
                                    fv(1.0 - cd(il + 1) - cd(jl + 1), cd(il + 1), cd(jl + 1))
                                        * circrad,
                                ],
                            });
                        }
                    }
                }
            }

            viewports.push(ViewportSeed {
                abstract_viewport,
                conjugate: (),
                stickers,
                default_layers: vec![vec![n, -n], vec![m, -m]],
            });

            current_y += current_height;
        }

        current_x += current_width;
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
