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

    fn opposite(&self) -> Self {
        Self(self.0, -self.1, -self.2)
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
const CORE_SIZE: f32 = 0.4;

fn turn(vec: Vec3) -> Vec3 {
    use crate::puzzle::dodeca::name::PB;
    let mat = DodecaRay::turn_to_transform((PB, 1), BinaryConjugate::Id);
    (mat * vec.extend(1.0)).truncate()
}

// c1-PD-c2-PL-c3-BL-c4-BR-c5-PR-
fn bary_nosc(scale: f32, c1: f32, c2: f32, c3: f32, c4: f32, c5: f32) -> Vec3 {
    let v1 = Vec3::new((3.0 * SQ5 - 5.0) / 2.0, SQ5, 0.0);
    let v2 = turn(v1);
    let v3 = turn(v2);
    let v4 = turn(v3);
    let v5 = turn(v4);
    (c1 * v1 + c2 * v2 + c3 * v3 + c4 * v4 + c5 * v5) * scale
}

fn bary(scale: f32, c1: f32, c2: f32, c3: f32, c4: f32, c5: f32) -> Vec3 {
    bary_nosc(scale, c1, c2, c3, c4, c5) / (c1 + c2 + c3 + c4 + c5)
}

pub fn pentultimate_seeds(_prefs: &ConcretePuzzlePreferences) -> PuzzleSeed<DodecaRay> {
    use crate::puzzle::dodeca::name::*;

    let grips: Vec<Vec<i8>> = vec![vec![-1, 1], vec![1, -1]];

    let mut viewports: Vec<ViewportSeed<DodecaRay>> = vec![];

    let key_layers = vec![
        HashMap::from([(NUMBER_KEYS[0], vec![1, -1]), (NUMBER_KEYS[1], vec![-1, 1])]),
        HashMap::from([(NUMBER_KEYS[0], vec![-1, 1]), (NUMBER_KEYS[1], vec![1, -1])]),
    ];

    let bary = |a, b, c, d, e| bary(SCALE_CIRCUMRAD, a, b, c, d, e);

    for conj in enum_iter::<BinaryConjugate>() {
        let x = match conj {
            BinaryConjugate::Id => 0.0,
            BinaryConjugate::Conj => 1.0,
        };

        // core guide
        {
            let layers = enum_map! {U=>1,R=>1,F=>1,L=>1,BL=>1,BR=>1,DL=>-1,DR=>-1,PL=>-1,PB=>-1,PR=>-1,PD=>-1};
            viewports.push(ViewportSeed {
                abstract_viewport: AbstractViewport {
                    x: x + 0.5 - 0.5 * CORE_SIZE,
                    y: -CORE_SIZE * 0.7, // yeah it intersects. what of it?
                    width: CORE_SIZE,
                    height: CORE_SIZE,
                },
                conjugate: conj,
                stickers: vec![StickerSeed {
                    layers,
                    face: PB.conjugate(conj),
                    color: PB.conjugate(conj),
                    vertices: vec![
                        bary(1.0, 0.0, 0.0, 0.0, 0.0) * CORE_SIZE * 0.8,
                        bary(0.0, 1.0, 0.0, 0.0, 0.0) * CORE_SIZE * 0.8,
                        bary(1.0, 1.0, 1.0, 1.0, 1.0) * CORE_SIZE * 0.8,
                    ],
                    options: StickerOptions {
                        core: true,
                        ..Default::default()
                    },
                }],
                key_layers: vec![HashMap::new(), HashMap::new()],
            });
        }

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
    }

    PuzzleSeed {
        grips,
        viewports,
        key_layers: key_layers.clone(),
    }
}

const MEGA_SCALE: f32 = 0.8;
const MEGA_DEPTH: f32 = 0.4;
const MEGA_SUPER: f32 = 1.2;

const MPENT_SCALE: f32 = 0.9;
const MPENT_DEPTH: f32 = 0.3;
const MPENT_SUPER: f32 = 0.1;

//const SLICE_SCALE:f32=1.0;
const SLICE_INNER_DEPTH: f32 = (25.0 - SQ5) / 62.0;
const SLICE_OUTER_DEPTH: f32 = (1.0 - 2.0 * SLICE_INNER_DEPTH) / (PHI + 1.0);
const SLICE_SUPER: f32 = 0.03;

pub fn mega_seeds(_prefs: &ConcretePuzzlePreferences) -> PuzzleSeed<DodecaRay> {
    use crate::puzzle::dodeca::name::*;

    let grips: Vec<Vec<i8>> = vec![vec![-2, 2], vec![0, 0], vec![2, -2]];

    let mut viewports: Vec<ViewportSeed<DodecaRay>> = vec![];

    let key_layers = vec![
        HashMap::from([
            (NUMBER_KEYS[0], vec![2, -2]),
            (NUMBER_KEYS[1], vec![0, 0]),
            (NUMBER_KEYS[2], vec![-2, 2]),
        ]),
        HashMap::from([
            (NUMBER_KEYS[0], vec![-2, 2]),
            (NUMBER_KEYS[1], vec![0, 0]),
            (NUMBER_KEYS[2], vec![2, -2]),
        ]),
    ];

    for conj in enum_iter::<BinaryConjugate>() {
        let y_off = match conj {
            BinaryConjugate::Id => 0.0,
            BinaryConjugate::Conj => 1.0,
        };

        let make_grips = |grips: Vec<DodecaRay>| {
            enum_map::EnumMap::from_fn(|ray: DodecaRay| {
                if grips.contains(&ray.unconjugate(conj)) {
                    2
                } else if grips.contains(&ray.unconjugate(conj).opposite()) {
                    -2
                } else {
                    0
                }
            })
        };

        // megaminx
        {
            let abstract_viewport = AbstractViewport {
                x: 0.0,
                y: (1.0 - MEGA_SCALE) / 2.0 - y_off,
                width: MEGA_SCALE,
                height: MEGA_SCALE,
            };

            let bary = |a, b, c, d, e| bary(SCALE_CIRCUMRAD * MEGA_SCALE, a, b, c, d, e);

            let mut stickers: Vec<StickerSeed<DodecaRay>> = vec![];

            // center
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    bary(
                        MEGA_DEPTH * MEGA_SUPER,
                        1.0 - 2.0 * MEGA_DEPTH * MEGA_SUPER,
                        MEGA_DEPTH * MEGA_SUPER,
                        0.0,
                        0.0,
                    ),
                    bary(
                        0.0,
                        MEGA_DEPTH * MEGA_SUPER,
                        1.0 - 2.0 * MEGA_DEPTH * MEGA_SUPER,
                        MEGA_DEPTH * MEGA_SUPER,
                        0.0,
                    ),
                    bary(1.0, 1.0, 1.0, 1.0, 1.0),
                ],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB]),
                face: PB.conjugate(conj),
                color: PL.conjugate(conj),
                vertices: vec![
                    bary(MEGA_DEPTH, 1.0 - 2.0 * MEGA_DEPTH, MEGA_DEPTH, 0.0, 0.0),
                    bary(0.0, MEGA_DEPTH, 1.0 - 2.0 * MEGA_DEPTH, MEGA_DEPTH, 0.0),
                    bary(
                        0.0,
                        MEGA_DEPTH * MEGA_SUPER,
                        1.0 - 2.0 * MEGA_DEPTH * MEGA_SUPER,
                        MEGA_DEPTH * MEGA_SUPER,
                        0.0,
                    ),
                    bary(
                        MEGA_DEPTH * MEGA_SUPER,
                        1.0 - 2.0 * MEGA_DEPTH * MEGA_SUPER,
                        MEGA_DEPTH * MEGA_SUPER,
                        0.0,
                        0.0,
                    ),
                ],
                options: Default::default(),
            });

            // edge
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    bary(0.0, MEGA_DEPTH, 1.0 - 2.0 * MEGA_DEPTH, MEGA_DEPTH, 0.0),
                    bary(MEGA_DEPTH, 1.0 - 2.0 * MEGA_DEPTH, MEGA_DEPTH, 0.0, 0.0),
                    bary(0.0, 1.0 - MEGA_DEPTH, MEGA_DEPTH, 0.0, 0.0),
                    bary(0.0, MEGA_DEPTH, 1.0 - MEGA_DEPTH, 0.0, 0.0),
                ],
                options: Default::default(),
            });

            // corner
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    bary(MEGA_DEPTH, 1.0 - 2.0 * MEGA_DEPTH, MEGA_DEPTH, 0.0, 0.0),
                    bary(MEGA_DEPTH, 1.0 - MEGA_DEPTH, 0.0, 0.0, 0.0),
                    bary(0.0, 1.0, 0.0, 0.0, 0.0),
                    bary(0.0, 1.0 - MEGA_DEPTH, MEGA_DEPTH, 0.0, 0.0),
                ],
                options: Default::default(),
            });

            viewports.push(ViewportSeed {
                abstract_viewport,
                conjugate: conj,
                stickers,
                key_layers: key_layers.clone(),
            });
        }

        // master pentultiate
        {
            let abstract_viewport = AbstractViewport {
                x: MEGA_SCALE,
                y: (1.0 - MPENT_SCALE) / 2.0 - y_off,
                width: MPENT_SCALE,
                height: MPENT_SCALE,
            };

            let bary = |a, b, c, d, e| bary(SCALE_CIRCUMRAD * MPENT_SCALE, a, b, c, d, e);
            let pt_b = bary(0.0, 1.0, 0.0, 0.0, 0.0);
            let pt_ba = bary(MPENT_DEPTH, 1.0 - MPENT_DEPTH, 0.0, 0.0, 0.0);
            let pt_ab = bary(1.0 - MPENT_DEPTH, MPENT_DEPTH, 0.0, 0.0, 0.0);
            let pt_iba = bary(
                MPENT_DEPTH,
                MPENT_DEPTH,
                1.0 - 2.0 * MPENT_DEPTH,
                -(1.0 - 2.0 * MPENT_DEPTH),
                1.0 - 2.0 * MPENT_DEPTH,
            );

            let sup_ac =
                bary_nosc(SCALE_CIRCUMRAD * MPENT_SCALE, -1.0, 0.0, 1.0, 0.0, 0.0) * MPENT_SUPER;
            let sup_be =
                bary_nosc(SCALE_CIRCUMRAD * MPENT_SCALE, 0.0, -1.0, 0.0, 0.0, 1.0) * MPENT_SUPER;

            let mut stickers: Vec<StickerSeed<DodecaRay>> = vec![];

            // starminx center
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD, PR, BL, BR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    pt_iba + sup_be,
                    pt_iba + sup_ac,
                    turn(pt_iba + sup_be),
                    bary(1.0, 1.0, 1.0, 1.0, 1.0),
                ],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD, PR, BL, BR]),
                face: PB.conjugate(conj),
                color: PD.conjugate(conj),
                vertices: vec![pt_iba, pt_iba + sup_ac, pt_iba + sup_be],
                options: Default::default(),
            });

            // crystal edge
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD, PR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![pt_iba, pt_ab, pt_ba],
                options: Default::default(),
            });

            // pentultimate corner
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD, PR, BL, DL]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![pt_ba, pt_b, turn(pt_ab)],
                options: Default::default(),
            });

            // star point
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD, PR, BL]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    pt_ba + sup_be,
                    pt_ba + sup_ac,
                    turn(pt_ab + sup_be),
                    turn(pt_ab + sup_ac),
                    turn(pt_iba),
                    pt_iba,
                ],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD, PR, BL]),
                face: PB.conjugate(conj),
                color: PD.conjugate(conj),
                vertices: vec![pt_ba + sup_ac, pt_ba + sup_be, pt_ba],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PB, PL, PD, PR, BL]),
                face: PB.conjugate(conj),
                color: PL.conjugate(conj),
                vertices: vec![turn(pt_ab + sup_ac), turn(pt_ab + sup_be), turn(pt_ab)],
                options: Default::default(),
            });

            viewports.push(ViewportSeed {
                abstract_viewport,
                conjugate: conj,
                stickers,
                key_layers: key_layers.clone(),
            });
        }

        // funny slice puzzle
        {
            let abstract_viewport = AbstractViewport {
                x: MEGA_SCALE + MPENT_SCALE,
                y: -y_off,
                width: 1.0,
                height: 1.0,
            };

            let bary = |a, b, c, d, e| bary(SCALE_CIRCUMRAD, a, b, c, d, e);
            let pt_b = bary(0.0, 1.0, 0.0, 0.0, 0.0);
            let pt_bai = bary(SLICE_INNER_DEPTH, 1.0 - SLICE_INNER_DEPTH, 0.0, 0.0, 0.0);
            let pt_bao = bary(SLICE_OUTER_DEPTH, 1.0 - SLICE_OUTER_DEPTH, 0.0, 0.0, 0.0);
            let pt_abi = bary(1.0 - SLICE_INNER_DEPTH, SLICE_INNER_DEPTH, 0.0, 0.0, 0.0);
            let pt_abo = bary(1.0 - SLICE_OUTER_DEPTH, SLICE_OUTER_DEPTH, 0.0, 0.0, 0.0);
            let pt_iba = bary(
                SLICE_INNER_DEPTH,
                SLICE_INNER_DEPTH,
                1.0 - 2.0 * SLICE_INNER_DEPTH,
                -(1.0 - 2.0 * SLICE_INNER_DEPTH),
                1.0 - 2.0 * SLICE_INNER_DEPTH,
            ); // also on the outer cut
            let pt_ibai = pt_iba + pt_bai - pt_abi;
            let pt_iabi = pt_iba + pt_abi - pt_bai;
            let pt_ib = pt_bao + turn(pt_abo) - pt_b;

            let sup_ac = bary_nosc(SCALE_CIRCUMRAD, -1.0, 0.0, 1.0, 0.0, 0.0) * SLICE_SUPER;
            let sup_be = bary_nosc(SCALE_CIRCUMRAD, 0.0, -1.0, 0.0, 0.0, 1.0) * SLICE_SUPER;
            let sup_ba = bary_nosc(SCALE_CIRCUMRAD, 1.0, -1.0, 0.0, 0.0, 0.0) * SLICE_SUPER;
            let sup_bc = bary_nosc(SCALE_CIRCUMRAD, 0.0, -1.0, 1.0, 0.0, 0.0) * SLICE_SUPER;

            let mut stickers: Vec<StickerSeed<DodecaRay>> = vec![];

            // ring
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, PR, BL, BR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    pt_iba + sup_be,
                    pt_iba + sup_ac,
                    turn(pt_iba + sup_be),
                    bary(1.0, 1.0, 1.0, 1.0, 1.0),
                ],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, PR, BL, BR]),
                face: PB.conjugate(conj),
                color: PD.conjugate(conj),
                vertices: vec![pt_iba, pt_iba + sup_ac, pt_iba + sup_be],
                options: Default::default(),
            });

            // coedge
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![pt_iba, pt_abi, pt_bai],
                options: Default::default(),
            });

            // cocorner
            stickers.push(StickerSeed {
                layers: make_grips(vec![PR, BL, DL]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![pt_bao, pt_b, turn(pt_abo), pt_ib],
                options: Default::default(),
            });

            // snake
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, PR, BL]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    pt_iba + sup_ac,
                    pt_ibai + sup_ac,
                    turn(pt_iabi + sup_be),
                    turn(pt_iba + sup_be),
                ],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, PR, BL]),
                face: PB.conjugate(conj),
                color: PD.conjugate(conj),
                vertices: vec![pt_iba, pt_ibai, pt_ibai + sup_ac, pt_iba + sup_ac],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, PR, BL]),
                face: PB.conjugate(conj),
                color: PL.conjugate(conj),
                vertices: vec![
                    turn(pt_iabi),
                    turn(pt_iba),
                    turn(pt_iba + sup_be),
                    turn(pt_iabi + sup_be),
                ],
                options: Default::default(),
            });

            // bull
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, DL, BL, PR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![
                    pt_ibai + sup_ac,
                    pt_ib + sup_ba + sup_bc,
                    turn(pt_iabi + sup_be),
                ],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, DL, BL, PR]),
                face: PB.conjugate(conj),
                color: PD.conjugate(conj),
                vertices: vec![pt_ibai, pt_ib, pt_ib + sup_ba + sup_bc, pt_ibai + sup_ac],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PD, DL, BL, PR]),
                face: PB.conjugate(conj),
                color: PL.conjugate(conj),
                vertices: vec![
                    turn(pt_iabi),
                    turn(pt_iabi + sup_be),
                    pt_ib + sup_ba + sup_bc,
                    pt_ib,
                ],
                options: Default::default(),
            });

            // cotadpole
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, DL, BL, PR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![pt_ibai, pt_bai, pt_bao, pt_ib],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PD, DL, BL, PR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![turn(pt_abi), turn(pt_iabi), pt_ib, turn(pt_abo)],
                options: Default::default(),
            });

            // coworm
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PR, BL]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![pt_bai + sup_be, pt_bai + sup_ac, pt_ibai, pt_iba],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PR, BL]),
                face: PB.conjugate(conj),
                color: PD.conjugate(conj),
                vertices: vec![pt_bai + sup_ac, pt_bai + sup_be, pt_bai],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PR, BR]),
                face: PB.conjugate(conj),
                color: PB.conjugate(conj),
                vertices: vec![pt_abi + sup_be, pt_abi + sup_ac, pt_iba, pt_iabi],
                options: Default::default(),
            });
            stickers.push(StickerSeed {
                layers: make_grips(vec![PL, PR, BR]),
                face: PB.conjugate(conj),
                color: PD.conjugate(conj),
                vertices: vec![pt_abi + sup_ac, pt_abi + sup_be, pt_abi],
                options: Default::default(),
            });

            viewports.push(ViewportSeed {
                abstract_viewport,
                conjugate: conj,
                stickers,
                key_layers: key_layers.clone(),
            });
        }
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
