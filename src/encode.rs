use crate::cnf::Cnf;
use crate::game::{Pos, Tile, View};
use crate::lit::{Lit, Var};

/// Todas as combinações de `size` elementos de `lits`, mantendo a ordem.
fn combinations(lits: &[Lit], size: usize) -> Vec<Vec<Lit>> {
    match (size, lits) {
        (0, _) => vec![vec![]],   // grupo completo: uma combinação, a vazia
        (_, []) => vec![],        // faltam elementos: nenhuma combinação
        (_, [first, rest @ ..]) => {
            // com `first`: escolhe size-1 do resto e coloca `first` na frente
            let with_first = combinations(rest, size - 1)
                .into_iter()
                .map(|c| std::iter::once(*first).chain(c).collect());
            // sem `first`: escolhe size do resto
            let without_first = combinations(rest, size);
            with_first.chain(without_first).collect()
        }
    }
}


/// Cláusulas de "no máximo k dos literais são verdadeiros".
///
/// Codificação binomial: para cada subconjunto de k+1 literais, pelo menos
/// um é falso -> cláusula (¬l1 ∨ ... ∨ ¬l(k+1)). Sem variáveis auxiliares,
/// por isso é pura. Só serve para listas pequenas (vizinhança, ≤ 8 literais):
/// C(8, 4) = 70 cláusulas no pior caso.
///
/// k >= lits.len() -> nenhuma cláusula (sempre vale).
pub fn at_most_k(lits: &[Lit], k: usize) -> Vec<Vec<Lit>> {
    let negated: Vec<Lit> = lits.iter().map(|&l| !l).collect();
    combinations(&negated, k + 1)
}


/// No mínimo k = no máximo (n-k) dos negados.
/// k > lits.len() -> impossível: devolve a cláusula vazia.
pub fn at_least_k(lits: &[Lit], k: usize) -> Vec<Vec<Lit>> {
    if k > lits.len() {
        return vec![vec![]];
    }
    let negated: Vec<Lit> = lits.iter().map(|&l| !l).collect();
    at_most_k(&negated, lits.len() - k)
}

pub fn exactly_k(lits: &[Lit], k: usize) -> Vec<Vec<Lit>> {
    let mut clauses = at_most_k(lits, k);
    clauses.extend(at_least_k(lits, k));
    clauses
}

/// Uma região da fronteira traduzida para CNF.
pub struct Encoding {
    pub cnf: Cnf,
    /// Células da região e a variável "tem mina" de cada uma.
    pub cells: Vec<(Pos, Var)>,
}

/// "Exatamente `mines` minas entre `cells`", vinda de um número aberto.
#[derive(Clone, Debug)]
pub struct Constraint {
    pub cells: Vec<Pos>,
    pub mines: usize,
}

/// Uma restrição por `Revealed(n)` com vizinhas escondidas:
/// exatamente `n - vizinhas Flagged` minas entre as vizinhas `Hidden`.
/// Bandeiras contam como minas (o solver só marca minas provadas).
pub fn constraints(view: &View) -> Vec<Constraint> {
    view.positions()
        .filter_map(|pos| match view.get(pos) {
            Tile::Revealed(n) => {
                let neighbors: Vec<Pos> = view.neighbors(pos).collect();
                let flags = neighbors.iter().filter(|&&p| view.get(p) == Tile::Flagged).count();
                let cells: Vec<Pos> = neighbors
                    .into_iter()
                    .filter(|&p| view.get(p) == Tile::Hidden)
                    .collect();
                // Bandeira errada deixaria flags > n; satura e a região vira Unsat.
                let mines = (n as usize).saturating_sub(flags);
                (!cells.is_empty()).then_some(Constraint { cells, mines })
            }
            _ => None,
        })
        .collect()
}

/// Agrupa as restrições em regiões independentes: duas restrições ficam na
/// mesma região se compartilham (direta ou indiretamente) alguma célula.
///
/// Importante para o DPLL ingênuo: num CNF com duas regiões A e B, provar
/// Unsat em A pode obrigá-lo a enumerar todas as atribuições de B antes.
pub fn regions(constraints: &[Constraint]) -> Vec<Vec<Constraint>> {
    let mut visited = vec![false; constraints.len()];
    (0..constraints.len())
        .filter_map(|start| {
            if visited[start] {
                return None;
            }
            visited[start] = true;
            let mut stack = vec![start];
            let mut region = vec![];
            while let Some(i) = stack.pop() {
                region.push(constraints[i].clone());
                for j in 0..constraints.len() {
                    if !visited[j]
                        && constraints[j].cells.iter().any(|p| constraints[i].cells.contains(p))
                    {
                        visited[j] = true;
                        stack.push(j);
                    }
                }
            }
            Some(region)
        })
        .collect()
}

/// Uma variável por célula da região; `exactly_k` para cada restrição.
pub fn encode_region(region: &[Constraint]) -> Encoding {
    let mut positions: Vec<Pos> = region.iter().flat_map(|c| c.cells.iter().copied()).collect();
    positions.sort();
    positions.dedup();
    let var_of = |p: &Pos| Var(positions.binary_search(p).unwrap() as u32);

    let clauses: Vec<Vec<Lit>> = region
        .iter()
        .flat_map(|c| {
            let lits: Vec<Lit> = c.cells.iter().map(|p| var_of(p).pos()).collect();
            exactly_k(&lits, c.mines)
        })
        .collect();

    let cells = positions.iter().map(|p| (*p, var_of(p))).collect();
    Encoding { cnf: Cnf::new(positions.len(), clauses), cells }
}

/// O tabuleiro visível como um CNF por região da fronteira
/// (células `Hidden` vizinhas de algum número).
///
/// Células escondidas longe dos números ficam de fora: sem cláusulas,
/// elas só fariam o DPLL ingênuo ramificar à toa.
///
/// Para jogar: para cada (pos, x) em `cells`,
///   solve(cnf, [x])  é Unsat -> não pode ter mina -> `Move::Reveal(pos)`
///   solve(cnf, [¬x]) é Unsat -> tem que ter mina  -> `Move::Flag(pos)`
pub fn encode(view: &View) -> Vec<Encoding> {
    regions(&constraints(view)).iter().map(|r| encode_region(r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_2_of_3() {
        let (a, b, c) = (Var(0), Var(1), Var(2));
        let clauses = exactly_k(&[a.pos(), b.pos(), c.pos()], 2);
        assert_eq!(
            clauses,
            vec![
                // at_most_2: nenhum trio todo verdadeiro
                vec![a.neg(), b.neg(), c.neg()],
                // at_least_2: nenhum par todo falso
                vec![a.pos(), b.pos()],
                vec![a.pos(), c.pos()],
                vec![b.pos(), c.pos()],
            ]
        );
    }
}