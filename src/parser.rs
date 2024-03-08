//! Parser for quale language.
//! It translates the given code into an AST.
use crate::ast::{Qast, Token};
use crate::attributes::Attributes;
use crate::config::*;
use crate::error::{QccError, QccErrorKind, QccErrorLoc, Result};
use crate::lexer::{Lexer, Location};
use crate::utils::usage;
use std::path::Path;

#[derive(Default)]
pub struct Parser {}

impl Parser {
    /// Parses the cmdline arguments and populate the `Config` options. This
    /// configuration persists for an entire compilation session.
    pub fn parse_cmdline(&self, args: Vec<String>) -> Result<Option<Config>> {
        if args.len() < 1 {
            Err(QccErrorKind::InvalidArgs)?;
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
                    _ => Err(QccErrorKind::NoSuchArg)?,
                }
            } else {
                config.analyzer.src = option;
            }
        }

        let path = &config.analyzer.src;
        if path.is_empty() {
            Err(QccErrorKind::NoFile)?;
        }

        if !Path::new(&path).is_file() {
            Err(QccErrorKind::NoFile)?;
        }

        Ok(Some(config))
    }

    /// Parses the source file.
    pub fn parse(&self, src: &String) -> Result<Qast> {
        let lines = std::fs::read(src)?;
        let mut lexer: Lexer = Lexer::new(&lines, src);
        let mut qast: Qast = Default::default();
        let mut attrs: Attributes = Default::default();
        let mut attr_assoc = false; // Has the parsed attribute been
                                    // associated with a function yet?
        let mut is_fn = false;
        let mut seen_errors = false;

        while let Some(token) = lexer.next_token() {
            match lexer.last_token.unwrap() {
                Token::Identifier => {
                    if is_fn {
                        if !attr_assoc {
                            qast.append_function(
                                lexer.identifier(),
                                lexer.location.clone(),
                                attrs.clone(),
                            );
                            attr_assoc = true;
                        } else {
                            qast.append_function(
                                lexer.identifier(),
                                lexer.location.clone(),
                                Default::default(),
                            );
                        }
                        is_fn = false;
                    }
                }
                Token::Literal => {}
                Token::Attribute => match lexer.identifier().parse::<Attributes>() {
                    Ok(a) => {
                        attrs = a;
                        attr_assoc = false;
                    }

                    Err(partial_err) => {
                        seen_errors = true;
                        let row = lexer.location.row();
                        let mut col = partial_err.get_loc().borrow().col();
                        let row_s = format!("{}", row);

                        let loc = Location::new(
                            lexer.location.path().as_str(),
                            lexer.location.row(),
                            col,
                        );

                        let full_err: QccErrorLoc = (partial_err, loc).into();
                        eprintln!("{full_err}");

                        let builder = format!("\t{}\t{}", row_s, lexer.identifier());
                        eprintln!("{builder}");

                        col += 2 + row_s.len(); // for inserted tabs

                        for c in builder.chars() {
                            if col > 0 {
                                col -= 1;
                            } else {
                                eprintln!("^");
                                break;
                            }
                            if c.is_whitespace() {
                                eprint!("{c}");
                            } else {
                                eprint!(" ");
                            }
                        }
                    }
                },
                Token::Function => {
                    is_fn = true;
                }
            };
            lexer.consume(token);
        }

        // If there is at least one attribute which is not associated with any
        // function, it is a semantic error.
        if !attr_assoc && !attrs.is_empty() {
            let err = QccErrorLoc::new(QccErrorKind::ExpectedFnForAttr, lexer.location);
            eprintln!("{err}");
            Err(QccErrorKind::ParseError)?
        }

        if seen_errors {
            Err(QccErrorKind::ParseError)?
        } else {
            Ok(qast)
        }
    }
}
