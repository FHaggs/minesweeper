use crate::game::View;
use crate::solver::{Move, Solver};

pub struct CnfSolver {}

impl CnfSolver {
    pub fn new() -> Self {
        CnfSolver {}
    }
}

impl Solver for CnfSolver {
    fn name(&self) -> &str {
        "cnf"
    }

    fn next_move(&mut self, _view: &View) -> Option<Move> {
        // TODO: seu algoritmo aqui.
        None
    }
}
