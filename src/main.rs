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

use crate::error::Result;
use crate::parser::Parser;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let parser: Parser = Default::default();

    match parser.parse_cmdline(args) {
        Ok(Some(config)) => match parser.parse(&config.analyzer.src) {
            Ok(qast) => {
                #[cfg(debug_assertions)]
                println!("{}", qast);

                if config.analyzer.status {
                    config.analyzer.analyze(&qast);
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        Ok(None) => {} /* help asked, no errors */
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_main() -> Result<()> {
        let path = String::from("./tests/test1.ql");
        let args = vec![path.clone(), "--analyze".into()];
        let parser: Parser = Default::default();

        if let Some(config) = parser.parse_cmdline(args)? {
            let qast = parser.parse(&config.analyzer.src)?;
            println!("{qast}");
        }

        Ok(())
    }
}
