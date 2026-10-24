use std::fmt::Write;

use crate::game::{Game, Pos, Status, Tile};
use crate::runner::Stats;
use crate::solver::Move;

const RESET: &str = "\x1b[0m";

pub fn clear_screen() {
    print!("\x1b[2J");
}

/// Desenha o tabuleiro inteiro, voltando o cursor para o topo (sem piscar).
pub fn draw(game: &Game, stats: &Stats, last: Option<Move>, title: &str) {
    let mut out = String::from("\x1b[H");
    let view = game.view();
    let over = game.status() != Status::Playing;

    writeln!(out, "\x1b[1m{title}\x1b[0m\x1b[K").unwrap();
    writeln!(out, "\x1b[K").unwrap();

    for row in 0..view.rows() {
        out.push_str("  ");
        for col in 0..view.cols() {
            let pos = Pos::new(row, col);
            let (fg, symbol) = cell_style(game, pos, over);
            let bg = match last {
                Some(mv) if mv.pos() == pos => match mv {
                    Move::Reveal(_) => "\x1b[42m",
                    Move::Flag(_) => "\x1b[43m",
                    Move::Guess(_) => "\x1b[45m",
                },
                _ => "",
            };
            write!(out, "{bg}{fg}{symbol}{RESET} ").unwrap();
        }
        out.push_str("\x1b[K\n");
    }

    writeln!(out, "\x1b[K").unwrap();
    writeln!(
        out,
        "  jogadas {:>4} | deduzidas {:>4} | bandeiras {:>3}/{:<3} | chutes {:>3}\x1b[K",
        stats.steps,
        stats.reveals,
        view.flag_count(),
        view.total_mines(),
        stats.guesses
    )
    .unwrap();
    writeln!(
        out,
        "  última: \x1b[42m  {RESET} dedução  \x1b[43m  {RESET} bandeira  \x1b[45m  {RESET} chute\x1b[K"
    )
    .unwrap();
    out.push_str("\x1b[J");
    print!("{out}");
}

fn cell_style(game: &Game, pos: Pos, over: bool) -> (&'static str, char) {
    let mine = game.is_mine(pos);
    match game.view().get(pos) {
        Tile::Hidden if over && mine => {
            if game.status() == Status::Lost(pos) {
                ("\x1b[1;97;41m", '*')
            } else {
                ("\x1b[91m", '*')
            }
        }
        Tile::Hidden => ("\x1b[90m", '■'),
        Tile::Flagged if over && !mine => ("\x1b[1;93m", 'X'),
        Tile::Flagged => ("\x1b[1;91m", 'F'),
        Tile::Revealed(0) => ("\x1b[90m", '·'),
        Tile::Revealed(n) => {
            let color = match n {
                1 => "\x1b[94m",
                2 => "\x1b[32m",
                3 => "\x1b[91m",
                4 => "\x1b[34m",
                5 => "\x1b[31m",
                6 => "\x1b[36m",
                7 => "\x1b[35m",
                _ => "\x1b[37m",
            };
            (color, char::from(b'0' + n))
        }
    }
}
