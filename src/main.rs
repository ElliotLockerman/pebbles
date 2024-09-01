
#![feature(assert_matches)]

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
            ParseError::User{error: ast::Error::LitParse(_)},
        );
        assert_matches!(
            parser.parse("0xg").unwrap_err(),
            ParseError::InvalidToken{..},
        );
        assert_matches!(
            parser.parse("0x1000000000000").unwrap_err(),
            ParseError::User{error: ast::Error::LitParse(_)},
        );
        assert_matches!(
            parser.parse("0o9").unwrap_err(),
            ParseError::InvalidToken{..},
        );
        assert_matches!(
            parser.parse("0o1000000000000").unwrap_err(),
            ParseError::User{error: ast::Error::LitParse(_)},
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


