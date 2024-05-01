#![allow(warnings)]

//! Quale Compiler Framework
mod analyzer;
mod ast;
mod attributes;
mod codegen;
mod config;
mod error;
mod inference;
mod lexer;
mod optimizer;
mod parser;
mod types;
mod utils;

use crate::codegen::{qasm, Translator};
use crate::error::Result;
use crate::inference::infer;
use crate::parser::Parser;

fn init_session(args: Vec<&str>) -> Result<()> {
    let session = Parser::new(args)?;

    match session {
        Some(mut parser) => {
            let config = parser.get_config();

            let mut qast = parser.parse(&config.analyzer.src)?;

            return infer(&mut qast);

            if config.dump_ast_only {
                println!("{qast}");
                return Ok(());
            }
            if config.dump_ast {
                println!("{qast}");
            }

            if config.analyzer.status {
                config.analyzer.analyze(&qast)?;
            }

            let asm = qasm::QasmModule::translate(qast)?;
            if config.dump_qasm {
                println!("{asm}");
            }
            asm.generate(&config.optimizer.asm)?;
        }
        None => {} /* help was asked, no errors */
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let args = args.iter().map(|s| s.as_str()).collect();

    if let Err(err) = init_session(args) {
        eprintln!("{err}");
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
        let mut parser = Parser::new(args)?.unwrap();
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
