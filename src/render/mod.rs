use crate::puzzle::common::*;
use crate::puzzle::cube::*;
use crate::ANIMATION_INIT_V;
use crate::ANIMATION_LENGTH;
use enum_map::{enum_map, EnumMap};
use std::cmp;
use std::iter;

use std::f32::consts::PI;
use three_d::*;

/// The initial data which will be symmetry-expanded into a sticker.
#[derive(Debug)]
pub struct StickerSeed<Ray>
where
    Ray: RaySystem,
{
    /// The index of the piece this sticker is part of in `permutation` on `Puzzle`.
    pub layers: EnumMap<CubeRay, u8>,
    /// The face the sticker is on. Controls what turn is done when clicked.
    pub face: Ray,
    /// The face that controls the color of the sticker.
    pub color: Ray,
    pub cpu_mesh: CpuMesh,
}

#[derive(Debug)]
pub struct StickerAnimation {
    /// the axis the sticker is turning around
    pub rotation_axis: Vec3,
    /// the angle the axis starts at
    pub start_angle: f32,
    /// the time remaining for the animation, in milliseconds
    pub time_remaining: f32,
}

#[derive(Debug)]
pub struct Sticker<Ray>
where
    Ray: RaySystem,
{
    /// The index of the piece this sticker is part of in `permutation` on `Puzzle`.
    pub piece_ind: usize,
    /// The face the sticker is on. Controls what turn is done when clicked.
    pub face: Ray,
    /// The face that controls the color of the sticker.
    pub color: Ray,
    /// The mesh of the sticker.
    pub cpu_mesh: CpuMesh,
    pub animation: Option<StickerAnimation>,
}

impl<Ray: RaySystem> Sticker<Ray> {
    fn ray_intersect(&self, position: &Vec3, direction: &Vec3) -> Option<f32> {
        let positions = self.cpu_mesh.positions.to_f32();
        let indices = self
            .cpu_mesh
            .indices
            .to_u32()
            .unwrap_or_else(|| (0..self.cpu_mesh.positions.len() as u32).collect());
        indices[..]
            .chunks_exact(3)
            .map(|inds| {
                let verts = &inds
                    .iter()
                    .map(|&i| positions[i as usize])
                    .collect::<Vec<_>>()[..];
                ray_triangle_intersect(position, direction, verts)
            })
            .filter_map(|x| x)
            .reduce(f32::min)
    }
}

fn ray_triangle_intersect(position: &Vec3, direction: &Vec3, verts: &[Vec3]) -> Option<f32> {
    // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    const EPSILON: f32 = 0.0000001;
    let edge1 = verts[1] - verts[0];
    let edge2 = verts[2] - verts[0];
    let h = direction.cross(edge2);
    let a = edge1.dot(h);
    if a.abs() < EPSILON {
        return None;
    }
    let f = 1.0 / a;
    let s = position - verts[0];
    let u = f * s.dot(h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = f * edge2.dot(q);

    if t > EPSILON {
        // ray intersection
        return Some(t);
    } else {
        // This means that there is a line intersection but not a ray intersection.
        return None;
    }
}

//pub struct ConcretePuzzle<'a, Ray>
pub struct ConcretePuzzle<'a>
//where
//    Ray: RaySystem,
{
    pub puzzle: Puzzle<'a, CubeRay>,
    pub stickers: Vec<Sticker<CubeRay>>,
}

//impl<Ray: RaySystem> ConcretePuzzle<'_, Ray> {
impl ConcretePuzzle<'_> {
    pub fn ray_intersect(&self, position: &Vec3, direction: &Vec3) -> Option<&Sticker<CubeRay>> {
        self.stickers
            .iter()
            .filter_map(|sticker| {
                sticker
                    .ray_intersect(position, direction)
                    .map(|d| (d, sticker))
            })
            .reduce(|ds1, ds2| {
                cmp::min_by(ds1, ds2, |ds1, ds2| {
                    ds1.0.partial_cmp(&ds2.0).expect("not nan")
                })
            })
            .map(|ds| ds.1)
    }

    pub fn twist(&mut self, &(ray, order): &(CubeRay, i8), grip: &[u8]) {
        let twisted = self.puzzle.twist((ray, order), grip);
        //println!("{:?}", twisted);
        for sticker in self.stickers.iter_mut() {
            if twisted.contains(&sticker.piece_ind) {
                sticker.animation = Some(StickerAnimation {
                    rotation_axis: axis_to_vec(&ray),
                    start_angle: (order as f32) * 2.0 * PI / (ray.order() as f32),
                    time_remaining: ANIMATION_LENGTH,
                })
            }
        }
    }
}

fn axis_to_transform(&(ray, order): &(CubeRay, i8)) -> Mat4 {
    use CubeRay::*;
    match ray {
        U | D => Mat4::from_angle_z(Rad(PI / 2.0 * (order as f32))),
        R | L => Mat4::from_angle_x(Rad(PI / 2.0 * (order as f32))),
        F | B => Mat4::from_angle_y(Rad(-PI / 2.0 * (order as f32))),
    }
}

fn axis_to_vec(ray: &CubeRay) -> Vec3 {
    use CubeRay::*;
    match ray {
        U | D => Vec3::new(0.0, 0.0, 1.0),
        R | L => Vec3::new(1.0, 0.0, 0.0),
        F | B => Vec3::new(0.0, -1.0, 0.0),
    }
}

fn ray_to_color(ray: &CubeRay) -> Srgba {
    use CubeRay::*;
    match ray {
        U => Srgba::WHITE,
        F => Srgba::RED,
        R => Srgba::BLUE,
        B => Srgba::new_opaque(255, 128, 0),
        L => Srgba::GREEN,
        D => Srgba::new_opaque(255, 255, 0),
    }
}

//pub fn make_concrete_puzzle<'a>() -> ConcretePuzzle<'a, CubeRay> {
pub fn make_concrete_puzzle<'a>() -> ConcretePuzzle<'a> {
    use CubeRay::*;
    let puzzle: Puzzle<'a, CubeRay> = Puzzle::make_solved(vec![&[0, 0], &[1, 0], &[0, 1]]);
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
    let sticker_seeds = &mut [
        StickerSeed {
            layers: enum_map! {U=>1,R=>1,B=>1,D=>0,L=>0,F=>0,},
            face: U,
            color: U,
            cpu_mesh: corner_mesh,
        },
        StickerSeed {
            layers: enum_map! {U=>1,R=>1,B=>0,D=>0,L=>0,F=>0,},
            face: U,
            color: U,
            cpu_mesh: edge_mesh,
        },
        StickerSeed {
            layers: enum_map! {U=>1,R=>0,B=>0,D=>0,L=>0,F=>0,},
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
            layers: enum_map! {U=>1,R=>0,B=>0,D=>0,L=>0,F=>0,},
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
    let mut stickers = vec![];
    for seed in sticker_seeds.into_iter() {
        for turn_m in iter::once(None).chain(CubeRay::CYCLE.iter().map(|x| Some(x))) {
            if let Some(turn) = turn_m {
                let &(turn_ray, turn_order) = turn;
                seed.layers = EnumMap::from_fn(|ray: CubeRay| {
                    seed.layers[ray.turn(&(turn_ray, -turn_order))]
                });
                seed.face = seed.face.turn(turn);
                seed.color = seed.color.turn(turn);
                seed.cpu_mesh
                    .transform(&axis_to_transform(turn))
                    .expect("the axis transform matrices should be invertible");
            }
            let piece_ind =
                puzzle.piece_to_index(&Piece::make_solved_from_layers(seed.layers.clone()));
            stickers.push(Sticker {
                piece_ind,
                face: seed.face.clone(),
                color: seed.color.clone(),
                cpu_mesh: seed.cpu_mesh.clone(),
                animation: None,
            });
        }
    }

    ConcretePuzzle { puzzle, stickers }
}

/// Smoothly maps 0 to 0 and 1 to 1, with derivative ANIMATION_INIT_V at 0 and 1.
pub fn cubic_interpolate(t: f32) -> f32 {
    ANIMATION_INIT_V * (2.0 * t * t * t - 3.0 * t * t + t) - (2.0 * t * t * t - 3.0 * t * t)
}

pub fn concrete_puzzle_gm(
    context: &Context,
    elapsed_time: &f32,
    //concrete_puzzle: &ConcretePuzzle<CubeRay>,
    concrete_puzzle: &mut ConcretePuzzle,
) -> Vec<Gm<Mesh, ColorMaterial>> {
    let puzzle = &concrete_puzzle.puzzle;

    let mut sticker_meshes = vec![];

    for sticker in concrete_puzzle.stickers.iter_mut() {
        let mut sticker_mesh = Gm::new(
            Mesh::new(context, &sticker.cpu_mesh),
            ColorMaterial {
                color: ray_to_color(
                    &puzzle.pieces[puzzle.permutation[sticker.piece_ind]].orientation
                        [sticker.color],
                ),
                ..Default::default()
            },
        );

        // can this section be written better
        let remove_animation;
        let sticker_mat;
        if let Some(animation) = &mut sticker.animation {
            animation.time_remaining -= elapsed_time;
            remove_animation = animation.time_remaining < 0.0;
        } else {
            remove_animation = false;
        }
        if remove_animation {
            sticker.animation = None;
        }
        if let Some(animation) = &mut sticker.animation {
            let sticker_angle = animation.start_angle
                * cubic_interpolate(animation.time_remaining / ANIMATION_LENGTH);
            sticker_mat = Mat4::from_axis_angle(animation.rotation_axis, Rad(sticker_angle));
        } else {
            sticker_mat = Mat4::identity();
        }
        sticker_mesh.set_transformation(sticker_mat);

        sticker_meshes.push(sticker_mesh);
        //screen.render(camera, sticker_mesh.into_iter(), &[]);
    }
    return sticker_meshes;
}
