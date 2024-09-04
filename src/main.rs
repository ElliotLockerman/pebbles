
#![feature(assert_matches)]
#![feature(trait_alias)]

mod expr;
mod traits;

use std::process::ExitCode;
use std::thread_local;

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

fn print_int<T: Int>(val: T, base: Base) {
    println!("{val}₁₀");

    print_int_continue(val.as_unsigned(), base);
}

fn print_int_continue<T: Int>(mut val: T, base: Base) {
    // For oct and hex, split the binary in digit-sized chunks, and align them.
    let (digit_bits, subscript) = match base {
        Base::Oct => (T::from(3).unwrap(), &"₈"), 
        Base::Hex => (T::from(4).unwrap(), &"₁₆"),
    };
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

    // Print hex/oct, aligned with binary.
    let mut seen_nonzero = false;
    for (i, digit) in digits.iter().rev().enumerate() {
        if i != 0 {
            print!(" ");
        }

        let chunk_width = if i == 0 { top_bits } else { digit_bits }.to_usize().unwrap();

        // Don't print oct/hex leading zeros.
        if *digit != T::zero() {
            seen_nonzero = true;
        }
        if !seen_nonzero && i + 1 != digits.len() {
            print!("{:chunk_width$}", "");
        } else {
            match base {
                Base::Oct => print!("{digit:chunk_width$o}"),
                Base::Hex => print!("{digit:chunk_width$X}"),
            }
        }
    }
    println!("{subscript}");

    // Print binary (including leading zeros).
    for (i, digit) in digits.iter().rev().enumerate() {
        if i != 0 {
            print!(" ");
        }
        let chunk_width = if i == 0 { top_bits } else { digit_bits }.to_usize().unwrap();
        print!("{digit:0chunk_width$b}");
    }
    println!("₂");

}


#[derive(Debug, Clone, Copy, strum::Display, ValueEnum)]
#[strum(serialize_all = "kebab_case")]
enum IntType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Parser, Debug)]
struct Args {
    expr: Option<String>,

    #[arg(long, default_value_t=Base::Hex)]
    base: Base,

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
        U128 => eval!(expr, base, u128),
        I8 => eval!(expr, base, i8),
        I16 => eval!(expr, base, i16),
        I32 => eval!(expr, base, i32),
        I64 => eval!(expr, base, i64),
        I128 => eval!(expr, base, i128),
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

