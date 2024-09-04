
use crate::traits::Int;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Error parsing literal: {}", .0)]
    LitParse(String),
}



#[derive(Debug, Clone, Copy, Error)]
pub enum EvalErr{
    #[error("Literal '{}' invalid", .0)]
    Invalid(i128),
}


#[derive(Debug, Clone)]
pub enum Expr {
    // Precedence 1 (or parenthensized).
    Num(i128),

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
    pub fn eval<T: Int>(&self) -> Result<T, EvalErr> {
        let t_bits = T::from(T::zero().count_zeros()).unwrap();
        use Expr::*;
        Ok(match self {
            Num(n) => T::from_i128(*n).ok_or(EvalErr::Invalid(*n))?,

            Neg(e) => {
                if T::is_signed() {
                    if let Num(n) = &**e {
                        // For signe numbers, negation of a literal needs special 
                        // handling: -INT_MIN isn't representable, so do the negation 
                        // before converting from i128.
                        let val = n.wrapping_neg();
                        return T::from_i128(val).ok_or(EvalErr::Invalid(val));
                    }
                }
                e.eval::<T>()?.wrapping_neg()
            }

            Bitnot(e) => e.eval::<T>()?.not(),

            Mul(l, r) => l.eval::<T>()?.wrapping_mul(&r.eval::<T>()?),
            Div(l, r) => l.eval::<T>()?.wrapping_div(&r.eval::<T>()?),
            Rem(l, r) => l.eval::<T>()?.wrapping_rem(&r.eval::<T>()?),

            Add(l, r) => l.eval::<T>()?.wrapping_add(&r.eval::<T>()?),
            Sub(l, r) => l.eval::<T>()?.wrapping_sub(&r.eval::<T>()?),

            Shr(l, r) => l.eval::<T>()?.wrapping_shr((r.eval::<T>()? & (t_bits - T::one())).to_u32().unwrap()),
            Shl(l, r) => l.eval::<T>()?.wrapping_shl((r.eval::<T>()? & (t_bits - T::one())).to_u32().unwrap()),

            And(l, r) => l.eval::<T>()?.bitand(r.eval::<T>()?),

            Xor(l, r) => l.eval::<T>()?.bitxor(r.eval::<T>()?),

            Or(l, r) => l.eval::<T>()?.bitor(r.eval::<T>()?),
        })
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::ExprParser;

    use std::assert_matches::assert_matches;
    use std::thread_local;

    use lalrpop_util::ParseError;

    fn eval<T: Int>(s: &str) -> T {
        thread_local! {
            static PARSER: ExprParser = Default::default();
        }
        PARSER.with(|p|
            p.parse(s).unwrap().eval::<T>().unwrap()
        )
    }

    macro_rules! simple_tests {
        ($typ:ty) => {
            assert_eq!(eval::<$typ>("1"), 1);
            assert_eq!(eval::<$typ>("(1)"), 1);
            assert_eq!(eval::<$typ>("(1 + 1)"), 2);
            assert_eq!(eval::<$typ>("(1) + 1"), 2);
            assert_eq!(eval::<$typ>("1 + (1)"), 2);
            assert_eq!(eval::<$typ>("10 + 2 * 5"), 20);
            assert_eq!(eval::<$typ>("(10 + 2) * 5"), 60);

            assert_eq!(eval::<$typ>("10/3"), 3);
            assert_eq!(eval::<$typ>("10 % 3"), 1);

            assert_eq!(eval::<$typ>("2 & 1"), 0);
            assert_eq!(eval::<$typ>("2 | 1"), 3);
            assert_eq!(eval::<$typ>("3 ^ 1"), 2);
            assert_eq!(eval::<$typ>("1 | 6 ^ 7 & 12"), 3);
            assert_eq!(eval::<$typ>("(1 | 6) ^ 7 & 12"), 3);
            assert_eq!(eval::<$typ>("1 | (6 ^ 7) & 12"), 1);

            assert_eq!(eval::<$typ>("3 << 2"), 12);
            assert_eq!(eval::<$typ>("12 >> 2"), 3);

            assert_eq!(eval::<$typ>("0 - 1"), 0.wrapping_sub(&1));
            assert_eq!(eval::<$typ>("-64 + 3"), 64.wrapping_neg().wrapping_add(&3));

            assert_eq!(eval::<$typ>(&format!("{} + 1", <$typ>::MAX)), <$typ>::MAX.wrapping_add(1));
        }
    }

    #[test]
    fn simple() {
        simple_tests!(u8);
        simple_tests!(u16);
        simple_tests!(u32);
        simple_tests!(u64);

        simple_tests!(i8);
        simple_tests!(i16);
        simple_tests!(i32);
        simple_tests!(i64);
    }

    #[test]
    fn malformed() {
        let parser = ExprParser::new();

        assert_matches!(
            parser.parse("1000000000000").unwrap().eval::<u32>(),
            Err(EvalErr::Invalid(_))
        );
        assert_matches!(
            parser.parse("0xg").unwrap_err(),
            ParseError::InvalidToken{..},
        );
        assert_matches!(
            parser.parse("0x1000000000000").unwrap().eval::<u32>(),
            Err(EvalErr::Invalid(_))
        );
        assert_matches!(
            parser.parse("0o9").unwrap_err(),
            ParseError::InvalidToken{..},
        );
        assert_matches!(
            parser.parse("0o1000000000000").unwrap().eval::<u32>(),
            Err(EvalErr::Invalid(_))
        );
        parser.parse("10 + 1)").unwrap_err();
        parser.parse("10 ++ 1)").unwrap_err();
        parser.parse("10 += 1)").unwrap_err();
        parser.parse("(10 + 1").unwrap_err();
        parser.parse("10)( + 1").unwrap_err();
        parser.parse("10() + 1").unwrap_err();

        assert_matches!(
            parser.parse("-256 - 1").unwrap().eval::<i8>(),
            Err(EvalErr::Invalid(_))
        );
    }

    #[test]
    fn unary() {
        assert_eq!(eval::<u32>("-1"), -1i32 as u32);
        assert_eq!(eval::<u32>("-0"), -0i32 as u32);
        assert_eq!(eval::<u32>("-5 - 6"), (-5i32 as u32).wrapping_sub(6));
        assert_eq!(eval::<u32>("-5 + 6"), (-5i32 as u32).wrapping_add(6));
        assert_eq!(eval::<u32>("-5 + -6"), (-5i32 as u32).wrapping_add(-6i32 as u32));

        assert_eq!(eval::<u32>("!0"), !0);
        assert_eq!(eval::<u32>("!1"), !1);
        assert_eq!(eval::<u32>("!32"), !32);
        assert_eq!(eval::<u32>("!(-32)"), !(-32i32 as u32));
    }

    #[test]
    fn radix_literal() {
        assert_eq!(eval::<u32>("0xf"), 15);
        assert_eq!(eval::<u32>("0o20"), 16);
        assert_eq!(eval::<u32>("0xf ^ 0o20"), 31);
    }

    macro_rules! signed_tests {
        ($typ:ty) => {
            assert_eq!(eval::<$typ>("0"), 0);
            assert_eq!(eval::<$typ>("1"), 1);
            assert_eq!(eval::<$typ>("-1"), -1);
            assert_eq!(eval::<$typ>("8 - 15"), -7);
            assert_eq!(eval::<$typ>("-3 * - 15"), 45);
            assert_eq!(eval::<$typ>("-3 * 15"), -45);

            assert_eq!(eval::<$typ>(&format!("{} - 1", <$typ>::MIN)), <$typ>::MIN.wrapping_sub(1));
            assert_eq!(eval::<$typ>(&format!("{} + 1", <$typ>::MAX)), <$typ>::MAX.wrapping_add(1));
            assert_eq!(eval::<$typ>(&format!("-1 * {}", <$typ>::MIN)), <$typ>::MIN.wrapping_mul(-1));
            assert_eq!(eval::<$typ>("23 >> 3"), 23.wrapping_shr(3));
            assert_eq!(eval::<$typ>(&format!("{} >> 1", <$typ>::MIN)), <$typ>::MIN.wrapping_shr(1));
            assert_eq!(eval::<$typ>(&"-1 >> 1"), -1);
        }
    }

    #[test]
    fn signed() {
        signed_tests!(i8);
        signed_tests!(i16);
        signed_tests!(i32);
        signed_tests!(i64);
    }
}


