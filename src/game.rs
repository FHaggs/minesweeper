use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::solver::Move;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

impl Pos {
    pub fn new(row: usize, col: usize) -> Self {
        Pos { row, col }
    }
}

/// O que o jogador (e o solver) enxerga de uma célula.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    Hidden,
    Flagged,
    /// Célula aberta com o número de minas vizinhas (0 = vazia).
    Revealed(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Won,
    Lost(Pos),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MoveError {
    OutOfBounds(Pos),
    AlreadyRevealed(Pos),
    AlreadyFlagged(Pos),
    GameOver,
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::OutOfBounds(p) => write!(f, "({}, {}) fora do tabuleiro", p.row, p.col),
            MoveError::AlreadyRevealed(p) => write!(f, "({}, {}) já está aberta", p.row, p.col),
            MoveError::AlreadyFlagged(p) => write!(f, "({}, {}) já tem bandeira", p.row, p.col),
            MoveError::GameOver => write!(f, "o jogo já acabou"),
        }
    }
}

/// Visão do tabuleiro sem a posição das minas. É tudo que o solver recebe.
#[derive(Clone, Debug)]
pub struct View {
    rows: usize,
    cols: usize,
    total_mines: usize,
    tiles: Vec<Tile>,
}

impl View {
    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn total_mines(&self) -> usize {
        self.total_mines
    }

    pub fn get(&self, pos: Pos) -> Tile {
        self.tiles[self.index(pos)]
    }

    /// Índice linear `row * cols + col`, em `0..rows*cols`.
    /// Útil para mapear células em variáveis do CNF (ex.: `var = index + 1`).
    pub fn index(&self, pos: Pos) -> usize {
        pos.row * self.cols + pos.col
    }

    pub fn pos(&self, index: usize) -> Pos {
        Pos::new(index / self.cols, index % self.cols)
    }

    pub fn in_bounds(&self, pos: Pos) -> bool {
        pos.row < self.rows && pos.col < self.cols
    }

    pub fn positions(&self) -> impl Iterator<Item = Pos> + '_ {
        (0..self.rows * self.cols).map(|i| self.pos(i))
    }

    /// As até 8 células adjacentes a `pos`.
    pub fn neighbors(&self, pos: Pos) -> impl Iterator<Item = Pos> + use<> {
        neighbors(self.rows, self.cols, pos)
    }

    pub fn flag_count(&self) -> usize {
        self.tiles.iter().filter(|t| **t == Tile::Flagged).count()
    }

    /// Células ainda não abertas (inclui as com bandeira).
    pub fn unrevealed_count(&self) -> usize {
        self.tiles.iter().filter(|t| !matches!(t, Tile::Revealed(_))).count()
    }

    /// Nada foi aberto ainda (o primeiro clique é sempre seguro).
    pub fn is_untouched(&self) -> bool {
        !self.tiles.iter().any(|t| matches!(t, Tile::Revealed(_)))
    }
}

fn neighbors(rows: usize, cols: usize, pos: Pos) -> impl Iterator<Item = Pos> {
    let r0 = pos.row.saturating_sub(1);
    let c0 = pos.col.saturating_sub(1);
    let r1 = (pos.row + 1).min(rows - 1);
    let c1 = (pos.col + 1).min(cols - 1);
    (r0..=r1)
        .flat_map(move |r| (c0..=c1).map(move |c| Pos::new(r, c)))
        .filter(move |p| *p != pos)
}

pub struct Game {
    view: View,
    mines: Vec<bool>,
    placed: bool,
    status: Status,
    rng: StdRng,
}

impl Game {
    /// As minas só são sorteadas no primeiro `Reveal`, garantindo que a primeira
    /// célula aberta (e, se couber, sua vizinhança) não tenha mina.
    pub fn new(rows: usize, cols: usize, mines: usize, seed: u64) -> Self {
        assert!(rows > 0 && cols > 0, "tabuleiro vazio");
        assert!(mines < rows * cols, "minas demais para o tabuleiro");
        Game {
            view: View {
                rows,
                cols,
                total_mines: mines,
                tiles: vec![Tile::Hidden; rows * cols],
            },
            mines: vec![false; rows * cols],
            placed: false,
            status: Status::Playing,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// Só para a UI mostrar o tabuleiro no fim do jogo. Não passe isso ao solver.
    pub fn is_mine(&self, pos: Pos) -> bool {
        self.placed && self.mines[self.view.index(pos)]
    }

    pub fn apply(&mut self, mv: Move) -> Result<Status, MoveError> {
        if self.status != Status::Playing {
            return Err(MoveError::GameOver);
        }
        let pos = mv.pos();
        if !self.view.in_bounds(pos) {
            return Err(MoveError::OutOfBounds(pos));
        }
        match (mv, self.view.get(pos)) {
            (_, Tile::Revealed(_)) => Err(MoveError::AlreadyRevealed(pos)),
            (_, Tile::Flagged) => Err(MoveError::AlreadyFlagged(pos)),
            (Move::Flag(_), Tile::Hidden) => {
                let i = self.view.index(pos);
                self.view.tiles[i] = Tile::Flagged;
                Ok(self.status)
            }
            (Move::Reveal(_) | Move::Guess(_), Tile::Hidden) => {
                self.reveal(pos);
                Ok(self.status)
            }
        }
    }

    fn reveal(&mut self, start: Pos) {
        if !self.placed {
            self.place_mines(start);
        }
        if self.mines[self.view.index(start)] {
            self.status = Status::Lost(start);
            return;
        }

        let mut stack = vec![start];
        while let Some(pos) = stack.pop() {
            let i = self.view.index(pos);
            if self.view.tiles[i] != Tile::Hidden {
                continue;
            }
            let n = self.count_mines_around(pos);
            self.view.tiles[i] = Tile::Revealed(n);
            if n == 0 {
                stack.extend(self.view.neighbors(pos));
            }
        }

        let safe_cells = self.view.rows * self.view.cols - self.view.total_mines;
        let revealed = self.view.rows * self.view.cols - self.view.unrevealed_count();
        if revealed == safe_cells {
            self.status = Status::Won;
        }
    }

    fn place_mines(&mut self, first: Pos) {
        let total = self.view.rows * self.view.cols;
        let mut excluded: Vec<Pos> = self.view.neighbors(first).collect();
        excluded.push(first);
        // Se não couber deixar a vizinhança livre, protege só a célula clicada.
        if total - excluded.len() < self.view.total_mines {
            excluded = vec![first];
        }

        let mut candidates: Vec<usize> = (0..total)
            .filter(|i| !excluded.contains(&self.view.pos(*i)))
            .collect();
        candidates.shuffle(&mut self.rng);
        for &i in candidates.iter().take(self.view.total_mines) {
            self.mines[i] = true;
        }
        self.placed = true;
    }

    fn count_mines_around(&self, pos: Pos) -> u8 {
        self.view
            .neighbors(pos)
            .filter(|p| self.mines[self.view.index(*p)])
            .count() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_reveal_is_safe_and_opens_area() {
        for seed in 0..50 {
            let mut game = Game::new(9, 9, 10, seed);
            let pos = Pos::new(4, 4);
            assert_ne!(game.apply(Move::Reveal(pos)), Ok(Status::Lost(pos)));
            assert_eq!(game.view().get(pos), Tile::Revealed(0));
            assert_eq!(game.mines.iter().filter(|m| **m).count(), 10);
        }
    }

    #[test]
    fn numbers_match_mines() {
        let mut game = Game::new(16, 30, 99, 7);
        game.apply(Move::Reveal(Pos::new(0, 0))).unwrap();
        for pos in game.view().positions() {
            if let Tile::Revealed(n) = game.view().get(pos) {
                assert!(!game.is_mine(pos));
                let real = game.view().neighbors(pos).filter(|p| game.is_mine(*p)).count();
                assert_eq!(n as usize, real);
            }
        }
    }

    #[test]
    fn revealing_every_safe_cell_wins() {
        let mut game = Game::new(5, 5, 3, 1);
        game.apply(Move::Reveal(Pos::new(2, 2))).unwrap();
        let safe: Vec<Pos> = game.view().positions().filter(|p| !game.is_mine(*p)).collect();
        for pos in safe {
            if game.view().get(pos) == Tile::Hidden {
                game.apply(Move::Reveal(pos)).unwrap();
            }
        }
        assert_eq!(game.status(), Status::Won);
    }

    #[test]
    fn invalid_moves_are_rejected() {
        let mut game = Game::new(5, 5, 3, 1);
        let pos = Pos::new(2, 2);
        game.apply(Move::Reveal(pos)).unwrap();
        assert_eq!(game.apply(Move::Reveal(pos)), Err(MoveError::AlreadyRevealed(pos)));
        assert_eq!(
            game.apply(Move::Flag(Pos::new(9, 9))),
            Err(MoveError::OutOfBounds(Pos::new(9, 9)))
        );
    }

    #[test]
    fn neighbors_at_corner() {
        let n: Vec<Pos> = neighbors(3, 3, Pos::new(0, 0)).collect();
        assert_eq!(n, vec![Pos::new(0, 1), Pos::new(1, 0), Pos::new(1, 1)]);
    }
}
