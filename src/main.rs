#![allow(dead_code)]
use enum_map::{enum_map, Enum, EnumMap};

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
enum CubeRay {
    U,
    D,
    F,
    B,
    R,
    L,
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
enum CubeAxis {
    UD,
    FB,
    RL,
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
enum CubeSubaxis {
    Ray(CubeRay),
    Slice(CubeAxis),
}

type CubeLayer = (CubeSubaxis, u8);

fn ray_axis(ray: &CubeRay) -> CubeAxis {
    use CubeAxis::*;
    use CubeRay::*;

    match ray {
        U | D => UD,
        F | B => FB,
        R | L => RL,
    }
}

fn subaxis_axis(layer: &CubeSubaxis) -> CubeAxis {
    match layer {
        CubeSubaxis::Ray(ray) => ray_axis(ray),
        CubeSubaxis::Slice(axis) => *axis,
    }
}

fn turn_ray(ray: &CubeRay, axis: &CubeAxis) -> CubeRay {
    use CubeAxis::*;
    use CubeRay::*;

    match (axis, ray) {
        (UD, U) => U,
        (UD, D) => D,
        (UD, F) => R,
        (UD, B) => L,
        (UD, R) => B,
        (UD, L) => F,

        (FB, U) => F,
        (FB, D) => B,
        (FB, F) => D,
        (FB, B) => U,
        (FB, R) => R,
        (FB, L) => L,

        (RL, U) => L,
        (RL, D) => R,
        (RL, F) => F,
        (RL, B) => B,
        (RL, R) => U,
        (RL, L) => D,
    }
}

fn turn_axis(to_move: &CubeAxis, axis: &CubeAxis) -> CubeAxis {
    use CubeAxis::*;

    match (axis, to_move) {
        (UD, UD) => UD,
        (UD, FB) => RL,
        (UD, RL) => FB,

        (RL, UD) => FB,
        (RL, FB) => UD,
        (RL, RL) => RL,

        (FB, UD) => RL,
        (FB, FB) => FB,
        (FB, RL) => UD,
    }
}

fn turn_subaxis(subaxis: &CubeSubaxis, axis: &CubeAxis) -> CubeSubaxis {
    match subaxis {
        CubeSubaxis::Ray(ray) => CubeSubaxis::Ray(turn_ray(ray, axis)),
        CubeSubaxis::Slice(to_move) => CubeSubaxis::Slice(turn_axis(to_move, axis)),
    }
}

/// A single piece of an abstract laminated puzzle.
#[derive(Debug)]
struct Piece {
    /// For each axis, the layer this piece occupies.
    layers: EnumMap<CubeAxis, CubeLayer>,
    /// For each ray, the ray that is currently occupying its direction.
    orientation: EnumMap<CubeRay, CubeRay>,
}

fn make_solved(layers: EnumMap<CubeAxis, CubeLayer>) -> Piece {
    Piece {
        layers,
        orientation: enum_map! {ray => ray},
    }
}

fn is_solved(piece: &Piece) -> bool {
    piece.orientation.iter().all(|(pos, cur)| &pos == cur)
}

fn turn(piece: &mut Piece, layer: &CubeLayer, order: u8) {
    let (subaxis, _) = layer;
    let axis = subaxis_axis(&subaxis);
    if layer == &piece.layers[axis] {
        for _ in 0..order {
            for (pos, cur) in piece.orientation {
                piece.orientation[pos] = turn_ray(&cur, &axis);
            }
            let mut new_layers = enum_map! {_=>(CubeSubaxis::Slice(CubeAxis::UD),0)};
            for (_, (sax, i)) in piece.layers {
                let new_subaxis = turn_subaxis(&sax, &axis);
                new_layers[subaxis_axis(&new_subaxis)] = (new_subaxis, i);
            }
            piece.layers = new_layers;
        }
    }
}

fn main() {
    use CubeAxis::*;
    use CubeRay::*;
    use CubeSubaxis::*;

    let mut edge = make_solved(enum_map! {
        UD => (Ray(U), 1),
        FB => (Ray(B), 1),
        RL => (Slice(RL), 0),
    });
    println!("{:?}", edge);
    println!("edge is solved? {:?}", is_solved(&edge));

    println!("turn U");
    turn(&mut edge, &(Ray(U), 1), 1);
    println!("{:?}", edge);
}
