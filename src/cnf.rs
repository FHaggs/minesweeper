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
    pub fn num_vars(&self) -> usize {
        self.num_vars as usize
    }
    pub fn clauses(&self) -> &[Vec<Lit>] {
        &self.clauses
    }

    pub fn add_clause(&mut self, lits: impl IntoIterator<Item = Lit>) {
        self.clauses.push(lits.into_iter().collect());
    }
}

pub struct Model(Vec<bool>);

impl Model {
    pub fn value(&self, v: Var) -> bool {
        self.0[v.index()]
    }
    pub fn lit(&self, l: Lit) -> bool {
        self.value(l.var()) != l.is_neg()
    }
}

pub enum SatResult {
    Sat(Model),
    Unsat,
}

pub trait SatSolver {
    /// `assumptions` são literais forçados só nesta chamada.
    fn solve(&mut self, cnf: &Cnf, assumptions: &[Lit]) -> SatResult;
}

/// Motivo pelo qual uma atribuição ficou impossível.
#[derive(Debug)]
pub enum Conflict {
    /// Tentou tornar verdadeiro um literal que já era falso.
    Lit(Lit),
    /// Uma cláusula ficou com todos os literais falsos.
    EmptyClause,
}

/// Valor de cada variável durante a busca, indexado por `Var::index()`.
/// `None` = ainda sem valor.
type Assignment = Vec<Option<bool>>;

#[derive(Debug, PartialEq, Eq)]
enum ClauseState {
    /// Algum literal é verdadeiro.
    Satisfied,
    /// Todos os literais são falsos.
    Conflict,
    /// Todos falsos menos este, que está sem valor: ele precisa virar verdadeiro.
    Unit(Lit),
    /// Dois ou mais literais sem valor e nenhum verdadeiro.
    Unresolved,
}

pub struct NaiveSolver;

impl SatSolver for NaiveSolver {
    fn solve(&mut self, cnf: &Cnf, assumptions: &[Lit]) -> SatResult {
        // 1. Atribuição inicial: `num_vars()` posições, todas `None`.
        //
        // 2. Para cada assumption, `assign`. Se alguma contradiz outra
        //    (ex.: [x1, ¬x1]), já é Unsat.
        //
        // 3. `search` a partir dessa atribuição.
        //
        // 4. Some(atribuição) -> Sat(Model(...)): troque cada `None` que
        //    sobrou por `false` (todas as cláusulas já estão satisfeitas,
        //    então tanto faz). None -> Unsat.

        let mut assignment: Assignment = vec![None; cnf.num_vars()];
        for &lit in assumptions {
            match assign(assignment.clone(), lit) {
                Ok(new_assignment) => assignment = new_assignment,
                Err(_) => return SatResult::Unsat,
            }
        }
        let result = search(cnf, assignment);
        match result {
            Some(assignments) => {
                let model = Model(
                    assignments
                        .into_iter()
                        .map(|v| v.unwrap_or(false))
                        .collect(),
                );
                SatResult::Sat(model)
            }
            None => SatResult::Unsat,
        }
    }
}

/// Regra 1: valor do literal = valor da variável, invertido se negado.
/// `None` se a variável ainda não tem valor.
fn lit_value(assignment: &Assignment, lit: Lit) -> Option<bool> {
    let index = lit.var().index();
    let var_value = assignment[index];
    match var_value {
        Some(v) => Some(v != lit.is_neg()),
        None => None,
    }
}

/// Regra 2: torna `lit` verdadeiro, ou seja, `lit.var()` recebe `!lit.is_neg()`.
/// Se `lit` já era verdadeiro, devolve a atribuição sem mudanças.
/// `Err(Conflict::Lit)` se `lit` já era falso.
fn assign(mut assignment: Assignment, lit: Lit) -> Result<Assignment, Conflict> {
    let index = lit.var().index();
    match lit_value(&assignment, lit) {
        Some(true) => Ok(assignment),
        Some(false) => Err(Conflict::Lit(lit)),
        None => {
            assignment[index] = Some(!lit.is_neg());
            Ok(assignment)
        }
    }
}

/// Classifica a cláusula olhando o valor de cada literal (regra 1).
/// Só dá para concluir `Unresolved` no fim: um literal mais adiante ainda
/// pode ser verdadeiro. Cláusula vazia é `Conflict`.
fn clause_state(assignment: &Assignment, clause: &[Lit]) -> ClauseState {
    let mut unassigned_count: u32 = 0;
    let mut last_unassigned_lit = None;
    for lit in clause {
        match lit_value(assignment, *lit) {
            Some(true) => return ClauseState::Satisfied,
            Some(false) => continue,
            None => {
                unassigned_count += 1;
                last_unassigned_lit = Some(*lit);
            }
        }
    }
    match unassigned_count {
        0 => ClauseState::Conflict,
        1 => ClauseState::Unit(
            last_unassigned_lit
                .expect("last_unassigned_lit should be Some if unassigned_count is 1"),
        ),
        _ => ClauseState::Unresolved,
    }
}

/// Propagação unitária até ponto fixo: repete as passadas até que nenhuma
/// cláusula unitária reste, pois cada atribuição pode tornar unitária uma
/// cláusula já visitada. `Err` se alguma cláusula ficar toda falsa.
fn propagate(cnf: &Cnf, mut assignment: Assignment) -> Result<Assignment, Conflict> {
    loop {
        let mut changed = false;
        for clause in cnf.clauses() {
            match clause_state(&assignment, clause) {
                ClauseState::Conflict => return Err(Conflict::EmptyClause),
                ClauseState::Unit(lit) => {
                    assignment = assign(assignment, lit)?;
                    changed = true;
                }
                _ => continue,
            }
        }
        if !changed {
            return Ok(assignment);
        }
    }
}

/// DPLL recursivo. Some(atribuição que satisfaz tudo) ou None se Unsat.
/// Recebe a atribuição por valor: cada nível da recursão tem sua cópia,
/// então "voltar atrás" é só descartar a cópia.
///
/// Cada chamada decide só a primeira variável livre; as demais ficam para as
/// chamadas mais fundas. Saídas:
///   - conflito na propagação           -> None
///   - nenhuma variável livre           -> Some (tudo atribuído, sem conflito)
///   - um dos ramos achou solução       -> Some
///   - os dois ramos falharam           -> None
fn search(cnf: &Cnf, assignments: Assignment) -> Option<Assignment> {
    let assignments = match propagate(cnf, assignments) {
        Ok(assignments) => assignments,
        Err(_) => return None,
    };

    for (i, assignment) in assignments.iter().enumerate() {
        if assignment.is_none() {
            let decision = Var(i as u32).pos();

            let assign_pos = assign(assignments.clone(), decision);
            if let Ok(assign_pos_res) = assign_pos {
                if let Some(result) = search(cnf, assign_pos_res) {
                    return Some(result);
                }
            }

            let assign_neg = assign(assignments.clone(), !decision);
            if let Ok(assign_neg_res) = assign_neg {
                if let Some(result) = search(cnf, assign_neg_res) {
                    return Some(result);
                }
            }
            return None;
        }
    }
    return Some(assignments);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Padrão 1-2-1. Única solução: a = V, b = F, c = V.
    ///   1. (a ∨ b)
    ///   2. (¬a ∨ ¬b)
    ///   3. (¬a ∨ ¬b ∨ ¬c)
    ///   4. (a ∨ c)
    ///   5. (b ∨ c)
    ///   6. (¬b ∨ ¬c)
    fn one_two_one() -> (Cnf, Var, Var, Var) {
        let mut cnf = Cnf::default();
        let (a, b, c) = (cnf.new_var(), cnf.new_var(), cnf.new_var());
        cnf.add_clause([a.pos(), b.pos()]);
        cnf.add_clause([a.neg(), b.neg()]);
        cnf.add_clause([a.neg(), b.neg(), c.neg()]);
        cnf.add_clause([a.pos(), c.pos()]);
        cnf.add_clause([b.pos(), c.pos()]);
        cnf.add_clause([b.neg(), c.neg()]);
        (cnf, a, b, c)
    }

    #[test]
    fn baseline() {
        let (cnf, a, b, c) = one_two_one();
        println!("{:?}", cnf.clauses());

        // Atribuição parcial: só a = V.
        let mut assignment: Assignment = vec![None; cnf.num_vars()];
        assignment[a.index()] = Some(true);
        assert_eq!(lit_value(&assignment, a.pos()), Some(true));
        assert_eq!(lit_value(&assignment, a.neg()), Some(false));
        assert_eq!(lit_value(&assignment, b.pos()), None);
        assert_eq!(lit_value(&assignment, !c.neg()), None);

        let res = search(&cnf, assignment);

        println!("Result: {:?}", res);

        // Próximos passos:
        //   trocar o println acima por assert_eq! com a = V, b = F, c = V
        //   search partindo de b = V -> None (b é segura)
        //   NaiveSolver.solve(&cnf, &[]) -> Sat com a = V, b = F, c = V
        //   NaiveSolver.solve(&cnf, &[b.pos()]) -> Unsat (b é segura)
    }
}
