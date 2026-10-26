use crate::{cnf::Cnf, lit::Lit};

/// No máximo k dos literais são verdadeiros.
pub fn at_most_k(cnf: &mut Cnf, lits: &[Lit], k: usize) { todo!() }

/// No mínimo k = no máximo (n-k) dos negados.
pub fn at_least_k(cnf: &mut Cnf, lits: &[Lit], k: usize) {
    let negated: Vec<Lit> = lits.iter().map(|&l| !l).collect();
    at_most_k(cnf, &negated, lits.len() - k);
}

pub fn exactly_k(cnf: &mut Cnf, lits: &[Lit], k: usize) {
    at_most_k(cnf, lits, k);
    at_least_k(cnf, lits, k);
}