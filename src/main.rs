
mod ast;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar, "/grammar.rs");

use rustyline::{DefaultEditor, error::ReadlineError};



fn main() {
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                if line.chars().all(|ch| ch.is_whitespace()) {
                    continue; 
                }
                let expr = match grammar::ExprParser::new().parse(&line) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    },
                };
       
                println!("{}", expr.eval());
            },
            Err(ReadlineError::Interrupted)| Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::grammar::ExprParser;
    use lalrpop_util::ParseError;

    #[test]
    fn simple() {
        let parser = ExprParser::new();

        assert_eq!(parser.parse("1").unwrap().eval(), 1);
        assert_eq!(parser.parse("1 + 1").unwrap().eval(), 2);
        assert_eq!(parser.parse("10 + 2 * 5").unwrap().eval(), 20);
        assert_eq!(parser.parse("(10 + 2) * 5").unwrap().eval(), 60);
    }

    #[test]
    fn too_big() {
        let parser = ExprParser::new();

        assert_eq!(
            parser.parse("1000000000000").unwrap_err(),
            ParseError::User{error: ast::Error::LitTooBig},
        );
    }
}


