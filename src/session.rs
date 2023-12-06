use crate::make_concrete_puzzle;
use crate::puzzle::common::*;
use crate::render;
use crate::render::common::*;
use crate::CubeRay;

pub struct Session<Ray: ConcreteRaySystem> {
    pub scramble: Vec<usize>,
    pub concrete_puzzle: ConcretePuzzle<Ray>,
    pub twists: Vec<((Ray, i8), Vec<Vec<i8>>)>,
    pub undid_twists: Vec<((Ray, i8), Vec<Vec<i8>>)>,
    // None: the mouse is not pressed.
    // Some((conj, None)): the mouse is being held from a viewport with conjugation conj, and camera orbiting has started.
    // Some((conj, Some((loc, button)))): the mouse is being held from a viewport with conjugation conj, and camera orbiting has not yet started. the mouse was pressed at loc with button.
    pub mouse_press_location: Option<(
        Ray::Conjugate,
        Option<(three_d::LogicalPoint, three_d::MouseButton)>,
    )>,
}

impl<'a, Ray: ConcreteRaySystem> Session<Ray> {
    pub fn from_concrete(concrete_puzzle: ConcretePuzzle<Ray>) -> Session<Ray> {
        Session {
            scramble: concrete_puzzle.puzzle.permutation(),
            concrete_puzzle,
            twists: vec![],
            undid_twists: vec![],
            mouse_press_location: None,
        }
    }

    fn multi_layer_twist(&mut self, tw: (Ray, i8), grips: &Vec<Vec<i8>>) {
        for grip in grips {
            self.concrete_puzzle.twist(tw, &grip[..]);
        }
    }

    pub fn twist(&mut self, tw: (Ray, i8), grips: Vec<Vec<i8>>) {
        self.multi_layer_twist(tw, &grips);
        self.twists.push((tw, grips));
        self.undid_twists = vec![];
    }

    fn scramble_from_concrete(&mut self) {
        self.concrete_puzzle.reset_animations();
        self.scramble = self.concrete_puzzle.puzzle.permutation();
        self.twists = vec![];
        self.undid_twists = vec![];
    }

    pub fn scramble(&mut self) {
        self.concrete_puzzle.puzzle.scramble();
        self.scramble_from_concrete();
    }

    pub fn reset(&mut self) {
        let new_puzzle = Puzzle::make_solved(self.concrete_puzzle.puzzle.grips.clone());
        self.concrete_puzzle.puzzle = new_puzzle;
        self.scramble_from_concrete();
    }

    pub fn undo(&mut self) {
        if let Some(((ray, order), grips)) = self.twists.pop() {
            self.undid_twists.push(((ray, order), grips.clone()));
            // we want the animation this time
            self.multi_layer_twist((ray, -order), &grips);
        } else {
            // no undo left
        }
    }

    pub fn redo(&mut self) {
        if let Some(((ray, order), grips)) = self.undid_twists.pop() {
            self.twists.push(((ray, order), grips.clone()));
            // we want the animation this time
            self.multi_layer_twist((ray, order), &grips);
        } else {
            // no redo left
        }
    }

    pub fn do_inverse(&mut self) {
        if let Some(((ray, order), grips)) = self.twists.pop() {
            self.twists.push(((ray, -order), grips.clone()));
            self.undid_twists = vec![];
            // we want the animation this time
            self.multi_layer_twist((ray, -order), &grips);
            self.multi_layer_twist((ray, -order), &grips); // do it again
        } else {
            // no undo left
        }
    }
}

pub enum CubePuzzle {
    Nnn(i8),
}

pub enum SessionSeed {
    Cube(CubePuzzle),
}

pub enum SessionEnum {
    Cube(CubePuzzle, Session<CubeRay>),
}

impl SessionSeed {
    pub fn make_session_enum(
        self,
        window_size: (u32, u32),
        context: &three_d::Context,
    ) -> SessionEnum {
        match self {
            SessionSeed::Cube(ps @ CubePuzzle::Nnn(n)) => SessionEnum::Cube(
                ps,
                Session::from_concrete(make_concrete_puzzle(
                    window_size,
                    &context,
                    render::cube::nnn_seeds(n),
                )),
            ),
        }
    }
}
