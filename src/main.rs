
#![feature(assert_matches)]

mod expr;

use std::process::ExitCode;
use std::thread_local;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar, "/grammar.rs");

use rustyline::{DefaultEditor, error::ReadlineError};
use clap::{Parser, ValueEnum};
use strum::Display;

#[inline]
fn div_round_up(dividend: u32, divisor: u32) -> u32 {
    (dividend + divisor - 1) / divisor
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Display)]
#[strum(serialize_all = "kebab-case")]
enum Base {
    Hex,
    Oct,
}

impl Base {
    fn print(self, mut val: u32) {
        println!("{val}₁₀");

        // For oct and hex, split the binary in digit-sized chunks, and align them.
        let (digit_bits, subscript) = match self {
            Base::Oct => (3, &"₈"), 
            Base::Hex => (4, &"₁₆"),
        };
        let digit_mask = (0x1 << digit_bits) - 1;
        let mut digits = vec![];
        while val > 0 {
            digits.push(val & digit_mask);
            val >>= digit_bits;
        }

        let num_chunks = div_round_up(u32::BITS, digit_bits);
        while digits.len() < num_chunks as usize {
            digits.push(0);
        }

        // Bits in the most significant chunk (since chunk size may not evenly divide word size).
        let top_bits = if u32::BITS % digit_bits == 0 { 
            digit_bits 
        } else {
            u32::BITS % digit_bits
        };

        // Print hex/oct, aligned with binary.
        let mut seen_nonzero = false;
        for (i, digit) in digits.iter().rev().enumerate() {
            if i != 0 {
                print!(" ");
            }

            let chunk_width = if i == 0 { top_bits } else { digit_bits } as usize;

            // Don't print oct/hex leading zeros.
            if *digit != 0 {
                seen_nonzero = true;
            }
            if !seen_nonzero && i + 1 != digits.len() {
                print!("{:chunk_width$}", "");
            } else if self == Base::Oct {
                print!("{digit:chunk_width$o}");
            } else if self == Base::Hex {
                print!("{digit:chunk_width$X}");
            } else {
                unreachable!();
            }
        }
        println!("{subscript}");

        // Print binary.
        for (i, digit) in digits.iter().rev().enumerate() {
            if i != 0 {
                print!(" ");
            }
            let chunk_width = if i == 0 { top_bits } else { digit_bits } as usize;
            print!("{digit:0chunk_width$b}");
        }
        println!("₂");

    }
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
                args.base.print(val);
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
                    Ok(val) => args.base.print(val),
                    Err(msg) => eprintln!("{msg}"),
                }

            },
            Err(ReadlineError::Interrupted)| Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    ExitCode::SUCCESS
}

