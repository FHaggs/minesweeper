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

/// Valor de cada variável durante a busca, indexado por `Var::index()`.
/// `None` = ainda sem valor. (Pode virar um enum próprio se preferir.)
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
#[cfg_attr(cfg, must_use)]
fn assign(assignment: &mut Assignment, lit: Lit) -> bool {
    // Três casos, olhando `lit_value(assignment, lit)`:
    //   Some(true)  -> já está certo, nada a fazer
    //   Some(false) -> contradição
    //   None        -> grava o valor
    todo!()
}

/// Classifica a cláusula olhando o valor de cada literal (regra 1).
fn clause_state(assignment: &Assignment, clause: &[Lit]) -> ClauseState {
    // Percorra os literais contando quantos estão sem valor e guardando o
    // último deles. Um literal verdadeiro já decide: Satisfied.
    // No fim: 0 sem valor -> Conflict, 1 -> Unit(ele), 2+ -> Unresolved.
    // Cuidado: cláusula vazia é Conflict.
    todo!()
}

/// Propagação unitária até ponto fixo. Retorna `false` se encontrar conflito.
fn propagate(cnf: &Cnf, assignment: &mut Assignment) -> bool {
    // loop {
    //   changed = false
    //   para cada cláusula em `cnf.clauses()`:
    //     Conflict  -> return false
    //     Unit(lit) -> assign(lit), changed = true
    //     resto     -> segue
    //   se !changed -> return true
    // }
    // Uma passada só não basta: uma atribuição pode tornar unitária uma
    // cláusula que já foi visitada nesta mesma passada.
    todo!()
}

/// DPLL recursivo. Some(atribuição que satisfaz tudo) ou None se Unsat.
/// Recebe a atribuição por valor: cada nível da recursão tem sua cópia,
/// então "voltar atrás" é só descartar a cópia.
fn search(cnf: &Cnf, mut assignment: Assignment) -> Option<Assignment> {
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
    todo!()
}
        