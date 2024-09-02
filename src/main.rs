
#![feature(assert_matches)]

mod expr;

use std::process::ExitCode;
use std::thread_local;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar, "/grammar.rs");

use rustyline::{DefaultEditor, error::ReadlineError};
use clap::{Parser, ValueEnum};
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Display)]
#[strum(serialize_all = "kebab-case")]
enum Base {
    Dec,
    Oct,
    Hex
}

impl Base {
    fn print(self, val: u32) {
        match self {
            Base::Dec => Self::print_dec(val),
            Base::Oct => Self::print_oct(val),
            Base::Hex => Self::print_hex(val),
        }
    }

    fn print_dec(val: u32) {
        println!("{val:32}");
        println!("{val:032b}");
    }

    fn print_oct(mut val: u32) {
        const OCT_BITS: u32 = 3; // Bits in an octal digits.
        const OCT_MASK: u32 = (1 << OCT_BITS) - 1;
        let mut digits = vec![];
        while val > 0 {
            digits.push(val & OCT_MASK);
            val >>= OCT_BITS;
        }

        while digits.len() < (u32::BITS / OCT_BITS) as usize {
            digits.push(0);
        }

        for (i, digit) in digits.iter().rev().enumerate() {
            if i != 0 {
                print!(" ");
            }

            let width = if i == 0 { 2 } else { OCT_BITS as usize };

            // Don't print oct leading zeros.
            if *digit == 0 && i + 1 != digits.len() {
                print!("{:width$}", "");
            } else {
                print!("{digit:width$o}");
            }
        }
        println!();

        for (i, digit) in digits.iter().rev().enumerate() {
            if i != 0 {
                print!(" ");
            }
            let width = if i == 0 { 2 } else { OCT_BITS as usize };
            print!("{digit:0width$b}");
        }
        println!();
    }

    fn print_hex(mut val: u32) {
        const HEX_BITS: u32 = 4; // Bits in an octal digits.
        const HEX_MASK: u32 = (1 << HEX_BITS) - 1;
        let mut digits = vec![];
        while val > 0 {
            digits.push(val & HEX_MASK);
            val >>= HEX_BITS;
        }

        while digits.len() < (u32::BITS / HEX_BITS) as usize {
            digits.push(0);
        }

        let cluster_width = HEX_BITS as usize;
        for (i, digit) in digits.iter().rev().enumerate() {
            if i != 0 {
                print!(" ");
            }

            // Don't print hex leading zeros.
            if *digit == 0 && i + 1 != digits.len() {
                print!("{:cluster_width$}", "");
            } else {
                print!("{digit:cluster_width$X}");
            }
        }
        println!();

        for (i, digit) in digits.iter().rev().enumerate() {
            if i != 0 {
                print!(" ");
            }
            print!("{digit:0cluster_width$b}");
        }
        println!();
    }

}

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t=Base::Dec)]
    base: Base,
    expr: Option<String>,
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

