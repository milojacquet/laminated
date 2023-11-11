use crate::render::common::ConcreteRaySystem;
use crate::CubeRay;
use std::f32::consts::PI;
use std::iter;
use three_d::*;
use CubeRay::*;
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

/*pub fn make_concrete_nnn<'a>(window: &Window, order: i8) -> ConcretePuzzle<'a, CubeRay> {
    use CubeRay::*;

    let axes = (if order & 1 == 1 { &[&[0, 0]] } else { &[] })
        .chain((1..=order / 2).map(|k| &[&[k, -k], &[-k, k]]).flatten())
        .collect();
    let puzzle: Puzzle<'a, CubeRay> = Puzzle::make_solved(axes);

    /*let abstract_viewports = vec![
        AbstractViewport {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        AbstractViewport {
            x: 1.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
    ];

    let viewports = abstract_viewports
        .iter()
        .enumerate()
        .map(|(i, abstract_viewport)| {
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
                    &(Mat4::from_translation(vec3(2.0 / 3.0, 0.0, 1.0))
                        * Mat4::from_scale(1.0 / 3.0)),
                )
                .expect("the matrix should be invertible i made it");
            let sticker_seeds = &mut [
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
            ];

            let viewport = make_viewport(&window, &abstract_viewports[..], i);

            let mut stickers = vec![];
            for seed in sticker_seeds.iter_mut() {
                for turn_m in iter::once(None).chain(CubeRay::CYCLE.iter().map(Some)) {
                    if let Some(turn) = turn_m {
                        let &(turn_ray, turn_order) = turn;
                        seed.layers = EnumMap::from_fn(|ray: CubeRay| {
                            seed.layers[ray.turn(&(turn_ray, -turn_order))]
                        });
                        seed.face = seed.face.turn(turn);
                        seed.color = seed.color.turn(turn);
                        seed.cpu_mesh
                            .transform(&CubeRay::axis_to_transform(turn, Default::default()))
                            .expect("the axis transform matrices should be invertible");
                    }
                    let piece_ind =
                        puzzle.piece_to_index(&Piece::make_solved_from_layers(seed.layers.clone()));
                    let mut new_cpu_mesh = seed.cpu_mesh.clone();

                    new_cpu_mesh.compute_normals();
                    stickers.push(Sticker {
                        piece_ind,
                        face: seed.face.clone(),
                        color: seed.color.clone(),
                        cpu_mesh: new_cpu_mesh,
                        animation: None,
                    });
                }
            }

            PuzzleViewport {
                abstract_viewport: abstract_viewport.clone(),
                viewport,
                camera: Camera::new_perspective(
                    viewport,
                    vec3(5.0, -10.0, 4.0),
                    vec3(0.0, 0.0, 0.0),
                    vec3(0.0, 0.0, 1.0),
                    degrees(22.0),
                    0.1,
                    1000.0,
                ),
                conjugate: (),
                stickers,
            }
        })
        .collect();

    ConcretePuzzle { puzzle, viewports }*/
}
*/
