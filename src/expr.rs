
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Error parsing literal: {}", .0)]
    LitParse(String),
}


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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::ExprParser;

    use std::assert_matches::assert_matches;

    use lalrpop_util::ParseError;

    #[test]
    fn simple() {
        let parser = ExprParser::new();

        assert_eq!(parser.parse("1").unwrap().eval(), 1);
        assert_eq!(parser.parse("(1)").unwrap().eval(), 1);
        assert_eq!(parser.parse("(1 + 1)").unwrap().eval(), 2);
        assert_eq!(parser.parse("(1) + 1").unwrap().eval(), 2);
        assert_eq!(parser.parse("1 + (1)").unwrap().eval(), 2);
        assert_eq!(parser.parse("10 + 2 * 5").unwrap().eval(), 20);
        assert_eq!(parser.parse("(10 + 2) * 5").unwrap().eval(), 60);

        assert_eq!(parser.parse("10/3").unwrap().eval(), 3);
        assert_eq!(parser.parse("10 % 3").unwrap().eval(), 1);

        assert_eq!(parser.parse("2 & 1").unwrap().eval(), 0);
        assert_eq!(parser.parse("2 | 1").unwrap().eval(), 3);
        assert_eq!(parser.parse("3 ^ 1").unwrap().eval(), 2);
        assert_eq!(parser.parse("1 | 6 ^ 7 & 12").unwrap().eval(), 3);
        assert_eq!(parser.parse("(1 | 6) ^ 7 & 12").unwrap().eval(), 3);
        assert_eq!(parser.parse("1 | (6 ^ 7) & 12").unwrap().eval(), 1);

        assert_eq!(parser.parse("3 << 2").unwrap().eval(), 12);
        assert_eq!(parser.parse("12 >> 2").unwrap().eval(), 3);
    }

    #[test]
    fn malformed() {
        let parser = ExprParser::new();

        assert_matches!(
            parser.parse("1000000000000").unwrap_err(),
            ParseError::User{error: Error::LitParse(_)},
        );
        assert_matches!(
            parser.parse("0xg").unwrap_err(),
            ParseError::InvalidToken{..},
        );
        assert_matches!(
            parser.parse("0x1000000000000").unwrap_err(),
            ParseError::User{error: Error::LitParse(_)},
        );
        assert_matches!(
            parser.parse("0o9").unwrap_err(),
            ParseError::InvalidToken{..},
        );
        assert_matches!(
            parser.parse("0o1000000000000").unwrap_err(),
            ParseError::User{error: Error::LitParse(_)},
        );
        parser.parse("10 + 1)").unwrap_err();
        parser.parse("10 ++ 1)").unwrap_err();
        parser.parse("10 += 1)").unwrap_err();
        parser.parse("(10 + 1").unwrap_err();
        parser.parse("10)( + 1").unwrap_err();
        parser.parse("10() + 1").unwrap_err();
    }

    #[test]
    fn unary() {
        let parser = ExprParser::new();

        assert_eq!(parser.parse("-1").unwrap().eval(), -1i32 as u32);
        assert_eq!(parser.parse("-0").unwrap().eval(), -0i32 as u32);
        assert_eq!(parser.parse("-5 - 6").unwrap().eval(), (-5i32 as u32).wrapping_sub(6));
        assert_eq!(parser.parse("-5 + 6").unwrap().eval(), (-5i32 as u32).wrapping_add(6));
        assert_eq!(parser.parse("-5 + -6").unwrap().eval(), (-5i32 as u32).wrapping_add(-6i32 as u32));

        assert_eq!(parser.parse("!0").unwrap().eval(), !0);
        assert_eq!(parser.parse("!1").unwrap().eval(), !1);
        assert_eq!(parser.parse("!32").unwrap().eval(), !32);
        assert_eq!(parser.parse("!(-32)").unwrap().eval(), !(-32i32 as u32));
    }

    #[test]
    fn radix_literal() {
        let parser = ExprParser::new();

        assert_eq!(parser.parse("0xf").unwrap().eval(), 15);
        assert_eq!(parser.parse("0o20").unwrap().eval(), 16);
        assert_eq!(parser.parse("0xf ^ 0o20").unwrap().eval(), 31);
    }
}


