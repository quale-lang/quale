//! quale compiler framework
mod analyzer;
mod opt;
mod config;
mod parser;
mod ast;
mod error;

use crate::config::*;
use crate::parser::parse_src;
use std::path::Path;
use crate::error::Result;

fn usage() {
    print!(
        "usage: qcc [options] <quale-file>
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
",
        "--help", "show this page",
        "--analyze", "run static analyzer",
        "-O0", "disable optimizations",
        "-O1", "enable first-level optimizations",
        "-Og", "enable all optimizations",
        "-o", "compiled output"
    );
}

/// Parses the cmdline arguments and populate the `Config` options. This
/// configuration persists for an entire compilation session.
fn parse_cmdline(args: Vec<String>) -> Result<Option<Config>> {

    if args.len() < 1 {
        Err("no input files")?;
    }

    let mut config = Config::new();

    // Parse cmdline options
    for option in args {
        if option.starts_with("--") {
            if option == "--analyze" {
                config.analyzer.status = true;
            }
            if option == "--help" {
                usage();
                return Ok(None);
            }
        }
        else if option.starts_with('-') {
            // Parse opt level
            match option.as_str() {
                "-O0" => config.optimizer.level = 0,
                "-O1" => config.optimizer.level = 1,
                "-Og" => config.optimizer.level = 2,
                "-h"  => {
                    usage();
                    return Ok(None);
                },
                _ => unreachable!(),
            }
        }
        else {
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

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(config) = parse_cmdline(args)? {
        let _ast = parse_src(&config.analyzer.src)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_check() {
        let paths = std::fs::read_dir("./tests").unwrap();

        for p in paths {
            let path = p.unwrap().path().into_os_string().into_string().unwrap();
            let args = vec![path];
            let _ = parse_cmdline(args);
        }
    }
}
