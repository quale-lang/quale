//! quale compiler framework
mod analyzer;
mod ast;
mod attributes;
mod codegen;
mod config;
mod error;
mod lexer;
mod optimizer;
mod parser;
mod types;
mod utils;

use crate::codegen::{qasm, Translator};
use crate::error::Result;
use crate::parser::Parser;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let args = args.iter().map(|s| s.as_str()).collect();

    let session = Parser::new(args);

    match session {
        Ok(Some(parser)) => {
            let config = parser.get_config();

            match parser.parse(&config.analyzer.src) {
                Ok(qast) => {
                    #[cfg(debug_assertions)]
                    println!("{qast}");

                    if config.analyzer.status {
                        config.analyzer.analyze(&qast)?;
                    }

                    let asm = qasm::QasmModule::translate(qast)?;
                    asm.generate(&config.analyzer.src.replace(".ql", ".s"))?;

                    #[cfg(debug_assertions)]
                    println!("{}", asm);
                }
                Err(err) => eprintln!("{err}"),
            }
        }
        Ok(None) => {} /* help asked, no errors */
        Err(err) => eprintln!("{err}"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_main() -> Result<()> {
        let path = "./tests/test1.ql";
        let args = vec![path, "--analyze"];
        let parser = Parser::new(args)?.unwrap();
        let config = parser.get_config();
        let qast = parser.parse(&config.analyzer.src)?;
        println!("{qast}");

        Ok(())
    }

    #[test]
    fn check_wrong_parser_uses() -> Result<()> {
        use crate::error::QccErrorKind::NoFile;

        let path = "non_existing_src.ql";
        let args = vec![path];
        Ok(match Parser::new(args) {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, NoFile.into()),
        })
    }
}
