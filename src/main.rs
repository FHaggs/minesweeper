mod algo;
mod game;
mod runner;
mod solver;
mod ui;

use std::io::BufRead;
use std::time::{Duration, Instant};

use game::Game;
use runner::Outcome;
use solver::Solver;

const USAGE: &str = "\
uso: minesweeper [opções]

  --preset NOME      beginner (9x9/10), intermediate (16x16/40), expert (16x30/99) [padrão: expert]
  --rows N           linhas
  --cols N           colunas
  --mines N          número de minas
  --seed N           semente do tabuleiro [padrão: aleatória]
  --solver NOME      cnf | random [padrão: cnf]
  --delay MS         pausa entre jogadas na visualização [padrão: 80]
  --step             avança uma jogada por Enter (q + Enter sai)
  --bench N          roda N jogos sem UI e mostra estatísticas
";

struct Config {
    rows: usize,
    cols: usize,
    mines: usize,
    seed: u64,
    solver: String,
    delay: Duration,
    step: bool,
    bench: Option<usize>,
}

fn parse_args() -> Result<Config, String> {
    let mut cfg = Config {
        rows: 16,
        cols: 30,
        mines: 99,
        seed: rand::random(),
        solver: "cnf".into(),
        delay: Duration::from_millis(80),
        step: false,
        bench: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} precisa de um valor"));
        let num = |s: String| s.parse::<u64>().map_err(|_| format!("número inválido: {s}"));
        match arg.as_str() {
            "--preset" => {
                (cfg.rows, cfg.cols, cfg.mines) = match value()?.as_str() {
                    "beginner" => (9, 9, 10),
                    "intermediate" => (16, 16, 40),
                    "expert" => (16, 30, 99),
                    p => return Err(format!("preset desconhecido: {p}")),
                }
            }
            "--rows" => cfg.rows = num(value()?)? as usize,
            "--cols" => cfg.cols = num(value()?)? as usize,
            "--mines" => cfg.mines = num(value()?)? as usize,
            "--seed" => cfg.seed = num(value()?)?,
            "--solver" => cfg.solver = value()?,
            "--delay" => cfg.delay = Duration::from_millis(num(value()?)?),
            "--step" => cfg.step = true,
            "--bench" => cfg.bench = Some(num(value()?)? as usize),
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            a => return Err(format!("opção desconhecida: {a}")),
        }
    }
    if cfg.rows == 0 || cfg.cols == 0 || cfg.mines >= cfg.rows * cfg.cols {
        return Err("tamanho/minas inválidos".into());
    }
    make_solver(&cfg.solver, 0)?;
    Ok(cfg)
}

fn make_solver(name: &str, seed: u64) -> Result<Box<dyn Solver>, String> {
    match name {
        "cnf" => Ok(Box::new(algo::CnfSolver::new())),
        "random" => Ok(Box::new(solver::RandomSolver::new(seed))),
        s => Err(format!("solver desconhecido: {s}")),
    }
}

fn main() {
    let cfg = parse_args().unwrap_or_else(|e| {
        eprint!("erro: {e}\n\n{USAGE}");
        std::process::exit(2);
    });
    match cfg.bench {
        Some(n) => bench(&cfg, n),
        None => visualize(&cfg),
    }
}

fn visualize(cfg: &Config) {
    let mut game = Game::new(cfg.rows, cfg.cols, cfg.mines, cfg.seed);
    let mut solver = make_solver(&cfg.solver, cfg.seed).unwrap();
    let title = format!(
        "{} — {}x{}, {} minas, seed {}",
        solver.name(),
        cfg.rows,
        cfg.cols,
        cfg.mines,
        cfg.seed
    );
    let stdin = std::io::stdin();

    ui::clear_screen();
    ui::draw(&game, &Default::default(), None, &title);
    let (outcome, _) = runner::play(&mut game, solver.as_mut(), |game, stats, mv| {
        ui::draw(game, stats, Some(mv), &title);
        if cfg.step {
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
            return line.trim() != "q";
        }
        std::thread::sleep(cfg.delay);
        true
    });

    println!();
    match outcome {
        Outcome::Won => println!("  \x1b[1;32mVenceu!\x1b[0m"),
        Outcome::LostOnGuess => println!("  \x1b[1;31mPerdeu num chute.\x1b[0m"),
        Outcome::LostOnReveal => {
            println!("  \x1b[1;31mPerdeu numa jogada 'segura' — bug na dedução!\x1b[0m")
        }
        Outcome::GaveUp => println!("  O solver desistiu."),
        Outcome::Invalid(e) => println!("  \x1b[1;31mJogada inválida: {e}\x1b[0m"),
        Outcome::Aborted => println!("  Interrompido."),
    }
}

fn bench(cfg: &Config, n: usize) {
    let mut wins = 0;
    let mut lost_guess = 0;
    let mut lost_reveal = 0;
    let mut gave_up = 0;
    let mut invalid = 0;
    let mut guesses = 0;
    let mut wrong_flags = 0;
    let start = Instant::now();

    for i in 0..n {
        let seed = cfg.seed.wrapping_add(i as u64);
        let mut game = Game::new(cfg.rows, cfg.cols, cfg.mines, seed);
        let mut solver = make_solver(&cfg.solver, seed).unwrap();
        let (outcome, stats) = runner::play(&mut game, solver.as_mut(), |_, _, _| true);
        guesses += stats.guesses;
        wrong_flags += stats.wrong_flags;
        match outcome {
            Outcome::Won => wins += 1,
            Outcome::LostOnGuess => lost_guess += 1,
            Outcome::LostOnReveal => {
                lost_reveal += 1;
                eprintln!("seed {seed}: perdeu numa jogada 'segura'");
            }
            Outcome::GaveUp => gave_up += 1,
            Outcome::Invalid(e) => {
                invalid += 1;
                eprintln!("seed {seed}: jogada inválida: {e}");
            }
            Outcome::Aborted => unreachable!(),
        }
    }

    let elapsed = start.elapsed();
    let pct = |x: usize| 100.0 * x as f64 / n as f64;
    println!(
        "{} — {}x{}, {} minas, {n} jogos (seeds {}..)",
        cfg.solver, cfg.rows, cfg.cols, cfg.mines, cfg.seed
    );
    println!("  vitórias               {wins:>6} ({:.1}%)", pct(wins));
    println!("  derrotas em chute      {lost_guess:>6} ({:.1}%)", pct(lost_guess));
    println!("  derrotas em dedução    {lost_reveal:>6}");
    println!("  desistências           {gave_up:>6}");
    println!("  jogadas inválidas      {invalid:>6}");
    println!("  bandeiras erradas      {wrong_flags:>6}");
    println!("  chutes por jogo        {:>6.2}", guesses as f64 / n as f64);
    println!("  tempo por jogo         {:>6.2?}", elapsed / n as u32);
}
