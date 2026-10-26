#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var(pub u32);

/// Literal codificado como 2*var + (1 se negado).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lit(u32);

impl Var {
    pub fn pos(self) -> Lit { Lit(self.0 << 1) }
    pub fn neg(self) -> Lit { Lit((self.0 << 1) | 1) }
    pub fn index(self) -> usize { self.0 as usize }
}

impl Lit {
    pub fn var(self) -> Var { Var(self.0 >> 1) }
    pub fn is_neg(self) -> bool { self.0 & 1 == 1 }
    /// Útil para watch lists: vetor de tamanho 2*num_vars.
    pub fn index(self) -> usize { self.0 as usize }
}

impl std::ops::Not for Lit {
    type Output = Lit;
    fn not(self) -> Lit { Lit(self.0 ^ 1) }
}