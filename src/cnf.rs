use crate::lit::{Lit, Var};

#[derive(Default, Clone, Debug)]
pub struct Cnf {
    num_vars: u32,
    clauses: Vec<Vec<Lit>>,
}

impl Cnf {
    pub fn new_var(&mut self) -> Var {
        let v = Var(self.num_vars);
        self.num_vars += 1;
        v
    }
    pub fn num_vars(&self) -> usize { self.num_vars as usize }
    pub fn clauses(&self) -> &[Vec<Lit>] { &self.clauses }

    pub fn add_clause(&mut self, lits: impl IntoIterator<Item = Lit>) {
        self.clauses.push(lits.into_iter().collect());
    }
}

pub struct Model(Vec<bool>);

impl Model {
    pub fn value(&self, v: Var) -> bool { self.0[v.index()] }
    pub fn lit(&self, l: Lit) -> bool { self.value(l.var()) != l.is_neg() }
}

pub enum SatResult {
    Sat(Model),
    Unsat,
}

pub trait SatSolver {
    /// `assumptions` são literais forçados só nesta chamada.
    fn solve(&mut self, cnf: &Cnf, assumptions: &[Lit]) -> SatResult;
}