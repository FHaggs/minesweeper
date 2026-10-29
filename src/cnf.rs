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
#[derive(Debug)]
pub enum Conflict {
    /// Literal que causou a contradição.
    Lit(Lit),
    /// Cláusula vazia.
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
        todo!()
    }
}

/// Regra 1: valor do literal = valor da variável, invertido se negado.
/// `None` se a variável ainda não tem valor.
fn lit_value(assignment: &Assignment, lit: Lit) -> Option<bool> {
    // Use `lit.var().index()` para achar a posição e `lit.is_neg()` para o sinal.
    let index = lit.var().index();
    let var_value = assignment[index];
    match var_value {
        Some(v) => Some(v != lit.is_neg()),
        None => None,
    }
}

/// Regra 2: torna `lit` verdadeiro, ou seja, `lit.var()` recebe `!lit.is_neg()`.
/// Retorna `false` se `lit` já era falso (contradição); `true` caso contrário.
fn assign(assignment: Assignment, lit: Lit) -> Result<Assignment, Conflict> {
    // Três casos, olhando `lit_value(assignment, lit)`:
    //   Some(true)  -> já está certo, nada a fazer
    //   Some(false) -> contradição
    //   None        -> grava o valor
    let index = lit.var().index();
    match lit_value(&assignment, lit) {
        Some(true) => Ok(assignment),
        Some(false) => Err(Conflict::Lit(lit)),
        None => {
            let mut new_assignment = assignment;
            new_assignment[index] = Some(!lit.is_neg());
            Ok(new_assignment)
        }
    }
}

/// Classifica a cláusula olhando o valor de cada literal (regra 1).
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
    // Percorra os literais contando quantos estão sem valor e guardando o
    // último deles. Um literal verdadeiro já decide: Satisfied.
    // No fim: 0 sem valor -> Conflict, 1 -> Unit(ele), 2+ -> Unresolved.
    // Cuidado: cláusula vazia é Conflict.
    match unassigned_count {
        0 => ClauseState::Conflict,
        1 => ClauseState::Unit(last_unassigned_lit.unwrap()),
        _ => ClauseState::Unresolved,
    }
}

/// Propagação unitária até ponto fixo. Retorna `false` se encontrar conflito.
fn propagate(cnf: &Cnf, mut assignment: Assignment) -> Result<Assignment, Conflict> {
    loop {
        let mut changed = false;
        for clause in cnf.clauses()  {
            match clause_state(&assignment, clause) {
                ClauseState::Satisfied => continue,
                ClauseState::Conflict => return Err(Conflict::EmptyClause),
                ClauseState::Unit(lit) => {
                    let result = assign(assignment, lit);
                    match result {
                        Ok(new_assignment) => {
                            assignment = new_assignment;
                            changed = true;
                        }
                        Err(conflict) => return Err(conflict),
                    }
                },
                ClauseState::Unresolved => continue,
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
    // 1. `propagate`. Conflito -> None.
    //
    // 2. Procure uma cláusula que não esteja Satisfied.
    //    Nenhuma -> Some(assignment): achou um modelo.
    //
    // 3. Dessa cláusula, pegue um literal sem valor: essa é a decisão.
    //    (Depois de `propagate` ela não é Conflict nem Unit, então existe
    //    pelo menos um literal livre.)
    //
    // 4. Ramo 1: clone, `assign(decisão)`, `search` recursivo.
    //    Se deu Some -> retorna.
    //
    // 5. Ramo 2: `assign(!decisão)` na atribuição original, `search`.
    //    Retorna o que vier (Some ou None).
    // todo!()
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

        // Próximos passos, conforme for implementando:
        //   assign(b.neg()) e conferir assignment[b.index()]
        //   clause_state da cláusula 2 com a = V -> Unit(¬b)
        //   propagate -> b = F, c = V
        //   NaiveSolver.solve(&cnf, &[]) -> Sat com a = V, b = F, c = V
        //   NaiveSolver.solve(&cnf, &[b.pos()]) -> Unsat (b é segura)
    }
}
