//! Parser for quale language.
//! It translates the given code into an AST.
use crate::ast::*;
use crate::attributes::{Attribute, Attributes};
use crate::config::*;
use crate::error::{QccError, QccErrorKind, QccErrorLoc, Result};
use crate::lexer::{Lexer, Location};
use crate::mangler::{mangle, mangle_module, sanitize};
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
        let mut input_type: Vec<Type> = Default::default();

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

                input_type.push(type_.clone());
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

        let mut body: Vec<QccCell<Expr>> = Default::default();
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
            input_type,
            output_type,
            attrs,
            body,
        ))
    }

    /// Parses the import statement and returns a pair of module name and
    /// function name that is being imported.
    fn parse_import(&mut self, qast: &Qast) -> core::result::Result<(Ident, Ident), QccErrorLoc> {
        self.lexer.consume(Token::Import)?;

        if !self.lexer.is_token(Token::Identifier) {
            return Err(QccErrorKind::ExpectedMod)?;
        }
        let mod_name = self.lexer.identifier();
        let mod_location = self.lexer.location.clone();
        self.lexer.consume(Token::Identifier)?;

        // TODO: Colon location in error reporting is incorrect.
        if !self.lexer.is_token(Token::Colon) {
            return Err(QccErrorKind::ExpectedColon)?;
        }
        self.lexer.consume(Token::Colon)?;
        if !self.lexer.is_token(Token::Colon) {
            return Err(QccErrorKind::ExpectedColon)?;
        }
        self.lexer.consume(Token::Colon)?;

        if !self.lexer.is_token(Token::Identifier) {
            return Err(QccErrorKind::ExpectedFnName)?;
        }
        let fn_name = self.lexer.identifier();
        let fn_location = self.lexer.location.clone();
        self.lexer.consume(Token::Identifier)?;

        if !self.lexer.is_token(Token::Semicolon) {
            return Err(QccErrorKind::ExpectedSemicolon)?;
        }
        self.lexer.consume(Token::Semicolon);

        // TODO: Move these checks when mod_name and fn_name are parsed. That
        // way it can return QccErrorLoc back. But this may be more costly!
        let mut unknown_module = true;
        for module in qast {
            if module.get_name() == mod_name {
                unknown_module = false;
                for function in &*module {
                    if *function.get_name() == fn_name {
                        return Ok((mod_name, fn_name));
                    }
                }
            }
        }

        if unknown_module {
            Err((QccErrorKind::UnknownModName, mod_location))?
        } else {
            Err((QccErrorKind::UnknownImport, fn_location))?
        }
    }

    fn parse_return(&mut self) -> Result<QccCell<Expr>> {
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

    /// It parses a function call with its arguments. The `name` and `location`
    /// of function call is seen already so it simply appends this information.
    fn parse_fn_call_args(&mut self, name: Ident, location: Location) -> Result<QccCell<Expr>> {
        if !self.lexer.is_token(Token::OParenth) {
            return Err(QccErrorKind::ExpectedParenth)?;
        }
        self.lexer.consume(Token::OParenth)?;

        let mut args: Vec<QccCell<Expr>> = vec![];
        while !self.lexer.is_token(Token::CParenth) {
            let expr = self.parse_expr();
            if expr.is_ok() {
                let tmp = expr.unwrap();
                args.push(tmp);
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

        let function = FunctionAST::new(
            name,
            location, // location if found during
            // type checking
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        Ok(Expr::FnCall(function, args).into())
    }

    /// Parses a function call with its name and location.
    fn parse_fn_call(&mut self) -> Result<QccCell<Expr>> {
        if !self.lexer.is_token(Token::Identifier) {
            return Err(QccErrorKind::ExpectedFn)?;
        }
        let name = self.lexer.identifier();
        let location = self.lexer.location.clone();
        self.lexer.consume(Token::Identifier)?;

        self.parse_fn_call_args(name, location)
    }

    /// Returns the parsed expression.
    fn parse_expr(&mut self) -> Result<QccCell<Expr>> {
        if self.lexer.is_token(Token::Qbit) {
            let qbit = self.lexer.identifier().parse::<Qbit>()?;
            self.lexer.consume(Token::Qbit)?;
            let expr = Expr::Literal(LiteralAST::Lit_Qbit(qbit).into());
            return Ok(expr.into());
        }

        let mut unary_negative = false;
        if self.lexer.is_token(Token::Sub) {
            unary_negative = true;
            self.lexer.consume(Token::Sub)?;
        }

        if self.lexer.is_token(Token::Identifier) {
            let name = self.lexer.identifier();
            let location = self.lexer.location.clone();
            self.lexer.consume(Token::Identifier)?;

            let var: QccCell<Expr> = Expr::Var(VarAST::new_with_sign(
                name.clone(),
                location.clone(),
                unary_negative,
            ))
            .into();

            if self.lexer.is_none_token(&[
                Token::OParenth, /* function call */
                Token::Add,      /* binary expressions */
                Token::Sub,
                Token::Mul,
                Token::Div,
            ]) {
                // if none of the above tokens are seen then it is a named
                // variable
                return Ok(var);
            }

            if self.lexer.is_token(Token::OParenth) {
                // if open parenthesis is seen, then it is a function call
                self.parse_fn_call_args(name, location)
            } else if self.lexer.is_any_token(Token::all_binops()) {
                self.parse_binary_expr_with_lhs(var)
            } else {
                // NOTE: Comma will always be inside a function call
                return Err(QccErrorKind::UnexpectedExpr)?;
            }
        } else if self.lexer.is_token(Token::Digit) {
            let digit = self.lexer.digit();
            if digit.is_none() {
                return Err(QccErrorKind::UnexpectedDigit)?;
            }
            self.lexer.consume(Token::Digit)?;

            let digit = Expr::Literal(std::rc::Rc::new(std::cell::RefCell::new(
                LiteralAST::Lit_Digit(digit.unwrap()),
            )));

            if self.lexer.is_any_token(Token::all_binops()) {
                return self.parse_binary_expr_with_lhs(digit.into());
            }

            Ok(digit.into())
        } else if self.lexer.is_token(Token::OParenth) {
            // This will be a binary expression surrounded by parentheses.
            self.lexer.consume(Token::OParenth)?;

            let mut lhs: Option<QccCell<Expr>> = None;
            while !self.lexer.is_token(Token::CParenth) {
                lhs = Some(self.parse_expr()?);
            }
            self.lexer.consume(Token::CParenth)?;

            if lhs.is_some() {
                let lhs = lhs.unwrap();
                if self.lexer.is_any_token(Token::all_binops()) {
                    return self.parse_binary_expr_with_lhs(lhs);
                } else {
                    return Ok(lhs);
                }
            } else {
                return Err(QccErrorKind::ExpectedExpr)?;
            }
        } else {
            return Err(QccErrorKind::ExpectedExpr)?;
        }
    }

    /// Parses binary expression but the left-most expression is already parsed.
    fn parse_binary_expr_with_lhs(&mut self, lhs: QccCell<Expr>) -> Result<QccCell<Expr>> {
        if self.lexer.is_none_token(Token::all_binops()) {
            return Err(QccErrorKind::ExpectedOpcode)?;
        }

        let mut expr = lhs;

        while self.lexer.is_any_token(Token::all_binops()) {
            let op = self.lexer.identifier().parse::<Opcode>()?;
            self.lexer.consume(self.lexer.token.unwrap())?;

            let rhs = self.parse_expr()?;

            expr = Expr::BinaryExpr(expr, op, rhs).into();
        }

        Ok(expr.into())
    }

    /// Parse a binary expression.
    fn parse_binary_expr(&mut self) -> Result<QccCell<Expr>> {
        if self
            .lexer
            .is_none_token(&[Token::Sub, Token::Identifier, Token::Digit])
        {
            return Err(QccErrorKind::ExpectedExpr)?;
        }
        let lhs = self.parse_expr()?;
        self.parse_binary_expr_with_lhs(lhs)
    }

    fn parse_let(&mut self) -> Result<QccCell<Expr>> {
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

        Ok(Expr::Let(var, val).into())
    }

    fn parse_module(&mut self) -> Result<ModuleAST> {
        if !self.lexer.is_token(Token::Module) {
            return Err(QccErrorKind::ExpectedMod)?;
        }
        let location = self.lexer.location.clone();
        self.lexer.consume(Token::Module)?;

        let mut name: String = String::from("unnamed");

        if self.lexer.is_token(Token::Identifier) {
            name = sanitize(self.lexer.identifier());
            self.lexer.consume(Token::Identifier)?;
        }

        if !self.lexer.is_token(Token::OCurly) {
            return Err(QccErrorKind::ExpectedMod)?;
        }
        self.lexer.consume(Token::OCurly)?;

        let mut functions: Vec<QccCell<FunctionAST>> = Default::default();
        while !self.lexer.is_token(Token::CCurly) {
            let function = self.parse_function()?;
            functions.push(std::rc::Rc::new(function.into()));
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
        let module_name: Ident = module_name.trim_end_matches(".ql").into();
        let module_location = Location::new(src, 1, 1);
        // qast.add_module_info(module_name.clone(), module_location.clone());
        // representation for this module
        let mut this = ModuleAST::new(sanitize(module_name), module_location, Default::default());
        let mut imports = Vec::new();

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
                    Ok(module) => qast.append_module(module),
                    Err(e) => {
                        seen_errors = true;

                        let err: QccErrorLoc = (e, self.lexer.location.clone()).into();
                        err.report(self.lexer.line());
                    }
                }
            } else if self.lexer.is_token(Token::Hash) || self.lexer.is_token(Token::Function) {
                match self.parse_function() {
                    Ok(f) => this.append_function(f),
                    Err(e) => {
                        seen_errors = true;

                        let err: QccErrorLoc = (e, self.lexer.location.clone()).into();
                        err.report(self.lexer.line());
                    }
                }
            } else {
                if self.lexer.is_token(Token::Import) {
                    let line = self.lexer.line();
                    match self.parse_import(&qast) {
                        Ok((mod_name, fn_name)) => {
                            imports.push((mod_name, fn_name));
                        }
                        Err(err) => {
                            seen_errors = true;
                            err.report(line);
                        }
                    }
                } else {
                    self.lexer.consume(self.lexer.token.unwrap())?;
                }
            }
        }

        // collect all import statements and mangle accordingly
        for (mod_name, fn_name) in imports {
            mangle_module(&mut this, mod_name, fn_name);
        }
        qast.append_module(this);

        if seen_errors {
            Err(QccErrorKind::ParseError)?
        } else {
            Ok(qast)
        }
    }
}
