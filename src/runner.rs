use crate::game::{Game, MoveError, Status};
use crate::solver::{Move, Solver};

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub steps: usize,
    /// `Move::Reveal` aplicados (aberturas que o solver afirmou serem seguras).
    pub reveals: usize,
    pub flags: usize,
    pub guesses: usize,
    /// Bandeiras colocadas em células sem mina: indica bug na dedução.
    pub wrong_flags: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Won,
    /// Perdeu num chute: azar.
    LostOnGuess,
    /// Perdeu num `Move::Reveal`: o solver "provou" algo falso.
    LostOnReveal,
    GaveUp,
    Invalid(MoveError),
    Aborted,
}

/// Roda o jogo até o fim. `on_step` é chamado após cada jogada e pode
/// retornar `false` para interromper.
pub fn play(
    game: &mut Game,
    solver: &mut dyn Solver,
    mut on_step: impl FnMut(&Game, &Stats, Move) -> bool,
) -> (Outcome, Stats) {
    let mut stats = Stats::default();
    loop {
        let Some(mv) = solver.next_move(game.view()) else {
            return (Outcome::GaveUp, stats);
        };
        let status = match game.apply(mv) {
            Ok(s) => s,
            Err(e) => return (Outcome::Invalid(e), stats),
        };

        stats.steps += 1;
        match mv {
            Move::Reveal(_) => stats.reveals += 1,
            Move::Guess(_) => stats.guesses += 1,
            Move::Flag(p) => {
                stats.flags += 1;
                if !game.is_mine(p) {
                    stats.wrong_flags += 1;
                }
            }
        }

        if !on_step(game, &stats, mv) {
            return (Outcome::Aborted, stats);
        }
        match status {
            Status::Playing => {}
            Status::Won => return (Outcome::Won, stats),
            Status::Lost(_) => {
                let outcome = match mv {
                    Move::Reveal(_) => Outcome::LostOnReveal,
                    _ => Outcome::LostOnGuess,
                };
                return (outcome, stats);
            }
        }
    }
}
