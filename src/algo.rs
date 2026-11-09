use crate::cnf::{Model, NaiveSolver, SatResult, SatSolver};
use crate::encode::{self, Encoding};
use crate::game::{Pos, Tile, View};
use crate::solver::{Move, Solver};

pub struct CnfSolver {
    sat: NaiveSolver,
    /// Jogadas já provadas e ainda não feitas. Continuam válidas: o tabuleiro
    /// só ganha informação, nunca perde.
    pending: Vec<Move>,
}

impl CnfSolver {
    pub fn new() -> Self {
        CnfSolver { sat: NaiveSolver, pending: vec![] }
    }

    /// Prova o que der em cada região. Truque: cada modelo encontrado mostra
    /// valores possíveis. Se uma célula já apareceu com mina num modelo, ela
    /// não pode ser segura, então nem precisa testar `[x]` (e vice-versa).
    fn deduce(&mut self, enc: &Encoding) -> Vec<Move> {
        let n = enc.cnf.num_vars();
        let mut seen_mine = vec![false; n];
        let mut seen_safe = vec![false; n];
        let record = |m: &Model, seen_mine: &mut [bool], seen_safe: &mut [bool]| {
            for &(_, x) in &enc.cells {
                if m.value(x) {
                    seen_mine[x.index()] = true;
                } else {
                    seen_safe[x.index()] = true;
                }
            }
        };

        match self.sat.solve(&enc.cnf, &[]) {
            SatResult::Sat(m) => record(&m, &mut seen_mine, &mut seen_safe),
            // Board inconsistente (só com bandeira errada): não deduz nada.
            SatResult::Unsat => return vec![],
        }

        let mut moves = vec![];
        for &(pos, x) in &enc.cells {
            if !seen_mine[x.index()] {
                match self.sat.solve(&enc.cnf, &[x.pos()]) {
                    SatResult::Unsat => {
                        moves.push(Move::Reveal(pos));
                        continue;
                    }
                    SatResult::Sat(m) => record(&m, &mut seen_mine, &mut seen_safe),
                }
            }
            if !seen_safe[x.index()] {
                match self.sat.solve(&enc.cnf, &[x.neg()]) {
                    SatResult::Unsat => moves.push(Move::Flag(pos)),
                    SatResult::Sat(m) => record(&m, &mut seen_mine, &mut seen_safe),
                }
            }
        }
        moves
    }
}

impl Solver for CnfSolver {
    fn name(&self) -> &str {
        "cnf"
    }

    fn next_move(&mut self, view: &View) -> Option<Move> {
        if view.is_untouched() {
            // O primeiro clique é sempre seguro.
            return Some(Move::Reveal(Pos::new(view.rows() / 2, view.cols() / 2)));
        }

        self.pending.retain(|m| view.get(m.pos()) == Tile::Hidden);
        if self.pending.is_empty() {
            self.pending = encode::encode(view)
                .iter()
                .flat_map(|enc| self.deduce(enc))
                .collect();
            // pop() tira do fim: inverte para jogar na ordem do tabuleiro.
            self.pending.reverse();
        }
        self.pending.pop().or_else(|| guess(view))
    }
}

/// Nada provado: chuta a célula com menor risco estimado.
/// - fronteira: pior razão minas/células entre os números vizinhos;
/// - resto: densidade de minas restantes no tabuleiro.
fn guess(view: &View) -> Option<Move> {
    let constraints = encode::constraints(view);
    let hidden: Vec<Pos> = view.positions().filter(|&p| view.get(p) == Tile::Hidden).collect();
    let remaining = view.total_mines().saturating_sub(view.flag_count());
    let density = remaining as f64 / hidden.len().max(1) as f64;

    let risk = |p: Pos| {
        constraints
            .iter()
            .filter(|c| c.cells.contains(&p))
            .map(|c| c.mines as f64 / c.cells.len() as f64)
            .reduce(f64::max)
            .unwrap_or(density)
    };

    hidden
        .into_iter()
        .min_by(|&a, &b| risk(a).total_cmp(&risk(b)))
        .map(Move::Guess)
}
