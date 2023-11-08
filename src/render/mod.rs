#![allow(unused_variables)]
use crate::puzzle::common::*;
use crate::puzzle::cube::*;
use enum_map::{enum_map, EnumMap};
use std::cmp;
use std::iter;

use std::f32::consts::PI;
use three_d::*;

pub struct ConcretePuzzle<'a, Ray>
where
    Ray: RaySystem,
{
    pub puzzle: Puzzle<'a, Ray>,
    pub stickers: Vec<Sticker<Ray>>,
}

impl<Ray: RaySystem> ConcretePuzzle<'_, Ray> {
    pub fn ray_intersect(&self, position: &Vec3, direction: &Vec3) -> Option<&Sticker<Ray>> {
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
    pub cpu_mesh: CpuMesh,
}

impl<Ray: RaySystem> Sticker<Ray> {
    fn ray_intersect(&self, position: &Vec3, direction: &Vec3) -> Option<f32> {
        let positions = self.cpu_mesh.positions.to_f32();
        let indices = self
            .cpu_mesh
            .indices
            .to_u32()
            .unwrap_or_else(|| (0..self.cpu_mesh.positions.len() as u32 * 3).collect());
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

/*
/// Turn the entire sticker.
fn transform(sticker: &mut Sticker<CubeRay>, turn: &(CubeRay, i8), mat: &Mat4) {
    // TODO: make turns consistent across codebase
    let temp_layers = EnumMap::from_fn(|ray| sticker.piece.layers[ray.turn(turn.0)]);
}*/

fn axis_to_transform(&(ray, order): &(CubeRay, i8)) -> Mat4 {
    use CubeRay::*;
    match ray {
        U | D => Mat4::from_angle_z(Rad(PI / 2.0 * (order as f32))),
        R | L => Mat4::from_angle_x(Rad(PI / 2.0 * (order as f32))),
        F | B => Mat4::from_angle_y(Rad(-PI / 2.0 * (order as f32))),
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

pub fn make_concrete_puzzle<'a>() -> ConcretePuzzle<'a, CubeRay> {
    use CubeRay::*;
    let mut puzzle: Puzzle<'a, CubeRay> = Puzzle::make_solved(vec![&[0, 0], &[1, 0], &[0, 1]]);
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
    let init_data = &mut [
        (
            enum_map! {U=>1,R=>1,B=>1,D=>0,L=>0,F=>0,},
            U,
            U,
            corner_mesh,
        ),
        (enum_map! {U=>1,R=>1,B=>0,D=>0,L=>0,F=>0,}, U, U, edge_mesh),
    ];
    let mut stickers = vec![];
    for (layers, face, color, cpu_mesh) in init_data.into_iter() {
        for turn_m in iter::once(None).chain(CubeRay::CYCLE.iter().map(|x| Some(x))) {
            if let Some(turn) = turn_m {
                let &(turn_ray, turn_order) = turn;
                *layers =
                    EnumMap::from_fn(|ray: CubeRay| layers[ray.turn(&(turn_ray, -turn_order))]);
                *face = face.turn(&(turn_ray, turn_order));
                *color = color.turn(&(turn_ray, turn_order));
                cpu_mesh
                    .transform(&axis_to_transform(turn))
                    .expect("the axis transform matrices should be invertible");
            }
            let piece_ind = puzzle.piece_to_index(&Piece::make_solved_from_layers(layers.clone()));
            stickers.push(Sticker {
                piece_ind,
                face: face.clone(),
                color: color.clone(),
                cpu_mesh: cpu_mesh.clone(),
            });
        }
    }

    ConcretePuzzle { puzzle, stickers }
}

pub fn concrete_puzzle_gm(
    context: &Context,
    concrete_puzzle: &ConcretePuzzle<CubeRay>,
) -> Vec<Gm<Mesh, ColorMaterial>> {
    let puzzle = &concrete_puzzle.puzzle;

    let mut sticker_meshes = vec![];

    for sticker in &concrete_puzzle.stickers {
        let sticker_mesh = Gm::new(
            Mesh::new(context, &sticker.cpu_mesh),
            ColorMaterial {
                color: ray_to_color(
                    &puzzle.pieces[puzzle.permutation[sticker.piece_ind]].orientation
                        [sticker.color],
                ),
                ..Default::default()
            },
        );
        sticker_meshes.push(sticker_mesh);
        //screen.render(camera, sticker_mesh.into_iter(), &[]);
    }
    return sticker_meshes;
}
