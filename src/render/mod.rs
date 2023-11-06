use crate::puzzle::common::*;
use crate::puzzle::cube::*;
use enum_map::{enum_map, EnumMap};
use std::iter;

use std::f32::consts::PI;
use three_d::*;

struct ConcretePuzzle<'a, Ray>
where
    Ray: RaySystem,
{
    puzzle: Puzzle<'a, Ray>,
    stickers: Vec<Sticker<Ray>>,
}

struct Sticker<Ray>
where
    Ray: RaySystem,
{
    /// The index of the piece this sticker is part of in `permutation` on `Puzzle`.
    piece_ind: usize,
    /// The face the sticker is on. Controls what turn is done when clicked.
    face: Ray,
    /// The face that controls the color of the sticker.
    color: Ray,
    cpu_mesh: CpuMesh,
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

fn make_concrete_puzzle() {
    use CubeRay::*;
    let mut puzzle: Puzzle<'_, CubeRay> = Puzzle::make_solved(vec![&[0, 0], &[1, 0], &[0, 1]]);
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
            &(Mat4::from_translation(vec3(0.0, 2.0 / 3.0, 1.0)) * Mat4::from_scale(1.0 / 3.0)),
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

                /*let mut sticker = init_mesh.clone();
                sticker
                    .transform(&(sticker_transform))
                    .expect("the matrix should be invertible");
                stickers.push(sticker);*/
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
}
