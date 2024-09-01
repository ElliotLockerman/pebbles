



#[derive(Debug, Clone)]
pub enum Expr {
    // Precedence 1 (or parenthensized).
    Num(u32),

    // Precedence 2.
    Neg(Box<Expr>),
    Bitnot(Box<Expr>),

    // Precedence 3 reserved for "as".

    // Precedence 4.
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Rem(Box<Expr>, Box<Expr>),


    // Precedence 4.
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),

    // Precedence 6.
    Shr(Box<Expr>, Box<Expr>),
    Shl(Box<Expr>, Box<Expr>),

    // Precedence 7.
    And(Box<Expr>, Box<Expr>),

    // Precedence 8.
    Xor(Box<Expr>, Box<Expr>),

    // Precedence 9.
    Or(Box<Expr>, Box<Expr>),
}


impl Expr {
    pub fn eval(&self) -> u32 {
        use Expr::*;
        match self {
            Num(n) => *n,

            Neg(e) => e.eval().wrapping_neg(),
            Bitnot(e) => !e.eval(),

            Mul(l, r) => l.eval().wrapping_mul(r.eval()),
            Div(l, r) => l.eval().wrapping_div(r.eval()),
            Rem(l, r) => l.eval().wrapping_rem(r.eval()),

            Add(l, r) => l.eval().wrapping_add(r.eval()),
            Sub(l, r) => l.eval().wrapping_sub(r.eval()),

            Shr(l, r) => l.eval().wrapping_shr(r.eval()),
            Shl(l, r) => l.eval().wrapping_shl(r.eval()),

            And(l, r) => l.eval() & r.eval(),

            Xor(l, r) => l.eval() ^ r.eval(),

            Or(l, r) => l.eval() | r.eval(),
        }
    }

}
