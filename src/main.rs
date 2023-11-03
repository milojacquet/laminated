pub mod puzzle;

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

    /*let mut puzzle = puzzle::Puzzle::make_solved(|ax| {
        let mut layers: Vec<puzzle::CubeLayer> = ax
            .rays()
            .iter()
            .map(|r| (puzzle::CubeSubaxis::Ray(*r), 1))
            .collect();
        layers.splice(0..0, [(puzzle::CubeSubaxis::Slice(ax), 0)]);
        layers
    });
    puzzle.twist(&((puzzle::CubeSubaxis::Ray(puzzle::CubeRay::R), 1), 1));
    println!("{:?}", puzzle.pieces[1]);*/
}
