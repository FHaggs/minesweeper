use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::game::{Pos, Tile, View};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    /// Abrir uma célula que o solver *provou* ser segura.
    Reveal(Pos),
    /// Marcar uma célula que o solver *provou* ser mina.
    Flag(Pos),
    /// Abrir uma célula sem certeza (chute). Para o jogo é igual a `Reveal`,
    /// mas a UI e o benchmark contam separadamente.
    Guess(Pos),
}

impl Move {
    pub fn pos(self) -> Pos {
        match self {
            Move::Reveal(p) | Move::Flag(p) | Move::Guess(p) => p,
        }
    }
}

pub trait Solver {
    fn name(&self) -> &str;

    /// Chamado uma vez por jogada com o estado visível atual.
    /// Retornar `None` significa desistir.
    fn next_move(&mut self, view: &View) -> Option<Move>;
}

/// Baseline: chuta uma célula escondida qualquer.
pub struct RandomSolver {
    rng: StdRng,
}

impl RandomSolver {
    pub fn new(seed: u64) -> Self {
        RandomSolver { rng: StdRng::seed_from_u64(seed) }
    }
}

impl Solver for RandomSolver {
    fn name(&self) -> &str {
        "random"
    }

    fn next_move(&mut self, view: &View) -> Option<Move> {
        let hidden: Vec<Pos> = view.positions().filter(|p| view.get(*p) == Tile::Hidden).collect();
        hidden.choose(&mut self.rng).map(|p| Move::Guess(*p))
    }
}
