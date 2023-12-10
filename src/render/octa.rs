use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::{Basis, Sign};
use crate::puzzle::octa::OctaRay;
use crate::render::common::*;
use crate::NUMBER_KEYS;
use enum_map::enum_map;
use std::collections::HashMap;
use std::f32::consts::PI;

use three_d::*;

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

const SUPER_SIDE_RATIO: f32 = 0.6; // ratio of super sticker side to trapezoid short side
const CENTER_INRAD_RATIO: f32 = (1.0 + SUPER_SIDE_RATIO) / 3.0; // ratio between inradius of center and height of trapezoid
const FULL_SCALE: f32 = 1.0;

fn fto_circrad(order: i8) -> f32 {
    // assume the cut_width_on_axis is 1 for simplicity
    // it gets divided out anyway
    //3.0 * (order as f32 - 2.0 + 2.0 * CENTER_INRAD_RATIO)
    1.0
}

fn cut_depth(order: i8, cut: i8) -> f32 {
    // (order - 2 + 2 CIR) width = 2 circrad/3
    // => width = 2 circrad/(3 (order - 2 + 2 CIR))
    let half_width = fto_circrad(order) / (3.0 * (order as f32 - 2.0 + 2.0 * CENTER_INRAD_RATIO));
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

    let offsets: HashMap<i8, f32> = {
        let mut offsets = HashMap::new();
        let mut current_offset = 0.0;
        for n in 2..=order + 1 {
            offsets.insert(n, current_offset);
            current_offset += fto_circrad(n);
        }
        offsets
    };

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

    for n in (-order + 3..=order - 1).step_by(2) {
        for m in (-order + 1..=n - 2).step_by(2) {
            /*
            suppose inradius of octahedron is r
            range of cut depths will be between ± r/3
            if trapezoid height is h and local order is w, and CENTER_INRAD_RATIO is ρ,
            r/3 = 2ρ+h(w-2)  =>  r = 6ρ + 3h(w-2) ?????

            octahedron planes: x+y+z = r
            legal cuts: x+y+z = d where -r/3 < d < r/3

            */

            let s_order: i8 = (n - m) / 2 + 1;
            let circrad = fto_circrad(s_order) / fto_circrad(order) * FULL_SCALE;
            // side length of super sticker
            let sd = cut_width_on_axis(s_order) * FULL_SCALE * SUPER_SIDE_RATIO;

            let abstract_viewport = AbstractViewport {
                x: *offsets
                    .get(&((n + order + 1) / 2))
                    .expect("size should have been measured"),
                y: *offsets
                    .get(&((-m + order + 1) / 2))
                    .expect("size should have been measured"),
                //width: 1.0 * ((n + 1) as f32) / (order as f32),
                width: fto_circrad((n + order + 1) / 2),
                height: fto_circrad((-m + order + 1) / 2),
            };

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
                                cpu_mesh: polygon(vec![
                                    fv(1.0, 0.0, 0.0) * circrad,
                                    fv(0.0, 1.0, 0.0) * circrad,
                                    fv(0.0, 0.0, 1.0) * circrad,
                                ]),
                            });
                        } else if i == m && j == n {
                            // corner
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                cpu_mesh: polygon(vec![
                                    fv(cd(il + 1), 0.0, cd(jl - 1)) * circrad,
                                    fv(0.0, cd(il + 1), cd(jl - 1)) * circrad,
                                    fv(0.0, 0.0, 1.0) * circrad,
                                ]),
                            });
                        } else if i == n && j == m {
                            // another corner
                            // we don't need to render this one
                        } else if i == m && j == m {
                            // BU center
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                cpu_mesh: polygon(vec![
                                    fv(1.0 - 2.0 * cd(il + 1) + sd, cd(il + 1), cd(il + 1) - sd)
                                        * circrad,
                                    fv(1.0 - 2.0 * cd(il + 1) + sd, cd(il + 1) - sd, cd(il + 1))
                                        * circrad,
                                    fv(cd(il + 1) - sd, 1.0 - 2.0 * cd(il + 1) + sd, cd(il + 1))
                                        * circrad,
                                    fv(1.0, 1.0, 1.0) / 3.0 * circrad,
                                ]),
                            });
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BL),
                                cpu_mesh: polygon(vec![
                                    fv(1.0 - 2.0 * cd(il + 1) + sd, cd(il + 1) - sd, cd(il + 1))
                                        * circrad,
                                    fv(1.0 - 2.0 * cd(il + 1) + sd, cd(il + 1), cd(il + 1) - sd)
                                        * circrad,
                                    fv(1.0 - 2.0 * cd(il + 1), cd(il + 1), cd(il + 1)) * circrad,
                                ]),
                            });
                        } else if i == m {
                            // trapezoid (x-center) near L
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                cpu_mesh: polygon(vec![
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
                                ]),
                            });
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BL),
                                cpu_mesh: polygon(vec![
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
                                ]),
                            });
                        } else if j == m {
                            // trapezoid (x-center) near D
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                cpu_mesh: polygon(vec![
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
                                ]),
                            });
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BL),
                                cpu_mesh: polygon(vec![
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
                                ]),
                            });
                        } else if i + j == m + n {
                            // edge or wing
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                cpu_mesh: polygon(vec![
                                    fv(1.0 - cd(il - 1) - cd(jl - 1), cd(il - 1), cd(jl - 1))
                                        * circrad,
                                    fv(1.0 - cd(il + 1) - cd(jl - 1), cd(il + 1), cd(jl - 1))
                                        * circrad,
                                    fv(1.0 - cd(il - 1) - cd(jl + 1), cd(il - 1), cd(jl + 1))
                                        * circrad,
                                ]),
                            });
                        } else {
                            // rhombus
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BU),
                                cpu_mesh: polygon(vec![
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
                                ]),
                            });
                            stickers.push(StickerSeed {
                                layers,
                                face: fr(BU),
                                color: fr(BL),
                                cpu_mesh: polygon(vec![
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
                                ]),
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
        }
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
