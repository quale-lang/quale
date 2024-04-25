//! QAST is an abstract representation for quale language.
use crate::attributes::Attributes;
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
        body: Vec<Box<dyn Expr>>,
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

/// A trait to represent expression. An expression can be of various types.
/// Binary expressions, let assignments, function calls and returning
/// expressions should implement it.
pub(crate) trait Expr: std::fmt::Display {}

/// A repr for a variable. It contains a `name` of the variable and its
/// `location`.
pub(crate) struct VarAST {
    name: Ident,
    location: Location,
    type_: std::cell::RefCell<Type>,
}

impl VarAST {
    pub(crate) fn new(name: Ident, location: Location) -> Self {
        Self { name, location, type_: Default::default() }
    }

    pub(crate) fn set_type(&mut self, type_: Type) {
        self.type_ = type_.into();
    }
}

impl Expr for VarAST {}

impl std::fmt::Display for VarAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.type_.borrow())
    }
}

/// Repr for `let` statements. Location of a let expression is the location of
/// its `var` member.
pub(crate) struct LetAST<T: Expr> {
    var: VarAST,
    val: std::cell::RefCell<T>,
}

impl<T> LetAST<T>
where
    T: Expr,
{
    pub(crate) fn new(var: VarAST, val: T) -> Self {
        Self { var, val: val.into() }
    }
}

impl<T: Expr> Expr for LetAST<T> {}

impl<T> std::fmt::Display for LetAST<T>
where
    T: Expr + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} = {}", self.var, self.val.borrow())
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

/// Repr for binary expressions.
pub(crate) struct BinaryExprAST<T1: Expr, T2: Expr> {
    lhs: T1,
    rhs: T2,
    op: Opcode,
}

impl<T1, T2> Expr for BinaryExprAST<T1, T2>
where
    T1: Expr,
    T2: Expr,
{
}

impl<T1, T2> BinaryExprAST<T1, T2>
where
    T1: Expr,
    T2: Expr,
{
    pub(crate) fn new(lhs: T1, op: Opcode, rhs: T2) -> Self {
        Self { lhs, rhs, op }
    }
}

impl<T1, T2> std::fmt::Display for BinaryExprAST<T1, T2>
where
    T1: Expr + std::fmt::Display,
    T2: Expr + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "({} {} {})", self.lhs, self.op, self.rhs)
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
    body: Vec<Box<dyn Expr>>,
}

impl Expr for FunctionAST {}

impl FunctionAST {
    pub(crate) fn new(
        name: Ident,
        location: Location,
        params: Vec<(Ident, Type)>,
        output_type: Type,
        attrs: Attributes,
        body: Vec<Box<dyn Expr>>,
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
            write!(f, "[[{}]]", self.attrs)?;
        }
        writeln!(
            f,
            " {} ({}) : {} {{  // {}",
            self.name, self.input_type, self.output_type, self.location
        )?;

        for expr in &self.body {
            write!(f, "    {}", *expr)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

/// A type for representing identifiers of all kinds. It includes
/// language-specific keywords and also variable names.
pub(crate) type Ident = String;
