//! Parser for quale language.
//! It translates the given code into an AST.
use crate::ast::{FunctionAST, Qast, Token};
use crate::attributes::Attributes;
use crate::config::*;
use crate::error::{QccErrorKind, QccErrorLoc, Result};
use crate::lexer::{Lexer, Location};
use crate::utils::usage;
use std::path::Path;

pub struct Parser {
    // args: Vec<String>,
    config: Config,
    lexer: std::cell::RefCell<Lexer>,
}

impl Parser {
    /// Create a new parser object depending upon the command-line arguments. In
    /// following situations a parser will not be returned:
    /// - If `help` arg was passed, it returns `None` wrapped as Result.
    /// - If some error occurs, returns the error.
    pub fn new(args: Vec<&str>) -> Result<Option<Self>> {
        if let Some(config) = Parser::parse_cmdline(args)? {
            let lines = std::fs::read(&config.analyzer.src)?;
            let lexer = Lexer::new(lines, config.analyzer.src.clone());

            Ok(Some(Self {
                config,
                lexer: lexer.into(),
            }))
        } else {
            // if help is asked, return without creating an object
            Ok(None)
        }
    }

    /// Returns a reference to `Config` for current parser session.
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Parses the cmdline arguments and populate the `Config` options. This
    /// configuration persists for an entire compilation session.
    ///
    /// Though, it can be used on its own, but `Parser::new` calls it underneath
    /// to construct a `Config`, so if you are dealing with the parser, use
    /// `Parser::new` instead.
    pub fn parse_cmdline(args: Vec<&str>) -> Result<Option<Config>> {
        if args.len() < 1 {
            Err(QccErrorKind::InvalidArgs)?;
        }

        let mut config = Config::new();
        let mut output_direct: u8 = 0x0;

        // Parse cmdline options
        for option in args {
            if option.starts_with("--") {
                if option == "--help" || option == "-h" {
                    usage();
                    return Ok(None);
                }
                if option == "--analyze" {
                    config.analyzer.status = true;
                }
                if option == "--dump-ast" {
                    config.dump_ast = true;
                }
            } else if option.starts_with('-') {
                // Parse opt level
                match option {
                    "-O0" => config.optimizer.level = 0x0,
                    "-O1" => config.optimizer.level = 0x1,
                    "-O2" => config.optimizer.level = 0x2,
                    "-Og" => config.optimizer.level = 0x3,
                    "-o" => output_direct |= 0x1,
                    "-h" => {
                        usage();
                        return Ok(None);
                    }
                    _ => Err(QccErrorKind::NoSuchArg)?,
                }
            } else {
                if output_direct == 0x1 {
                    config.optimizer.asm = option.into();
                    output_direct <<= 0x1;
                } else {
                    config.analyzer.src = option.into();
                    if output_direct == 0x0 {
                        config.optimizer.asm = option.replace(".ql", ".s");
                    }
                }
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

    /// Parses attribute.
    fn parseAttrs(&self) -> Result<Attributes> {
        Ok(Attributes(vec![]))
    }

    fn parseFunction(&self) -> Result<FunctionAST> {
        // FIXME: Can't do this because lexer is in immutable state.
        let attrs = self.parseAttrs()?;

        Ok(FunctionAST::new("main".into(), Location::default(), attrs))
    }

    /// Parses the source file.
    /* TODO: If we have more than one quale file in a parsing session
     * (inside Config), then we can select which one to parse via here */
    pub fn parse(&self, src: &String) -> Result<Qast> {
        if !src.ends_with(".ql") {
            Err(QccErrorKind::ParseError)?
        }

        let mut qast: Qast = Default::default();
        let mut attrs: Attributes = Default::default();
        let mut attr_assoc = false; // Has the parsed attribute been
                                    // associated with a function yet?
        let mut is_fn = false;
        let mut seen_errors = false;

        let _fns: Vec<FunctionAST> = vec![];
        let mut lexer = self.lexer.borrow_mut();

        // // TODO:
        // while let Some(token) = lexer.next_token() {
        //     let _fn = self.parseFunction()?;
        //     fns.push(_fn);
        // }
        // return Ok(Qast::new(fns));

        while let Some(token) = lexer.next_token()? {
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
                        // TODO: Move this to a standalone mechanism. That is,
                        // the source location dumping alongside error.
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
            let err = QccErrorLoc::new(QccErrorKind::ExpectedFnForAttr, lexer.location.clone());
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
