
#![feature(assert_matches)]

mod ast;

use std::process::ExitCode;
use std::thread_local;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar, "/grammar.rs");

use rustyline::{DefaultEditor, error::ReadlineError};
use clap::Parser;


#[derive(Parser, Debug)]
struct Args {
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
                println!("{val}");
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
                    Ok(val) => println!("{val}"),
                    Err(msg) => eprintln!("{msg}"),
                }

            },
            Err(ReadlineError::Interrupted)| Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    ExitCode::SUCCESS
}

