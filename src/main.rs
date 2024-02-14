//! quale compiler framework
mod analyzer;
mod ast;
mod config;
mod error;
mod lexer;
mod opt;
mod parser;
mod types;
mod utils;

use crate::error::Result;
use crate::parser::{parse_cmdline, parse_src};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(config) = parse_cmdline(args)? {
        let _ast = parse_src(&config.analyzer.src)?;
    }

    Ok(())
}
