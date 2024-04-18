//! Parser for quale language.
//! It translates the given code into an AST.
use crate::ast::{FunctionAST, Qast, Token};
use crate::attributes::{Attribute, Attributes};
use crate::config::*;
use crate::error::{QccErrorKind, QccErrorLoc, Result};
use crate::lexer::{Lexer, Location};
use crate::utils::usage;
use std::path::Path;

pub struct Parser {
    // args: Vec<String>,
    config: Config,
    lexer: Box<Lexer>,
}

impl Parser {
    /// Create a new parser object depending upon the command-line arguments. In
    /// following situations a parser will not be returned:
    /// - If `help` arg was passed, it returns `None` wrapped as Result.
    /// - If some error occurs, returns the error.
    pub fn new(args: Vec<&str>) -> Result<Option<Self>> {
        if let Some(config) = Parser::parse_cmdline(args)? {
            let lines = std::fs::read(&config.analyzer.src)?;
            let mut lexer = Lexer::new(lines, config.analyzer.src.clone());

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
    pub fn get_config(&self) -> Config {
        self.config.clone()
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
                match option {
                    "--help" | "-h" => {
                        usage();
                        return Ok(None);
                    }
                    "--analyze" => config.analyzer.status = true,
                    "--dump-ast" => config.dump_ast = true,
                    "--dump-ast-only" => config.dump_ast_only = true,
                    "--dump-qasm" => config.dump_qasm = true,
                    _ => Err(QccErrorKind::NoSuchArg)?,
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

    /// Parses a list of comma-separated attributes.
    fn parse_attributes(&mut self) -> Result<Attributes> {
        if !self.lexer.is_token(Token::Hash) {
            return Err(QccErrorKind::ExpectedAttr)?;
        }
        self.lexer.consume(Token::Hash)?;

        if !self.lexer.is_token(Token::OBracket) {
            return Err(QccErrorKind::ExpectedAttr)?;
        }
        self.lexer.consume(Token::OBracket)?;

        let mut attrs: Attributes = Default::default();

        if !self.lexer.is_token(Token::Identifier) {
            return Err(QccErrorKind::ExpectedAttr)?;
        }

        while !self.lexer.is_token(Token::CBracket) {
            if self.lexer.is_token(Token::Identifier) {
                let attr = self.lexer.identifier().parse::<Attribute>()?;
                attrs.push(attr);
                self.lexer.consume(Token::Identifier)?;
            }

            if !self.lexer.is_token(Token::Comma) && !self.lexer.is_token(Token::CBracket) {
                return Err(QccErrorKind::ExpectedAttr)?;
            }

            if self.lexer.is_token(Token::Comma) {
                self.lexer.consume(Token::Comma)?;
            }
        }
        self.lexer.consume(Token::CBracket)?;

        Ok(attrs)
    }

    /// Parses a function.
    fn parse_function(&mut self) -> Result<FunctionAST> {
        let mut attrs: Attributes = Default::default();

        if self.lexer.token == Some(Token::Hash) {
            attrs = self.parse_attributes()?;
        }

        if !self.lexer.is_token(Token::Function) {
            return Err(QccErrorKind::ExpectedFnForAttr)?;
        }

        self.lexer.consume(Token::Function)?;

        if !self.lexer.is_token(Token::Identifier) {
            // TODO: return Err(QccErrorKind::ExpectedFnName)?;
            return Err(QccErrorKind::ExpectedFnForAttr)?;
        }

        let name = self.lexer.identifier();
        let location = self.lexer.location.clone();

        self.lexer.consume(Token::Identifier)?;

        Ok(FunctionAST::new(name, location, attrs))
    }

    /* TODO: If we have more than one quale file in a parsing session
     * (inside Config), then we can select which one to parse via here */
    /// Parses the source file.
    pub fn parse(&mut self, src: &String) -> Result<Qast> {
        if !src.ends_with(".ql") {
            Err(QccErrorKind::ParseError)?
        }

        let mut qast: Qast = Default::default();
        let mut seen_errors = false;

        self.lexer.next_token()?;
        loop {
            if self.lexer.token.is_none() {
                break;
            }
            if self.lexer.is_token(Token::Hash) || self.lexer.is_token(Token::Function) {
                match self.parse_function() {
                    Ok(f) => qast.append(f),
                    Err(e) => {
                        seen_errors = true;
                        // TODO: Move this to a standalone mechanism. That is,
                        // the source location dumping alongside error.
                        let err: QccErrorLoc = (e, self.lexer.location.clone()).into();
                        eprintln!("{}", err);

                        let row_s = self.lexer.location.row().to_string();
                        let mut col = self.lexer.location.col();

                        let builder = format!("\t{}\t{}", row_s, self.lexer.line());
                        eprint!("{builder}");

                        col += 1 + row_s.len(); // for inserted tabs

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
                }
            } else {
                // TODO: Typing this over and over is no fun. This is a
                // scope for research.
                self.lexer.consume(self.lexer.token.unwrap())?;
            }
        }

        if seen_errors {
            Err(QccErrorKind::ParseError)?
        } else {
            Ok(qast)
        }
    }
}
