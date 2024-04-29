//! Parser for quale language.
//! It translates the given code into an AST.
use crate::ast::*;
use crate::attributes::{Attribute, Attributes};
use crate::config::*;
use crate::error::{QccError, QccErrorKind, QccErrorLoc, Result};
use crate::lexer::{Lexer, Location};
use crate::types::Type;
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
                    "--help" => {
                        usage();
                        return Ok(None);
                    }
                    "--analyze" => config.analyzer.status = true,
                    "--dump-ast" => config.dump_ast = true,
                    "--dump-ast-only" => config.dump_ast_only = true,
                    "--dump-qasm" => config.dump_qasm = true,
                    _ => {
                        let err: QccError = QccErrorKind::NoSuchArg.into();
                        err.report(option);
                        return Err(QccErrorKind::CmdlineErr)?;
                    }
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
                    _ => {
                        let err: QccError = QccErrorKind::NoSuchArg.into();
                        err.report(option);
                        return Err(QccErrorKind::CmdlineErr)?;
                    }
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

        if self.lexer.is_token(Token::Bang) {
            self.lexer.consume(Token::Bang)?;
        }

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
            return Err(QccErrorKind::ExpectedFn)?;
        }

        self.lexer.consume(Token::Function)?;

        if !self.lexer.is_token(Token::Identifier) {
            return Err(QccErrorKind::ExpectedFnName)?;
        }

        let name = self.lexer.identifier();
        let location = self.lexer.location.clone();
        let mut params: Vec<VarAST> = Default::default();

        self.lexer.consume(Token::Identifier)?;

        if !self.lexer.is_token(Token::OParenth) {
            return Err(QccErrorKind::ExpectedFnArgs)?;
        }
        self.lexer.consume(Token::OParenth)?;

        while !self.lexer.is_token(Token::CParenth) {
            if self.lexer.is_token(Token::Identifier) {
                let name = self.lexer.identifier();
                let location = self.lexer.location.clone();
                self.lexer.consume(Token::Identifier)?;

                if !self.lexer.is_token(Token::Colon) {
                    return Err(QccErrorKind::ExpectedParamType)?;
                }
                self.lexer.consume(Token::Colon)?;

                if !self.lexer.is_token(Token::Identifier) {
                    return Err(QccErrorKind::ExpectedParamType)?;
                }

                let type_ = self.lexer.identifier().parse::<Type>()?;
                self.lexer.consume(Token::Identifier)?;

                params.push(VarAST::new_with_type(name, location, type_));
            }

            if !self.lexer.is_token(Token::Comma) && !self.lexer.is_token(Token::CParenth) {
                return Err(QccErrorKind::ExpectedAttr)?;
            }

            if self.lexer.is_token(Token::Comma) {
                self.lexer.consume(Token::Comma)?;
            }
        }
        self.lexer.consume(Token::CParenth)?;

        // Parse function return type
        let mut output_type = Default::default();

        if self.lexer.is_token(Token::Colon) {
            self.lexer.consume(Token::Colon)?;

            // TODO: exponential type - linear logic
            if self.lexer.is_token(Token::Bang) {
                self.lexer.consume(Token::Bang)?;
            }

            if !self.lexer.is_token(Token::Identifier) {
                return Err(QccErrorKind::ExpectedFnReturnType)?;
            }

            output_type = self.lexer.identifier().parse::<Type>()?;
            self.lexer.consume(Token::Identifier)?;
        }

        if !self.lexer.is_token(Token::OCurly) {
            return Err(QccErrorKind::ExpectedFnBody)?;
        }
        self.lexer.consume(Token::OCurly)?;

        let mut body: Vec<Box<Expr>> = Default::default();
        while !self.lexer.is_token(Token::CCurly) {
            if self.lexer.is_token(Token::Let) {
                let expr = self.parse_let()?;
                body.push(expr);
            } else if self.lexer.is_token(Token::Return) {
                let expr = self.parse_return()?;
                body.push(expr);
            } else {
                if self.lexer.token.is_some() {
                    self.lexer.consume(self.lexer.token.unwrap());
                } else {
                    break;
                }
            }
        }
        self.lexer.consume(Token::CCurly)?;

        Ok(FunctionAST::new(
            name,
            location,
            params,
            output_type,
            attrs,
            body,
        ))
    }

    fn parse_return(&mut self) -> Result<Box<Expr>> {
        if self.lexer.is_token(Token::Return) {
            self.lexer.consume(Token::Return)?;
            let expr = self.parse_expr()?;
            return Ok(expr);
        } else {
            let expr = self.parse_expr()?;

            if !self.lexer.is_token(Token::CCurly) {
                return Err(QccErrorKind::ExpectedFnBodyEnd)?;
            }

            return Ok(expr);
        }
    }

    fn parse_fn_call(&mut self) -> Result<Box<Expr>> {
        if !self.lexer.is_token(Token::Identifier) {
            return Err(QccErrorKind::ExpectedFn)?;
        }
        let name = self.lexer.identifier();
        self.lexer.consume(Token::Identifier)?;

        if !self.lexer.is_token(Token::OParenth) {
            return Err(QccErrorKind::ExpectedParenth)?;
        }
        self.lexer.consume(Token::OParenth)?;

        let mut args: Vec<Box<Expr>> = vec![];
        while !self.lexer.is_token(Token::CParenth) {
            let expr = self.parse_expr();
            if expr.is_ok() {
                args.push(expr.unwrap());
            } else {
                return Err(QccErrorKind::UnexpectedExpr)?;
            }

            if self.lexer.is_token(Token::Comma) {
                self.lexer.consume(Token::Comma)?;
            }
        }

        let function = FunctionAST::new(
            name,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        Ok(Box::new(Expr::FnCall(function, args)))
    }

    /// An expression can be of following kinds:
    /// - literal : string, digit, etc.
    /// - already defined var.
    /// - function call
    /// - binary expression
    fn parse_expr(&mut self) -> Result<Box<Expr>> {
        if self.lexer.is_token(Token::Digit) {
            let d = self.lexer.digit();
            if d.is_none() {
                return Err(QccErrorKind::UnexpectedDigit)?;
            }

            let expr = LiteralAST::Lit_Digit((d.unwrap()));
            self.lexer.consume(Token::Digit)?;

            // TODO: Digit may be part of binary expression.
            return Ok(Box::new(Expr::Literal(Box::new(expr))));
        } else if self.lexer.is_any_token(&[Token::Identifier, Token::Sub]) {
            if self.lexer.is_token(Token::Sub) {
                // TODO: unary negative named variable
                self.lexer.consume(Token::Sub)?;
            }

            let name = self.lexer.identifier();
            let location = self.lexer.location.clone();
            self.lexer.consume(Token::Identifier)?;

            if self.lexer.is_token(Token::OParenth) {
                // parse a function call
                self.lexer.consume(Token::OParenth)?;

                let mut args: Vec<Box<Expr>> = vec![];
                while !self.lexer.is_token(Token::CParenth) {
                    let expr = self.parse_expr();
                    if expr.is_ok() {
                        args.push(expr.unwrap());
                    } else {
                        break;
                    }

                    if !self.lexer.is_any_token(&[Token::Comma, Token::CParenth]) {
                        if !self.lexer.is_token(Token::Comma) {
                            return Err(QccErrorKind::ExpectedComma)?;
                        } else {
                            return Err(QccErrorKind::ExpectedParenth)?;
                        }
                    }
                    if self.lexer.is_token(Token::Comma) {
                        self.lexer.consume(Token::Comma)?;
                    }
                }
                if !self.lexer.is_token(Token::CParenth) {
                    return Err(QccErrorKind::ExpectedParenth)?;
                }
                self.lexer.consume(Token::CParenth)?;

                if !self.lexer.is_token(Token::Semicolon) {
                    return Err(QccErrorKind::ExpectedSemicolon)?;
                }
                self.lexer.consume(Token::Semicolon)?;

                let function = FunctionAST::new(
                    name,
                    Default::default(), // location if found during
                    // type checking
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                );

                Ok(Box::new(Expr::FnCall(function, args)))
            } else if self.lexer.is_token(Token::Semicolon) {
                // only a variable
                let expr = Expr::Var(VarAST::new(name, location));
                self.lexer.consume(Token::Semicolon)?;

                Ok(Box::new(expr))
            } else {
                // parse a binary expression
                let op: Opcode;

                // get the opcode
                if self
                    .lexer
                    .is_any_token(&[Token::Add, Token::Sub, Token::Mul, Token::Div])
                {
                    // opcode in binary expression
                    op = self.lexer.identifier().parse()?;
                    self.lexer.consume(self.lexer.token.unwrap())?;
                } else {
                    // single named variable is enclosed in parenthesis
                    if self.lexer.is_token(Token::CParenth) {
                        return Ok(Box::new(Expr::Var(VarAST::new(name, location))));
                    }

                    return Err(QccErrorKind::ExpectedOpcode)?;
                }

                // parse rhs of binary expression
                if !self.lexer.is_token(Token::Identifier)
                    && !self.lexer.is_token(Token::Digit)
                    && !self.lexer.is_token(Token::OParenth)
                {
                    return Err(QccErrorKind::ExpectedExpr)?;
                }

                if self.lexer.is_token(Token::Identifier) {
                    // rhs is a named variable
                    let name_rhs = self.lexer.identifier();
                    let loc_rhs = self.lexer.location.clone();

                    self.lexer.consume(Token::Identifier)?;

                    let lhs = Expr::Var(VarAST::new(name, location));
                    let rhs = Expr::Var(VarAST::new(name_rhs, loc_rhs));

                    Ok(Box::new(Expr::BinaryExpr(Box::new(lhs), op, Box::new(rhs))))
                } else if self.lexer.is_token(Token::Digit) {
                    // rhs is a digit
                    let digit = self.parse_expr()?;
                    let lit: Box<LiteralAST>;

                    match *digit {
                        Expr::Literal(l) => lit = l,
                        _ => unreachable!(),
                    }

                    let lhs = Expr::Var(VarAST::new(name, location));
                    let rhs = Expr::Literal(lit);

                    Ok(Box::new(Expr::BinaryExpr(Box::new(lhs), op, Box::new(rhs))))
                } else if self.lexer.is_token(Token::OParenth) {
                    // rhs is a sub binary expression
                    let mut parenth_deep: usize = 1; // pairs of parentheses
                                                     // should become 0 when
                                                     // complete binary
                                                     // expression is returned
                    self.lexer.consume(Token::OParenth)?;

                    let subexpr = self.parse_expr()?;

                    if !self.lexer.is_token(Token::CParenth) {
                        return Err(QccErrorKind::ExpectedParenth)?;
                    }
                    self.lexer.consume(Token::CParenth)?;

                    let lhs = Expr::Var(VarAST::new(name, location));

                    Ok(Box::new(Expr::BinaryExpr(Box::new(lhs), op, subexpr)))
                } else {
                    return Err(QccErrorKind::UnknownBinaryExpr)?;
                }
            }
        } else {
            // TODO: Complex binary expressions
            return Err(QccErrorKind::ExpectedExpr)?;
        }
    }

    // TODO: a newer approach with adts
    fn parse_binary_expr(&mut self) -> Result<Box<BinaryExprAST>> {
        if self.lexer.is_token(Token::Identifier) {
            let name = self.lexer.identifier();
            let location = self.lexer.location.clone();
            self.lexer.consume(Token::Identifier)?;

            if self.lexer.is_token(Token::OParenth) {
                // definitely a function call
                self.lexer.consume(Token::OParenth)?;
                let mut args = vec![];
                while !self.lexer.is_token(Token::CParenth) {
                    let arg = self.parse_binary_expr()?;
                    args.push(arg);

                    if !self.lexer.is_token(Token::Comma) && !self.lexer.is_token(Token::CParenth) {
                        return Err(QccErrorKind::ExpectedAttr)?;
                    }
                    if self.lexer.is_token(Token::Comma) {
                        self.lexer.consume(Token::Comma)?;
                    }
                }
                self.lexer.consume(Token::CParenth)?;
                let f = FunctionAST::new(
                    name,
                    location,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                );
                return Ok(Box::new(BinaryExprAST::FnCall(f, args)));
            } else if self
                .lexer
                .is_any_token(&[Token::Add, Token::Sub, Token::Mul, Token::Div])
            {
                // (maybe complex) binary expression
                let op = self.lexer.identifier().parse::<Opcode>()?;

                if self.lexer.is_token(Token::OParenth) {
                    // a complex binary expression
                    let mut parenth_depth = 1;
                    while parenth_depth != 0
                        && self.lexer.is_any_token(&[
                            Token::Add,
                            Token::Sub,
                            Token::Mul,
                            Token::Div,
                        ])
                    {
                        let inner_expr = self.parse_binary_expr()?;
                    }
                } else {
                    let lhs = Box::new(BinaryExprAST::Var(VarAST::new(name, location)));

                    if self.lexer.is_token(Token::Identifier) {
                        let rhs = Box::new(BinaryExprAST::Var(VarAST::new(
                            self.lexer.identifier(),
                            self.lexer.location.clone(),
                        )));
                        self.lexer.consume(Token::Identifier)?;
                        return Ok(Box::new(BinaryExprAST::BinaryExpr(lhs, op, rhs)));
                    } else if self.lexer.is_token(Token::Digit) {
                        let digit = self.lexer.digit();
                        if digit.is_none() {
                            return Err(QccErrorKind::ExpectedExpr)?;
                        }
                        return Ok(Box::new(BinaryExprAST::Literal(Box::new(
                            LiteralAST::Lit_Digit(digit.unwrap()),
                        ))));
                    } else {
                        return Err(QccErrorKind::ExpectedExpr)?;
                    }
                }
            } else if self.lexer.is_token(Token::Semicolon) {
                // is simply a named variable
                self.lexer.consume(Token::Semicolon)?;
                return Ok(Box::new(BinaryExprAST::Var(VarAST::new(name, location))));
            } else {
                return Err(QccErrorKind::UnexpectedStr)?;
            }
        } else if self.lexer.is_token(Token::Digit) {
            // it must be a digit
            let digit = self.lexer.digit();
            self.lexer.consume(Token::Digit)?;
            if digit.is_none() {
                return Err(QccErrorKind::ExpectedExpr)?;
            }
            return Ok(Box::new(BinaryExprAST::Literal(Box::new(
                LiteralAST::Lit_Digit(digit.unwrap()),
            ))));
        } else {
            // TODO: ??
            Err(QccErrorKind::ExpectedExpr)?
        }

        // a binary expression can be: a digit, a named variable, fn call or a
        // complex binary expression.
        if self
            .lexer
            .is_any_token(&[Token::Add, Token::Sub, Token::Mul, Token::Div])
        {
            Err(QccErrorKind::CmdlineErr)?
        } else if self.lexer.is_token(Token::Identifier) {
            let name = self.lexer.identifier();
            let location = self.lexer.location.clone();

            if self.lexer.is_token(Token::OParenth) {
                self.lexer.consume(Token::OParenth)?;
                let mut args = vec![];
                while !self.lexer.is_token(Token::CParenth) {
                    let arg = self.parse_binary_expr()?;
                    args.push(arg);

                    if !self.lexer.is_token(Token::Comma) && !self.lexer.is_token(Token::CParenth) {
                        return Err(QccErrorKind::ExpectedAttr)?;
                    }
                    if self.lexer.is_token(Token::Comma) {
                        self.lexer.consume(Token::Comma)?;
                    }
                }
                self.lexer.consume(Token::CParenth)?;
                let f = FunctionAST::new(
                    name,
                    location,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                );
                Ok(Box::new(BinaryExprAST::FnCall(f, args)))
            } else if self.lexer.is_token(Token::Semicolon) {
                Ok(Box::new(BinaryExprAST::Var(VarAST::new(name, location))))
            } else {
                Err(QccErrorKind::CmdlineErr)?
            }
        } else if self.lexer.is_token(Token::Digit) {
            let digit = self.lexer.digit();
            if digit.is_none() {
                return Err(QccErrorKind::ExpectedExpr)?;
            }
            self.lexer.consume(Token::Digit)?;
            Ok(Box::new(BinaryExprAST::Literal(Box::new(
                LiteralAST::Lit_Digit(digit.unwrap()),
            ))))
        } else {
            Err(QccErrorKind::ExpectedExpr)?
        }
    }

    fn parse_let(&mut self) -> Result<Box<Expr>> {
        if !self.lexer.is_token(Token::Let) {
            return Err(QccErrorKind::ExpectedLet)?;
        }
        self.lexer.consume(Token::Let)?;

        if !self.lexer.is_token(Token::Identifier) {
            return Err(QccErrorKind::ExpectedLet)?;
        }

        let name = self.lexer.identifier();
        let location = self.lexer.location.clone();
        let mut var = VarAST::new(name, location); // lhs
        self.lexer.consume(Token::Identifier)?;

        // Parse given type if available
        if self.lexer.is_token(Token::Colon) {
            self.lexer.consume(Token::Colon)?;
            if !self.lexer.is_token(Token::Identifier) {
                return Err(QccErrorKind::ExpectedType)?;
            }
            let type_ = self.lexer.identifier().parse::<Type>()?;
            var.set_type(type_);
            self.lexer.consume(Token::Identifier)?;
        }

        if !self.lexer.is_token(Token::Assign) {
            return Err(QccErrorKind::ExpectedAssign)?;
        }
        self.lexer.consume(Token::Assign)?;

        let val = self.parse_expr()?;

        Ok(Box::new(Expr::Let(var, val)))
    }

    fn parse_module(&mut self) -> Result<ModuleAST> {
        if !self.lexer.is_token(Token::Module) {
            return Err(QccErrorKind::ExpectedMod)?;
        }
        let location = self.lexer.location.clone();
        self.lexer.consume(Token::Module)?;

        let mut name: String = String::from("unnamed");

        if self.lexer.is_token(Token::Identifier) {
            name = self.lexer.identifier();
            self.lexer.consume(Token::Identifier)?;
        }

        if !self.lexer.is_token(Token::OCurly) {
            return Err(QccErrorKind::ExpectedMod)?;
        }
        self.lexer.consume(Token::OCurly)?;

        let mut functions: Vec<FunctionAST> = Default::default();
        while !self.lexer.is_token(Token::CCurly) {
            let function = self.parse_function();
            if function.is_ok() {
                functions.push(function?);
            } else {
                self.lexer.consume(self.lexer.token.unwrap())?;
            }
        }

        self.lexer.consume(Token::CCurly)?;

        Ok(ModuleAST::new(name, location, functions))
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

        let module_basename = src.rsplit_once('/');
        let mut module_name: &str;
        if module_basename.is_none() {
            module_name = src;
        } else {
            (_, module_name) = module_basename.unwrap();
        }
        // TODO: We need a mangler for sanitizing module name.
        let module_name = module_name.trim_end_matches(".ql").into();
        let module_location = Location::new(src, 1, 1);
        qast.add_module_info(module_name, module_location);

        // TODO: Move this entirely in parse_module, parse_module should return
        // a Qast and it can recursively call itself when `module` is seen
        // inside the file.
        self.lexer.next_token()?;
        loop {
            if self.lexer.token.is_none() {
                break;
            }
            if self.lexer.is_token(Token::Module) {
                match self.parse_module() {
                    Ok(module) => println!("{module}"),
                    Err(e) => {
                        seen_errors = true;

                        let err: QccErrorLoc = (e, self.lexer.location.clone()).into();
                        err.report(self.lexer.line());
                    }
                }
            } else if self.lexer.is_token(Token::Hash) || self.lexer.is_token(Token::Function) {
                match self.parse_function() {
                    Ok(f) => qast.append(f),
                    Err(e) => {
                        seen_errors = true;

                        let err: QccErrorLoc = (e, self.lexer.location.clone()).into();
                        err.report(self.lexer.line());
                    }
                }
            } else {
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
