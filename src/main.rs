
#![feature(assert_matches)]
#![feature(trait_alias)]

mod expr;
mod traits;

use std::process::ExitCode;
use std::thread_local;
use std::io::{self, Write};

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar, "/grammar.rs");
use traits::Int;

use rustyline::{DefaultEditor, error::ReadlineError};
use clap::{Parser, ValueEnum};
use num_traits::int::PrimInt;

#[inline]
fn div_round_up<T: PrimInt>(dividend: T, divisor: T) -> T {
    (dividend + divisor - T::one()) / divisor
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, strum::Display)]
#[strum(serialize_all = "kebab-case")]
enum Base {
    Hex,
    Oct,
}

impl Base {
    fn bits(self) -> u32 {
        match self {
            Self::Hex => 4,
            Self::Oct => 3,
        }
    }


    fn subscript(self) -> &'static str {
         match self {
            Base::Oct => "₈", 
            Base::Hex => "₁₆",
        }
    }
}

fn write_int<T: Int>(f: &mut impl Write, val: T, base: Base) -> io::Result<()> {
    writeln!(f, "{val}₁₀")?;

    // Writing the decimal representation, above, is signedness-aware. The rest
    // of the writing is purely the underlying representation, and doesn't vary
    // between signed and unsigned. On the other hand, the rest of it needs a
    // logical right shift, so a convertion to unsigned is done. Since the type
    // changes (given Rust's restrictions) its easiest to do the rest in a separate
    // function.
    write_int_continue(f, val.as_unsigned(), base)
}

fn write_int_continue<T: Int>(f: & mut impl Write, mut val: T, base: Base) -> io::Result<()> {
    // For oct and hex, split the binary in digit-sized chunks, and align them.

    let subscript = base.subscript();
    let digit_bits = T::from_u32(base.bits()).unwrap();
    let digit_mask = (T::one() << digit_bits.to_usize().unwrap()) - T::one();
    let mut digits = vec![];
    while val > T::zero() {
        digits.push(val & digit_mask);
        val >>= digit_bits;
    }

    // Add extra zero chunks until we reach the full width.
    let t_bits = T::from(T::zero().count_zeros()).unwrap();
    let num_chunks = div_round_up(t_bits, digit_bits);
    while digits.len() < num_chunks.to_usize().unwrap() {
        digits.push(T::zero());
    }

    // Bits in the most significant chunk (since chunk size may not evenly divide word size).
    let top_bits = if t_bits % digit_bits == T::zero() { 
        digit_bits 
    } else {
        t_bits % digit_bits
    };

    // Write hex/oct, aligned with binary.
    let mut seen_nonzero = false;
    for (i, digit) in digits.iter().rev().enumerate() {
        if i != 0 {
            write!(f, " ")?;
        }

        let chunk_width = if i == 0 { top_bits } else { digit_bits }.to_usize().unwrap();

        // Don't write leading zeros for oct/hex.
        if *digit != T::zero() {
            seen_nonzero = true;
        }
        if !seen_nonzero && i + 1 != digits.len() {
            write!(f, "{:chunk_width$}", "")?;
        } else {
            match base {
                Base::Oct => write!(f, "{digit:chunk_width$o}")?,
                Base::Hex => write!(f, "{digit:chunk_width$X}")?,
            }
        }
    }
    writeln!(f, "{subscript}")?;

    // Write binary (including leading zeros).
    for (i, digit) in digits.iter().rev().enumerate() {
        if i != 0 {
            write!(f, " ")?;
        }
        let chunk_width = if i == 0 { top_bits } else { digit_bits }.to_usize().unwrap();
        write!(f, "{digit:0chunk_width$b}")?;
    }
    writeln!(f, "₂")
}

fn print_int<T: Int>(val: T, base: Base) {
    let mut stdout = io::stdout().lock();
    write_int(&mut stdout, val, base).expect("Error printing int");
}


#[derive(Debug, Clone, Copy, strum::Display, ValueEnum)]
#[strum(serialize_all = "kebab_case")]
enum IntType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

/// Programmer's calculator
#[derive(Parser, Debug)]
struct Args {
    /// Expression to evaluate. Leave empty for repl
    expr: Option<String>,

    /// Base in which to print results. Decimal and binary are always printed
    #[arg(long, default_value_t=Base::Hex)]
    base: Base,

    /// Type of expression
    #[arg(long = "type", default_value_t=IntType::U32)]
    typ: IntType,
}

macro_rules! eval {
    ($expr:ident, $base:ident, $typ:ty) => {{
        let val = match $expr.eval::<$typ>() {
            Ok(val) => val,
            Err(e) => { 
                eprintln!("{e}");
                return Err(());
            },
        };

        print_int(val, $base);
    }}
}

fn exec(expr: &str, base: Base, typ: IntType) -> Result<(), ()> {
    thread_local! {
        static PARSER: grammar::ExprParser = Default::default();
    }

    let expr = match PARSER.with(|p| p.parse(expr)) {
        Ok(expr) => expr,
        Err(e) => { 
            eprintln!("{e}");
            return Err(());
        },
    };

    use IntType::*;
    match typ {
        U8 => eval!(expr, base, u8),
        U16 => eval!(expr, base, u16),
        U32 => eval!(expr, base, u32),
        U64 => eval!(expr, base, u64),
        I8 => eval!(expr, base, i8),
        I16 => eval!(expr, base, i16),
        I32 => eval!(expr, base, i32),
        I64 => eval!(expr, base, i64),
    }

    Ok(())
}

fn main() -> ExitCode {

    let args = Args::parse();

    if let Some(expr) = &args.expr {
        return match exec(expr, args.base, args.typ) {
            Ok(()) => ExitCode::SUCCESS,
            Err(()) => ExitCode::FAILURE,
        }
    }

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                if line.chars().all(|ch| ch.is_whitespace()) {
                    continue; 
                }
                let _ = exec(&line, args.base, args.typ);
            },
            Err(ReadlineError::Interrupted)| Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::str::FromStr;
    use std::sync::LazyLock;
    use std::io::BufWriter;

    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(grammar, "/grammar.rs");
    use super::{Base, write_int};
    use crate::traits::Int;

    use regex::Regex;

    fn check_dec<T: Int + FromStr>(s: &str, expected: T) 
        where <T as FromStr>::Err: Debug {

        thread_local! {
            static RE: Regex = Regex::new(r#"(-?\d+)₁₀"#).unwrap();
        }
        let caps = RE.with(|re| re.captures(s).unwrap());
        let val = caps.get(1).unwrap().as_str().parse::<T>().unwrap();
        assert_eq!(val, expected); 
    }

    fn check_hex<T: Int>(s: &str, expected: T) 
        where <T as num_traits::Num>::FromStrRadixErr: Debug {

        thread_local! {
            static OVERALL_RE: Regex = Regex::new(r#"^(?:\s+[[:xdigit:]])+₁₆$"#).unwrap();
            static DIGIT_RE: Regex = Regex::new(r#"\s+([[:xdigit:]])"#).unwrap();
        }

        assert!(OVERALL_RE.with(|re| re.is_match(s)));
        let digit_bits = T::from_u32(Base::Hex.bits()).unwrap();
        let mut val = T::zero();
        DIGIT_RE.with(|re| {
            for x in re.captures_iter(s) {
                val <<= digit_bits;
                val += T::from_str_radix(x.get(1).unwrap().as_str(), 16).unwrap();
            }
        });
        assert_eq!(val, expected); 
    }

    fn check_oct<T: Int>(s: &str, expected: T) 
        where <T as num_traits::Num>::FromStrRadixErr: Debug {

        thread_local! {
            static OVERALL_RE: Regex = Regex::new(r#"^(?:\s*[0-7])+₈$"#).unwrap();
            static DIGIT_RE: Regex = Regex::new(r#"\s*([0-7])"#).unwrap();
        }

        assert!(OVERALL_RE.with(|re| re.is_match(s)));

        // The first digit has fewer, but the first shift doesn't do anything
        // (the accumulator is all zeros), and after that the shift is by regular
        // 3-bit digits.
        let digit_bits = T::from_u32(Base::Oct.bits()).unwrap();

        let mut val = T::zero();
        DIGIT_RE.with(|re| {
            for x in re.captures_iter(s) {
                val <<= digit_bits;
                val += T::from_str_radix(x.get(1).unwrap().as_str(), 8).unwrap();
            }
        });
        assert_eq!(val, expected); 
    }


    fn check_bin<T: Int>(s: &str, base: Base, expected: T)
        where <T as num_traits::Num>::FromStrRadixErr: Debug {

        thread_local! {
            static OVERALL_RE: Regex = Regex::new(r#"^(?: ?[01]{1,4})+₂$"#).unwrap();
            static DIGIT_RE: Regex = Regex::new(r#" ?([01]{1,4})"#).unwrap();
        }

        assert!(OVERALL_RE.with(|re| re.is_match(s)));
        // See comment on digit_bits in check_oct()
        let group_bits = T::from_u32(base.bits()).unwrap();
        let mut val = T::zero();
        DIGIT_RE.with(|re| {
            for x in re.captures_iter(s) {
                val <<= group_bits;
                val += T::from_str_radix(x.get(1).unwrap().as_str(), 2).unwrap();
            }
        });
        assert_eq!(val, expected); 
    }

    fn check_output<T: Int + FromStr>(s: &str, base: Base, expected: T)
        where <T as num_traits::Num>::FromStrRadixErr: Debug,
              <T as FromStr>::Err: Debug {

        let mut lines = s.lines();
        let dec = lines.next().expect("Missing dec line");
        check_dec::<T>(dec, expected);

        let hex_or_oct = lines.next().expect("Missing hex or oct line");
        match base {
            Base::Oct => check_oct::<T>(hex_or_oct, expected),
            Base::Hex => check_hex::<T>(hex_or_oct, expected),
        }

        let bin = lines.next().expect("Missing bin line");
        check_bin::<T>(bin, base, expected);

        assert_eq!(lines.next(), None);
    }

    fn run<T: Int + FromStr>(expr: &str, base: Base, expected: T)
        where <T as num_traits::Num>::FromStrRadixErr: Debug,
              <T as FromStr>::Err: Debug {

        static PARSER: LazyLock<grammar::ExprParser> = LazyLock::new(||
            Default::default()
        );
        
        let expr = PARSER.parse(expr).unwrap();
        let val = expr.eval::<T>().unwrap();
        assert_eq!(val, expected);

        let mut output = BufWriter::new(vec![]);
        write_int(&mut output, val, base).unwrap();

        let s = String::from_utf8(output.into_inner().unwrap()).unwrap();
        check_output(&s, base, expected);
    }

    #[test]
    fn tests() {
        run::<u32>("307200", Base::Hex, 307200);
        run::<i32>("-307200", Base::Hex, -307200);
        run::<u32>("307200", Base::Oct, 307200);

        run::<u32>("3 * 3", Base::Oct, 3 * 3);
    }

    fn simple_tests<T: Int + FromStr>(base: Base)
        where <T as num_traits::Num>::FromStrRadixErr: Debug,
              <T as FromStr>::Err: Debug {

        run("1", base, 1);
        run("(1)", base, 1);
        run("(1 + 1)", base, 2);
        run("(1) + 1", base, 2);
        run("1 + (1)", base, 2);
        run("10 + 2 * 5", base, 20);
        run("(10 + 2) * 5", base, 60);

        run("10/3", base, 3);
        run("10 % 3", base, 1);

        run("2 & 1", base, 0);
        run("2 | 1", base, 3);
        run("3 ^ 1", base, 2);
        run("1 | 6 ^ 7 & 12", base, 3);
        run("(1 | 6) ^ 7 & 12", base, 3);
        run("1 | (6 ^ 7) & 12", base, 1);
        run("!34", base, !34);
        run("~34", base, !34);

        run("3 << 2", base, 12);
        run("12 >> 2", base, 3);

        run("0 - 1", base, T::zero().wrapping_sub(&T::one()));
        run("-64 + 3", base, T::from_u32(64).unwrap().wrapping_neg().wrapping_add(&T::from_u32(3).unwrap()));

        run(&format!("{} + 1", T::max_value()), base, T::max_value().wrapping_add(&T::one()));
    }

    #[test]
    fn simple() {
        for base in [Base::Hex, Base::Oct] {
            simple_tests::<u8>(base);
            simple_tests::<u16>(base);
            simple_tests::<u32>(base);
            simple_tests::<u64>(base);

            simple_tests::<i8>(base);
            simple_tests::<i16>(base);
            simple_tests::<i32>(base);
            simple_tests::<i64>(base);

        }
    }

}
