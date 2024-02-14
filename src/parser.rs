//! Parser for quale language.
//! It translates the given code into an AST.
use crate::ast::{Qast};
use crate::config::*;
use crate::error::Result;
use crate::lexer::Lexer;
use crate::utils::usage;
use std::path::Path;

pub(crate) struct Parser {
    ast: Qast,
}

impl Parser {}

/// Parses the source file.
pub fn parse_src(src: &String) -> Result<Qast> {
    // TODO: Check for file existence.
    let lines = std::fs::read(src)?;
    let mut lexer = Lexer::new(&lines, src);

    while let Some(token) = lexer.next_token() {
        lexer.consume(token);
    }

    Ok(Qast::new(vec![]))
}

/// Parses the cmdline arguments and populate the `Config` options. This
/// configuration persists for an entire compilation session.
pub fn parse_cmdline(args: Vec<String>) -> Result<Option<Config>> {
    if args.is_empty() {
        Err("no input files")?;
    }

    let mut config = Config::new();

    // Parse cmdline options
    for option in args {
        if option.starts_with("--") {
            if option == "--analyze" {
                config.analyzer.status = true;
            }
            if option == "--help" || option == "-h" {
                usage();
                return Ok(None);
            }
        } else if option.starts_with('-') {
            // Parse opt level
            match option.as_str() {
                "-O0" => config.optimizer.level = 0,
                "-O1" => config.optimizer.level = 1,
                "-O2" => config.optimizer.level = 2,
                "-Og" => config.optimizer.level = 3,
                "-h" => {
                    usage();
                    return Ok(None);
                }
                _ => unreachable!(),
            }
        } else {
            config.analyzer.src = option;
        }
    }

    let path = &config.analyzer.src;

    if path.is_empty() {
        Err("provide a quale file to compile")?;
    }

    if !Path::new(&path).is_file() {
        Err(format!("{path} doesn't exist"))?;
    }

    Ok(Some(config))
}
