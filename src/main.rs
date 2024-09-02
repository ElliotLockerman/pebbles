
#![feature(assert_matches)]

mod expr;

use std::process::ExitCode;
use std::thread_local;
use std::{fmt, ops};

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar, "/grammar.rs");

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

fn print_int<T>(mut val: T, base: Base) 
    where T: PrimInt + fmt::Display + fmt::Octal + fmt::UpperHex + fmt::Binary + ops::ShrAssign {
    println!("{val}₁₀");

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

#[derive(Parser, Debug)]
struct Args {
    expr: Option<String>,
    #[arg(long, default_value_t=Base::Hex)]
    base: Base,
}

fn eval(expr: &str) -> Result<u32, String> {
    thread_local! {
        static PARSER: grammar::ExprParser = Default::default();
    }

    match PARSER.with(|p| p.parse(expr)) {
        Ok(expr) => Ok(expr.eval()),
        Err(e) => Err(e.to_string()),
    }
}

fn main() -> ExitCode {

    let args = Args::parse();

    if let Some(expr) = &args.expr {
        return match eval(expr) {
            Ok(val) => {
                print_int(val, args.base);
                ExitCode::SUCCESS
            },
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::FAILURE
            },
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
                match eval(&line) {
                    Ok(val) => print_int(val, args.base),
                    Err(msg) => eprintln!("{msg}"),
                }

            },
            Err(ReadlineError::Interrupted)| Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    ExitCode::SUCCESS
}

