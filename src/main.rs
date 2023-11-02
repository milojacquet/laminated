#![allow(dead_code)]
use enum_map::{enum_map, Enum, EnumMap};

trait CubeSystem {
    fn to_axis(&self) -> CubeAxis;
    fn turn(&self, axis: &CubeAxis) -> Self;
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
enum CubeRay {
    U,
    D,
    F,
    B,
    R,
    L,
}

impl CubeSystem for CubeRay {
    fn to_axis(&self) -> CubeAxis {
        use CubeAxis::*;
        use CubeRay::*;

        match self {
            U | D => UD,
            F | B => FB,
            R | L => RL,
        }
    }

    fn turn(&self, axis: &CubeAxis) -> Self {
        use CubeAxis::*;
        use CubeRay::*;

        match (axis, self) {
            (UD, U) => U,
            (UD, D) => D,
            (UD, F) => L,
            (UD, B) => R,
            (UD, R) => F,
            (UD, L) => B,

            (FB, U) => R,
            (FB, D) => L,
            (FB, F) => F,
            (FB, B) => B,
            (FB, R) => D,
            (FB, L) => U,

            (RL, U) => B,
            (RL, D) => F,
            (RL, F) => U,
            (RL, B) => D,
            (RL, R) => R,
            (RL, L) => L,
        }
    }
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
enum CubeAxis {
    UD,
    FB,
    RL,
}

// impl<V> EnumArray<V> for CubeAxis {}

impl CubeSystem for CubeAxis {
    fn to_axis(&self) -> CubeAxis {
        *self
    }

    fn turn(&self, axis: &CubeAxis) -> Self {
        use CubeAxis::*;

        match (axis, self) {
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
}

impl CubeAxis {
    fn rays(&self) -> Vec<CubeRay> {
        use CubeAxis::*;
        use CubeRay::*;

        match self {
            UD => vec![U, D],
            FB => vec![F, B],
            RL => vec![R, L],
        }
    }

    fn order(&self) -> i8 {
        4
    }
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
enum CubeSubaxis {
    Ray(CubeRay),
    Slice(CubeAxis),
}

impl CubeSystem for CubeSubaxis {
    fn to_axis(&self) -> CubeAxis {
        match self {
            CubeSubaxis::Ray(ray) => ray.to_axis(),
            CubeSubaxis::Slice(axis) => *axis,
        }
    }

    fn turn(&self, axis: &CubeAxis) -> Self {
        match self {
            CubeSubaxis::Ray(ray) => CubeSubaxis::Ray(ray.turn(axis)),
            CubeSubaxis::Slice(to_move) => CubeSubaxis::Slice(to_move.turn(axis)),
        }
    }
}

type CubeLayer = (CubeSubaxis, u8);
type CubeTwist = (CubeLayer, i8);

#[rustfmt::skip]
const CUBE_CYCLE: [(CubeAxis, i8); 23] = {
    use CubeAxis::*;
    // 3u 3u 3u 3f 3u 3u 3u 3f 3u 3u 3u 3f' 3u 3u 3u 3f' 3u 3u 3u 3f 3u 3u 3u 
    [
        (UD, 1), (UD, 1), (UD, 1), (FB, 1),
        (UD, 1), (UD, 1), (UD, 1), (FB, 1),
        (UD, 1), (UD, 1), (UD, 1), (FB, 3),
        (UD, 1), (UD, 1), (UD, 1), (FB, 3),
        (UD, 1), (UD, 1), (UD, 1), (FB, 1),
        (UD, 1), (UD, 1), (UD, 1),
    ]
};

/// A single piece of an abstract laminated puzzle.
#[derive(Debug)]
struct Piece {
    /// For each axis, the layer this piece occupies.
    layers: EnumMap<CubeAxis, CubeLayer>,
    /// For each ray, the direction the ray currently points.
    orientation: EnumMap<CubeRay, CubeRay>,
}

impl Piece {
    fn make_solved(layers: EnumMap<CubeAxis, CubeLayer>) -> Self {
        Self {
            layers,
            orientation: EnumMap::from_fn(|ray| ray),
        }
    }

    fn is_solved(&self) -> bool {
        self.orientation.iter().all(|(cur, pos)| &cur == pos)
    }

    fn twist(&mut self, (layer, order): &CubeTwist) {
        let (subaxis, _) = layer;
        let axis = subaxis.to_axis();
        let order = if order < &0 {
            layer.0.to_axis().order() + order
        } else {
            *order
        };
        if layer == &self.layers[axis] {
            for _ in 0..order {
                for (cur, pos) in self.orientation {
                    self.orientation[cur] = pos.turn(&axis);
                }
                let mut new_layers = enum_map! {_=>(CubeSubaxis::Slice(CubeAxis::UD),0)};
                for (_, (sax, i)) in self.layers {
                    let new_subaxis = sax.turn(&axis);
                    new_layers[new_subaxis.to_axis()] = (new_subaxis, i);
                }
                self.layers = new_layers;
            }
        }
    }
}

/// Abstract laminated puzzle.
#[derive(Debug)]
struct Puzzle {
    grips: EnumMap<CubeAxis, Vec<CubeLayer>>,
    pieces: Vec<Piece>,
}

impl Puzzle {
    fn make_solved<F>(axis_to_grips: F) -> Self
    where
        F: Fn(CubeAxis) -> Vec<CubeLayer>,
    {
        let grips = EnumMap::from_fn(axis_to_grips);
        let grips_arr = grips.as_array();
        let grip_count: usize = grips_arr[0].len();
        let piece_count: usize = grip_count.pow(grips_arr.len().try_into().unwrap());
        let pieces = (0..piece_count)
            .map(|i| {
                Piece::make_solved(EnumMap::from_array(
                    grips_arr
                        .into_iter()
                        .enumerate()
                        .map(|(j, gs)| gs[i / grip_count.pow(j.try_into().unwrap()) % grip_count])
                        .collect::<Vec<CubeLayer>>()
                        .try_into()
                        .expect("it had better be the right size"),
                ))
            })
            .collect();
        Self {
            grips,
            /*pieces: grips
            .values()
            .multi_cartesian_product()
            .map(|layers| Piece::make_solved(EnumMap::from_array(layers)))
            .collect(),*/
            pieces,
        }
    }

    fn is_solved(&self) -> bool {
        self.pieces.iter().all(|piece| piece.is_solved())
    }

    fn twist(&mut self, ctwist: &CubeTwist) {
        let (layer, order) = ctwist;
        if layer == &self.grips[layer.0.to_axis()][0] {
            // we cannot move the core
            let other_layers: Vec<CubeLayer> = self.grips[layer.0.to_axis()]
                .iter()
                .skip(1)
                .copied()
                .collect();
            for new_layer in other_layers {
                self.twist(&(new_layer, -order))
            }
        } else {
            for piece in &mut self.pieces {
                piece.twist(ctwist);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Applies one turn and asserts that it is unsolved..
    #[test]
    fn one_turn() {
        use CubeRay::*;
        use CubeSubaxis::*;

        let mut puzzle = Puzzle::make_solved(|ax| {
            let mut layers: Vec<CubeLayer> = ax.rays().iter().map(|r| (Ray(*r), 1)).collect();
            layers.splice(0..0, [(Slice(ax), 0)]);
            layers
        });
        puzzle.twist(&((Ray(R), 1), 1));
        assert!(!puzzle.is_solved());
    }

    /// Applies (R U R' U')6 to the 3x3x3 and asserts that it is solved.
    #[test]
    fn six_sexy() {
        use CubeRay::*;
        use CubeSubaxis::*;

        let mut puzzle = Puzzle::make_solved(|ax| {
            let mut layers: Vec<CubeLayer> = ax.rays().iter().map(|r| (Ray(*r), 1)).collect();
            layers.splice(0..0, [(Slice(ax), 0)]);
            layers
        });
        for _ in 0..6 {
            puzzle.twist(&((Ray(R), 1), 1));
            puzzle.twist(&((Ray(U), 1), 1));
            puzzle.twist(&((Ray(R), 1), 3));
            puzzle.twist(&((Ray(U), 1), 3));
        }
        assert!(puzzle.is_solved());
    }
}

fn main() {
    //use CubeAxis::*;
    //use CubeRay::*;
    //use CubeSubaxis::*;

    /*let mut edge = Piece::make_solved(enum_map! {
        UD => (Ray(U), 1),
        FB => (Ray(B), 1),
        RL => (Slice(RL), 0),
    });
    println!("{:?}", edge);
    println!("edge is solved? {:?}", edge.is_solved());

    println!("turn U");
    edge.twist(&(Ray(U), 1), 1);
    println!("{:?}", edge);*/
}
