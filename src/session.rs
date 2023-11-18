use crate::puzzle::common::*;
use crate::render::common::*;

pub struct Session<Ray: ConcreteRaySystem> {
    pub scramble: Vec<usize>,
    pub concrete_puzzle: ConcretePuzzle<Ray>,
    pub twists: Vec<((Ray, i8), Vec<Vec<i8>>)>,
    pub undid_twists: Vec<((Ray, i8), Vec<Vec<i8>>)>,
}

impl<'a, Ray: ConcreteRaySystem> Session<Ray> {
    pub fn from_concrete(concrete_puzzle: ConcretePuzzle<Ray>) -> Session<Ray> {
        Session {
            scramble: concrete_puzzle.puzzle.permutation(),
            concrete_puzzle,
            twists: vec![],
            undid_twists: vec![],
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
