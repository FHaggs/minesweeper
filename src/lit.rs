use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var(pub u32);

/// Literal codificado como 2*var + (1 se negado).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lit(pub u32);

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

/// `Var(3)` aparece como `x3`.
impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "x{}", self.0) }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(self, f) }
}

/// `Var(3).pos()` aparece como `x3` e `Var(3).neg()` como `¬x3`.
impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_neg() {
            write!(f, "¬")?;
        }
        write!(f, "{}", self.var())
    }
}

impl fmt::Debug for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(self, f) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting() {
        let b = Var(1);
        assert_eq!(b.to_string(), "x1");
        assert_eq!(b.neg().to_string(), "¬x1");
        assert_eq!(format!("{:?}", [b.pos(), !b.pos(), Var(0).neg()]), "[x1, ¬x1, ¬x0]");
    }
}
