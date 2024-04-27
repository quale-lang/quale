//! QAST is an abstract representation for quale language.
use crate::attributes::Attributes;
use crate::error::{QccError, QccErrorKind};
use crate::lexer::Location;
use crate::types::Type;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Token {
    Hash = '#' as isize,
    OBracket = '[' as isize,
    CBracket = ']' as isize,
    OParenth = '(' as isize,
    CParenth = ')' as isize,
    OCurly = '{' as isize,
    CCurly = '}' as isize,
    Comma = ',' as isize,
    Colon = ':' as isize,
    Semicolon = ';' as isize,
    Bang = '!' as isize,
    Assign = '=' as isize,

    Add = '+' as isize,
    Sub = '-' as isize,
    Mul = '*' as isize,
    Div = '/' as isize,

    /* Eof is replaced by None, Option<Token> is used. */
    Identifier = -1,
    Literal = -2,
    Attribute = -3,
    Function = -4,
    Multi = -5, // token comprises of more than one chars
    Digit = -6,
    Return = -7,
    Const = -8,
    Extern = -9,
    Module = -10,
    Import = -11,
    Let = -12,
}

// Design of Qast
// --------------
// We will target OpenQASM and our high-level IR for now is simply a classic
// AST.
#[derive(Default)]
pub struct Qast {
    name: Ident,
    location: Location,
    functions: Vec<FunctionAST>,
}

impl Qast {
    pub(crate) fn new(name: Ident, location: Location, functions: Vec<FunctionAST>) -> Self {
        Self {
            name,
            location,
            functions,
        }
    }

    /// Add module level information in the ast. This includes module name and
    /// its location. Location by default should be first row and first column.
    pub(crate) fn add_module_info(&mut self, name: Ident, location: Location) {
        self.name = name;
        self.location = location;
    }

    pub(crate) fn append(&mut self, function: FunctionAST) {
        self.functions.push(function);
    }

    pub(crate) fn append_function(
        &mut self,
        name: Ident,
        location: Location,
        params: Vec<(Ident, Type)>,
        output_type: Type,
        attrs: Attributes,
        body: Vec<Box<Expr>>,
    ) {
        self.append(FunctionAST::new(
            name,
            location,
            params,
            output_type,
            attrs,
            body,
        ));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FunctionAST> + '_ {
        self.functions.iter()
    }
}

impl std::fmt::Display for Qast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "module {} {{  // {}", self.name, self.location)?;
        for function in &self.functions {
            writeln!(f, "{}", function)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// Representation of a module or namespace.
pub(crate) struct ModuleAST {
    name: Ident,
    location: Location,
    functions: Vec<FunctionAST>,
}

impl ModuleAST {
    pub(crate) fn new(name: Ident, location: Location, functions: Vec<FunctionAST>) -> Self {
        Self {
            name,
            location,
            functions,
        }
    }
}

impl std::fmt::Display for ModuleAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "module {} {{  // {}", self.name, self.location)?;
        for function in &self.functions {
            // TODO: Add tab before each function line for pretty printing.
            writeln!(f, "{}", function)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// A repr for a variable. It contains a `name` of the variable and its
/// `location`.
pub(crate) struct VarAST {
    name: Ident,
    location: Location,
    type_: std::cell::RefCell<Type>,
}

impl VarAST {
    pub(crate) fn new(name: Ident, location: Location) -> Self {
        Self {
            name,
            location,
            type_: Default::default(),
        }
    }

    pub(crate) fn set_type(&mut self, type_: Type) {
        self.type_ = type_.into();
    }
}

impl std::fmt::Display for VarAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self.type_.borrow() == Type::Unknown {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.type_.borrow())
        }
    }
}

/// Mathematical operators.
pub(crate) enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
}

impl std::str::FromStr for Opcode {
    type Err = QccError;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            "==" => Ok(Self::Eq),
            "!=" => Ok(Self::Neq),
            _ => Err(QccErrorKind::UnknownOpcode.into()),
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+")?,
            Self::Sub => write!(f, "-")?,
            Self::Mul => write!(f, "*")?,
            Self::Div => write!(f, "/")?,
            Self::Eq => write!(f, "==")?,
            Self::Neq => write!(f, "!=")?,
        }
        Ok(())
    }
}

pub(crate) enum LiteralAST {
    Lit_Digit(f64),
    Lit_Str(Vec<u8>), // does not store the quotations around str
}

impl std::str::FromStr for LiteralAST {
    type Err = QccErrorKind;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        if s.starts_with("\"") {
            if s.ends_with("\"") {
                return Err(QccErrorKind::UnexpectedStr)?;
            }

            let mut v = vec![];
            while let Some(c) = s.chars().next() {
                if c != '\"' {
                    v.push(c as u8);
                }
            }
            return Ok(Self::Lit_Str((v)));
        } else {
            // parse digit
            let d = s.parse::<f64>();
            if d.is_err() {
                return Err(QccErrorKind::UnexpectedDigit)?;
            }
            return Ok(Self::Lit_Digit((d.unwrap())));
        }
    }
}

impl std::fmt::Display for LiteralAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LiteralAST::Lit_Digit(d) => write!(f, "{}", d),
            LiteralAST::Lit_Str(s) => {
                write!(f, "\"")?;
                for &c in s {
                    write!(f, "{}", c as char)?;
                }
                write!(f, "\"")
            }
        }
    }
}

pub(crate) enum Expr {
    // TODO: Literals should be included here too.
    Var(VarAST),
    BinaryExpr(Box<Expr>, Opcode, Box<Expr>),
    FnCall(FunctionAST, Vec<Box<Expr>>),
    Let(VarAST, Box<Expr>),
    Literal(Box<LiteralAST>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{}", v),
            Self::BinaryExpr(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
            Self::FnCall(function, args) => {
                write!(f, "{}(", function.name)?;
                let args_str = args
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{args_str}")?;
                write!(f, ")")?;
                Ok(())
            }
            Self::Let(var, val) => writeln!(f, "{} = {}", var, val),
            Self::Literal(lit) => write!(f, "{}", lit),
        }
    }
}

pub(crate) struct FunctionAST {
    name: Ident,
    location: Location,
    // description: String,
    params: Vec<(Ident, Type)>,
    input_type: Type,
    output_type: Type,
    attrs: Attributes,
    body: Vec<Box<Expr>>,
}

// impl Expr for FunctionAST {}

impl FunctionAST {
    pub(crate) fn new(
        name: Ident,
        location: Location,
        params: Vec<(Ident, Type)>,
        output_type: Type,
        attrs: Attributes,
        body: Vec<Box<Expr>>,
    ) -> Self {
        Self {
            name,
            location,
            params,
            input_type: Default::default(),
            output_type,
            attrs,
            body,
        }
    }

    #[inline]
    pub(crate) fn get_name(&self) -> &Ident {
        &self.name
    }

    #[inline]
    pub(crate) fn get_loc(&self) -> &Location {
        &self.location
    }

    #[inline]
    pub(crate) fn get_input_type(&self) -> &Type {
        &self.input_type
    }

    #[inline]
    pub(crate) fn get_output_type(&self) -> &Type {
        &self.output_type
    }

    #[inline]
    pub(crate) fn get_attrs(&self) -> &Attributes {
        &self.attrs
    }
}

impl std::fmt::Display for FunctionAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn ")?;
        if self.attrs.0.len() != 0 {
            write!(f, "[[{}]] ", self.attrs)?;
        }
        writeln!(
            f,
            "{} ({}) : {} {{  // {}",
            self.name, self.input_type, self.output_type, self.location
        )?;

        for expr in &self.body {
            write!(f, "    {}", *expr)?;
        }
        writeln!(f, "\n}}")?;

        Ok(())
    }
}

/// A type for representing identifiers of all kinds. It includes
/// language-specific keywords and also variable names.
pub(crate) type Ident = String;
