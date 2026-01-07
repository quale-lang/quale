//! QAST is an abstract representation for quale language.
use crate::attributes::Attributes;
use crate::error::{QccError, QccErrorKind, Result};
use crate::lexer::Location;
use crate::mangle::mangle_fns;
use crate::types::Type;
use std::borrow::Borrow;

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
    Qbit = -13,
    If = -14,
    Else = -15,
    Alias = -16,

    // Comparison operators
    Equal = -17,   // ==
    Unequal = -18, // !=
    LessThan = -19,
    GreaterThan = -20,

    Boolean = -21,
}

impl Token {
    pub(crate) fn all_binops() -> &'static [Self] {
        &[
            Self::Add,
            Self::Sub,
            Self::Mul,
            Self::Div,
            Self::Equal,
            Self::Unequal,
            Self::LessThan,
            Self::GreaterThan,
        ]
    }
}

// Design of Qast
// --------------
// We will target OpenQASM and our high-level IR for now is simply a classic
// AST.
#[derive(Default, Debug)]
pub struct Qast {
    modules: Vec<QccCell<ModuleAST>>,
}

impl Qast {
    #[inline]
    pub(crate) fn new(modules: Vec<QccCell<ModuleAST>>) -> Self {
        Self { modules }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.modules.len()
    }

    #[inline]
    pub(crate) fn append_module(&mut self, module: ModuleAST) {
        self.modules.push(std::rc::Rc::new(module.into()));
    }

    #[inline]
    pub(crate) fn extend(&mut self, other: &mut Qast) {
        self.modules.append(&mut other.modules);
    }

    /// Merge all modules in AST to one monolith module. Ensure mangling happens
    /// and function calls are referenced to their definitions.
    pub(crate) fn merge(&mut self) -> Qast {
        let flattened_ast = self
            .modules
            .iter()
            .flat_map(|s| {
                let mut module = s.borrow_mut();
                let module_name = module.get_name();

                let functions = module
                    .get_functions()
                    .clone()
                    .iter()
                    .map(|s| s.as_ref().borrow().get_name().clone())
                    .collect::<Vec<_>>();
                for mut function in &mut *module {
                    let function_name = function.get_name().clone();
                    function.set_name(module_name.clone() + "$" + &function_name);

                    for mut expr in &mut *function {
                        mangle_fns(expr, &module_name, &functions);
                    }
                }

                module.functions.clone()
            })
            .collect::<Vec<_>>();

        let module = ModuleAST {
            name: "Main".into(),
            location: Location::default(),
            functions: flattened_ast,
        };

        Qast::new(vec![std::rc::Rc::new(module.into())])
    }
}

impl<'a> IntoIterator for &'a Qast {
    type Item = std::cell::Ref<'a, ModuleAST>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = vec![];
        for module in &self.modules {
            iter.push(module.as_ref().borrow());
        }
        iter.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Qast {
    type Item = std::cell::RefMut<'a, ModuleAST>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = vec![];
        for module in &self.modules {
            iter.push(module.as_ref().borrow_mut());
        }
        iter.into_iter()
    }
}

impl std::fmt::Display for Qast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for module in &self.modules {
            write!(f, "|_ {}", module.as_ref().borrow())?;
        }
        Ok(())
    }
}

/// Representation of a module or namespace.
#[derive(Debug)]
pub struct ModuleAST {
    name: Ident,
    location: Location,
    functions: Vec<QccCell<FunctionAST>>,
}

impl ModuleAST {
    pub(crate) fn new(
        name: Ident,
        location: Location,
        functions: Vec<QccCell<FunctionAST>>,
    ) -> Self {
        Self {
            name,
            location,
            functions,
        }
    }

    pub(crate) fn append_function(&mut self, other: FunctionAST) -> Result<()> {
        // TODO: Use HashSet to store functions in ModuleAST.
        for function in &self.functions {
            if function.as_ref().borrow().name == other.name {
                return Err(QccErrorKind::RedefinedFunction.into());
            }
        }

        self.functions.push(std::rc::Rc::new(other.into()));
        Ok(())
    }

    #[inline]
    pub(crate) fn get_name(&self) -> Ident {
        self.name.clone()
    }

    #[inline]
    pub(crate) fn get_functions(&self) -> &Vec<QccCell<FunctionAST>> {
        &self.functions
    }
}

impl<'a> IntoIterator for &'a ModuleAST {
    type Item = std::cell::Ref<'a, FunctionAST>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = vec![];
        for function in &self.functions {
            iter.push(function.as_ref().borrow());
        }
        iter.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut ModuleAST {
    type Item = std::cell::RefMut<'a, FunctionAST>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = vec![];
        for function in &self.functions {
            iter.push(function.as_ref().borrow_mut());
        }
        iter.into_iter()
    }
}

impl std::fmt::Display for ModuleAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}\t\t\t// {}", self.name, self.location)?;
        for function in &self.functions {
            write!(f, "  |_ {}", function.as_ref().borrow())?;
        }
        Ok(())
    }
}

pub(crate) struct ImportAST(Vec<(ModuleAST, Vec<QccCell<FunctionAST>>)>);

impl ImportAST {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    // FIXME: What should be the param type?
    pub(crate) fn push(&mut self, module: &ModuleAST, function: &QccCell<FunctionAST>) {
        for (m, fns) in &mut self.0 {
            if m.get_name() == module.get_name() {
                fns.push(function.clone()); // FIXME: no clone pls
            }
        }
    }
}

impl std::fmt::Display for ImportAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (module, functions) in &self.0 {
            write!(f, "import {}::{{", module.get_name())?;
            let functions_str = functions
                .iter()
                .map(|p| p.as_ref().borrow().to_string())
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "{functions_str}")?;
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

/// A repr for a variable. It contains a `name` of the variable and its
/// `location`.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct VarAST {
    name: Ident,
    location: Location,
    type_: Type,
    unary_negative: bool, // represent unary negative named variables
}

impl VarAST {
    pub(crate) fn new(name: Ident, location: Location) -> Self {
        Self {
            name,
            location,
            type_: Default::default(),
            unary_negative: false,
        }
    }

    pub(crate) fn new_with_type(name: Ident, location: Location, type_: Type) -> Self {
        Self {
            name,
            location,
            type_,
            unary_negative: false,
        }
    }

    pub(crate) fn new_with_sign(name: Ident, location: Location, unary_negative: bool) -> Self {
        Self {
            name,
            location,
            type_: Default::default(),
            unary_negative,
        }
    }

    pub(crate) fn new_with_type_and_sign(
        name: Ident,
        location: Location,
        type_: Type,
        unary_negative: bool,
    ) -> Self {
        Self {
            name,
            location,
            type_,
            unary_negative,
        }
    }

    pub(crate) fn set_type(&mut self, type_: Type) {
        self.type_ = type_.into();
    }

    #[inline]
    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    #[inline]
    pub(crate) fn location(&self) -> &Location {
        &self.location
    }

    #[inline]
    pub(crate) fn is_typed(&self) -> bool {
        *self.type_.borrow() != Type::Bottom
    }

    /// Get the type of variable.
    ///
    /// # NOTE: It does not check for untyped variables.
    #[inline]
    pub(crate) fn get_type(&self) -> Type {
        self.type_
    }
}

impl std::fmt::Display for VarAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self.type_.borrow() {
            Type::Bottom => {
                if self.unary_negative {
                    write!(f, "-{}", self.name)
                } else {
                    write!(f, "{}", self.name)
                }
            }
            _ => {
                if self.unary_negative {
                    write!(f, "-{}: {}", self.name, self.type_.borrow())
                } else {
                    write!(f, "{}: {}", self.name, self.type_.borrow())
                }
            }
        }
    }
}

impl From<VarAST> for QccCell<Expr> {
    fn from(var: VarAST) -> Self {
        Expr::Var(var).into()
    }
}

/// Mathematical operators.
#[derive(Debug)]
pub(crate) enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    LT,
    GT,
    LTE,
    GTE,
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
            "<" => Ok(Self::LT),
            ">" => Ok(Self::GT),
            "<=" => Ok(Self::LTE),
            ">=" => Ok(Self::GTE),
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
            Self::LT => write!(f, "<")?,
            Self::GT => write!(f, ">")?,
            Self::LTE => write!(f, "<=")?,
            Self::GTE => write!(f, ">=")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Qbit {
    amp_0: f64,
    amp_1: f64,
}

impl std::fmt::Display for Qbit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0q{}_{}", self.amp_0, self.amp_1)
    }
}

impl std::str::FromStr for Qbit {
    type Err = QccErrorKind;

    /// A quantum numeral should be of the form `0q(<amplitude>, amplitude)`
    /// where the pair of amplitudes are probability amplitudes for zero and one
    /// basis vectors respectively.
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        if !s.starts_with("0q") {
            Err(QccErrorKind::ExpectedQbit)?
        }

        let s = s.trim_start_matches("0q");

        if !s.starts_with('(') || !s.ends_with(')') {
            Err(QccErrorKind::ExpectedParenth)?
        }

        let (s1, s2) = s
            .trim_matches(&['(', ')'])
            .split_once(',')
            .ok_or(QccErrorKind::ExpectedComma)?;

        let amp_0 = s1.trim().parse::<f64>();
        if amp_0.is_err() {
            Err(QccErrorKind::ExpectedAmpinQbit)?
        }
        let amp_0 = amp_0.unwrap();
        let amp_1 = s2.trim().parse::<f64>();
        if amp_1.is_err() {
            Err(QccErrorKind::ExpectedAmpinQbit)?
        }
        let amp_1 = amp_1.unwrap();

        Ok(Self { amp_0, amp_1 })
    }
}

#[derive(Debug)]
pub(crate) enum LiteralAST {
    Lit_Qbit(Qbit),
    Lit_Digit(f64),
    Lit_Boolean(bool),
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
        } else if s.starts_with("0q") {
            // quantum numeral
            let qn = s.parse::<Qbit>()?;
            Ok(Self::Lit_Qbit(qn))
        } else if s == "true" || s == "false" {
            return Ok(Self::Lit_Boolean(s == "true"));
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
            LiteralAST::Lit_Boolean(b) => write!(f, "{b}"),
            LiteralAST::Lit_Str(s) => {
                write!(f, "\"")?;
                for &c in s {
                    write!(f, "{}", c as char)?;
                }
                write!(f, "\"")
            }
            LiteralAST::Lit_Qbit(qn) => write!(f, "{}", qn),
        }
    }
}

impl From<LiteralAST> for QccCell<LiteralAST> {
    fn from(lit: LiteralAST) -> Self {
        QccCell::new(lit.into())
    }
}

pub(crate) enum BinaryExprAST {
    Var(VarAST),
    Literal(Box<LiteralAST>),
    BinaryExpr(Box<BinaryExprAST>, Opcode, Box<BinaryExprAST>),
    FnCall(FunctionAST, Vec<Box<BinaryExprAST>>),
}

impl std::fmt::Display for BinaryExprAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{v}"),
            Self::Literal(lit) => write!(f, "{lit}"),
            Self::BinaryExpr(lhs, op, rhs) => write!(f, "({lhs} {op} {rhs})"),
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
        }
    }
}

pub(crate) type QccCell<T> = std::rc::Rc<std::cell::RefCell<T>>;

#[derive(Debug)]
pub(crate) struct AliasAST(pub(crate) VarAST, pub(crate) VarAST);

impl AliasAST {
    pub(crate) fn new(alias: VarAST, function: VarAST) -> Self {
        Self(alias, function)
    }
}

#[derive(Debug)]
pub enum Expr {
    Var(VarAST),
    BinaryExpr(QccCell<Expr>, Opcode, QccCell<Expr>),
    Tensor(Vec<QccCell<Expr>>),
    FnCall(FunctionAST, Vec<QccCell<Expr>>),
    Let(VarAST, QccCell<Expr>),
    Assign(VarAST, QccCell<Expr>),
    Conditional(
        QccCell<Expr>,
        Vec<QccCell<Expr>>, /* truth_block */
        Vec<QccCell<Expr>>, /* false_block */
    ),
    Literal(QccCell<LiteralAST>),
}

impl Expr {
    pub(crate) fn get_location(&self) -> Location {
        match &self {
            Self::Var(v) => v.location().clone(),
            Self::BinaryExpr(lhs, _, _) => lhs.as_ref().borrow().get_location(),
            Self::Tensor(_) => Default::default(), // TODO: This will require subtracting the dimension of tensor
            Self::FnCall(f, _) => f.get_loc().clone(),
            Self::Let(var, _) => var.location.clone(),
            Self::Assign(v, _) => v.location().clone(),
            Self::Conditional(c, _, _) => c.as_ref().borrow().get_location(),
            Self::Literal(lit) =>
            /*TODO*/
            {
                Default::default()
            }
        }
    }

    pub(crate) fn get_type(&self) -> Type {
        match &self {
            Self::Var(v) => v.get_type(),
            Self::BinaryExpr(lhs, op, rhs) => {
                let lhs_type = lhs.as_ref().borrow().get_type();
                let rhs_type = rhs.as_ref().borrow().get_type();

                if lhs_type == rhs_type {
                    return lhs_type;
                } else {
                    if (lhs_type == Type::Qbit && rhs_type == Type::F64)
                        || (lhs_type == Type::F64 && rhs_type == Type::Qbit)
                    {
                        return Type::Qbit;
                    }
                    // TODO: more type rules.
                    return Type::Bottom;
                }
            }
            Self::Tensor(v) => {
                if v.len() == 0 {
                    Type::Bottom
                } else {
                    v[0].as_ref().borrow().get_type()
                }
            }
            Self::FnCall(f, args) => *f.get_output_type(),
            Self::Let(var, val) => var.get_type(),
            Self::Assign(var, val) => var.get_type(),
            Self::Conditional(conditional, truth_block, false_block) => {
                let last_expr = truth_block.last();
                if last_expr.is_none() {
                    return Type::Bottom;
                }

                return last_expr.unwrap().as_ref().borrow().get_type();
            }
            Self::Literal(lit) => match *lit.as_ref().borrow() {
                LiteralAST::Lit_Str(_) => Type::Bottom,
                LiteralAST::Lit_Digit(_) => Type::F64,
                LiteralAST::Lit_Boolean(_) => Type::Bool,
                LiteralAST::Lit_Qbit(_) => Type::Qbit,
            },
        }
    }
}

impl From<Expr> for QccCell<Expr> {
    fn from(expr: Expr) -> Self {
        std::rc::Rc::new(std::cell::RefCell::new(expr))
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{}", v),
            Self::BinaryExpr(lhs, op, rhs) => {
                write!(
                    f,
                    "({} {} {})",
                    *lhs.as_ref().borrow(),
                    op,
                    *rhs.as_ref().borrow()
                )
            }

            Self::Tensor(tensor) => {
                write!(f, "[");
                let tensor_str = tensor
                    .iter()
                    .map(|p| p.as_ref().borrow().to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{tensor_str}")?;
                write!(f, "]")
            }

            Self::FnCall(function, args) => {
                if *function.get_output_type() != Type::Bottom {
                    write!(f, "{}: {} (", function.name, function.output_type)?;
                } else {
                    write!(f, "{}(", function.name)?;
                }
                let args_str = args
                    .iter()
                    .map(|p| p.as_ref().borrow().to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{args_str}")?;
                write!(f, ")")?;
                Ok(())
            }

            Self::Let(var, val) => write!(f, "{} = {}", var, *val.as_ref().borrow()),

            Self::Assign(var, val) => write!(f, "{} = {}", var, *val.as_ref().borrow()),
            Self::Conditional(cond, _truth, _false) => {
                writeln!(f, "{}", *cond.as_ref().borrow())?;

                if !_truth.is_empty() {
                    writeln!(f, "      |_ True")?;
                    let truth_block = _truth
                        .iter()
                        .map(|p| p.as_ref().borrow().to_string())
                        .collect::<Vec<String>>()
                        .join("\n        |_ ");
                    writeln!(f, "        |_ {truth_block}")?;
                }

                if !_false.is_empty() {
                    writeln!(f, "      |_ False")?;
                    let false_block = _false
                        .iter()
                        .map(|p| p.as_ref().borrow().to_string())
                        .collect::<Vec<String>>()
                        .join("\n        |_ ");
                    writeln!(f, "        |_ {false_block}")?;
                }

                Ok(())
            }

            Self::Literal(lit) => write!(f, "{}", *lit.as_ref().borrow()),
        }
    }
}

// TODO: Remove Clone trait when FnCall supports reference to functions.
#[derive(Debug, Clone)]
pub struct FunctionAST {
    name: Ident,
    location: Location,
    // description: String,
    params: Vec<VarAST>,
    input_type: Vec<Type>,
    output_type: Type,
    attrs: Attributes,
    body: Vec<QccCell<Expr>>,
}

// impl Expr for FunctionAST {}

impl FunctionAST {
    pub(crate) fn new(
        name: Ident,
        location: Location,
        params: Vec<VarAST>,
        input_type: Vec<Type>,
        output_type: Type,
        attrs: Attributes,
        body: Vec<QccCell<Expr>>,
    ) -> Self {
        Self {
            name,
            location,
            params,
            input_type,
            output_type,
            attrs,
            body,
        }
    }

    /// Inserts the input type in function. This should be called successively
    /// for many-parametered functions to append types for each parameter into a
    /// vector.
    #[inline]
    pub(crate) fn insert_input_type(&mut self, type_: Type) {
        self.input_type.push(type_);
    }

    #[inline]
    pub(crate) fn set_output_type(&mut self, type_: Type) {
        self.output_type = type_;
    }

    #[inline]
    pub(crate) fn set_name(&mut self, name: Ident) {
        self.name = name;
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
    pub(crate) fn get_input_type(&self) -> &Vec<Type> {
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

    #[inline]
    pub(crate) fn last(&self) -> Option<&QccCell<Expr>> {
        self.body.last()
    }

    #[inline]
    pub(crate) fn last_mut(&mut self) -> Option<&mut QccCell<Expr>> {
        self.body.last_mut()
    }

    // /// If a return expression exists in function, return its reference.
    // // TODO:
    // pub(crate) fn get_return_expr(&self) -> Option<&Expr> {
    //     let last_instruction = self.body.last()?;
    //     match **last_instruction {
    //         Expr::Var(_) | Expr::Let(_, _) => None,
    //         x => Some(&x),
    //     }
    // }

    #[inline]
    pub(crate) fn iter_params(&self) -> impl Iterator<Item = &VarAST> + '_ {
        self.params.iter()
    }

    #[inline]
    pub(crate) fn iter_params_mut(&mut self) -> impl Iterator<Item = &mut VarAST> + '_ {
        self.params.iter_mut()
    }
}

impl<'a> IntoIterator for &'a FunctionAST {
    type Item = &'a QccCell<Expr>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = vec![];
        for expr in &self.body {
            iter.push(expr);
        }
        iter.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut FunctionAST {
    // type Item = std::cell::RefMut<'a, Expr>;
    type Item = &'a mut QccCell<Expr>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = vec![];
        for expr in &mut self.body {
            iter.push(expr);
        }
        iter.into_iter()
    }
}

impl std::fmt::Display for FunctionAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn ")?;
        if self.attrs.0.len() != 0 {
            write!(f, "[[{}]] ", self.attrs)?;
        }
        // parameters
        let params = self
            .params
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        writeln!(
            f,
            "{} ({}) : {}\t\t// {}",
            self.name, params, self.output_type, self.location
        )?;

        for expr in &self.body {
            writeln!(f, "    |_ {}", *expr.as_ref().borrow())?;
        }
        writeln!(f, "")?;

        Ok(())
    }
}

/// A type for representing identifiers of all kinds. It includes
/// language-specific keywords and also variable names.
pub(crate) type Ident = String;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn check_var_ast() {
        let x = VarAST::new(String::from("x"), Location::default());

        assert!(!x.is_typed());
        assert!(*x.name() == String::from("x"));
        assert!(*x.location() == Location::new("unknown", 0, 0));

        let y = VarAST::new_with_sign(String::from("y"), Location::default(), true);

        assert!(!y.is_typed());
        assert!(*y.name() == String::from("y"));
        assert_eq!(format!("{y}"), "-y");

        let z = VarAST::new_with_type(String::from("z"), Location::default(), Type::F64);

        assert!(z.is_typed());
        assert!(*z.name() == String::from("z"));
        assert_eq!(format!("{z}"), "z: float64");

        let w =
            VarAST::new_with_type_and_sign(String::from("w"), Location::default(), Type::F64, true);

        assert!(w.is_typed());
        assert!(*w.name() == String::from("w"));
        assert_eq!(format!("{}", w), "-w: float64");
    }

    #[test]
    fn check_function_ast() {
        let x = VarAST::new(String::from("x"), Default::default());
        let y = VarAST::new(String::from("y"), Default::default());
        let z = VarAST::new(String::from("z"), Default::default());
        let w = VarAST::new(String::from("w"), Default::default());

        // Example:
        //  fn foo(x, y) {
        //    let z = x * y;
        //    let w = z + x;
        //    return w;
        //  }
        let let_z = Expr::Let(
            z.clone(),
            Expr::BinaryExpr(x.clone().into(), Opcode::Mul, y.clone().into()).into(),
        );
        let let_w = Expr::Let(
            w.clone(),
            Expr::BinaryExpr(z.into(), Opcode::Add, x.clone().into()).into(),
        );
        let ret_w = Expr::Var(w);

        let foo = FunctionAST::new(
            String::from("foo"),
            Default::default(),
            vec![x, y],
            vec![],
            Type::Bottom,
            Attributes::default(),
            vec![let_z.into(), let_w.into(), ret_w.into()],
        );
        assert!(foo.last().is_some());
    }

    #[test]
    fn check_qast() {
        let x = VarAST::new(String::from("x"), Default::default());
        let y = VarAST::new(String::from("y"), Default::default());
        let z = VarAST::new(String::from("z"), Default::default());
        let w = VarAST::new(String::from("w"), Default::default());

        let let_z = Expr::Let(
            z.clone(),
            Expr::BinaryExpr(x.clone().into(), Opcode::Mul, y.clone().into()).into(),
        );
        let let_w = Expr::Let(
            w.clone(),
            Expr::BinaryExpr(z.into(), Opcode::Add, x.clone().into()).into(),
        );
        let ret_w = Expr::Var(w);

        let foo = FunctionAST::new(
            String::from("foo"),
            Default::default(),
            vec![x, y],
            vec![],
            Type::Bottom,
            Attributes::default(),
            vec![let_z.into(), let_w.into(), ret_w.into()],
        );

        let module = ModuleAST::new(
            String::from("Main"),
            Default::default(),
            vec![Rc::new(foo.into())],
        );

        let qast = Qast::new(vec![Rc::new(module.into())]);
        println!("{}", qast);

        assert_eq!(
            format!("{qast}"),
            "|_ Main\t\t\t// @unknown:0:0
  |_ fn foo (x, y) : Nothing\t\t// @unknown:0:0
    |_ z = (x * y)
    |_ w = (z + x)
    |_ w

"
        );
    }

    #[test]
    fn check_qbit() {
        let s0 = "0q(0.5, 0.5)";
        let qbit = s0.parse::<Qbit>();
        assert!(qbit.is_ok());
        assert_eq!(format!("{}", qbit.unwrap()), "0q0.5_0.5");

        let s1 = "0(1, 0)";
        let qbit1 = s1.parse::<Qbit>();
        assert!(qbit1.is_err());
        assert_eq!(qbit1.err().unwrap(), QccErrorKind::ExpectedQbit);

        let s2 = "0q0.5, 0.5";
        let qbit2 = s2.parse::<Qbit>();
        assert!(qbit2.is_err());
        assert_eq!(qbit2.err().unwrap(), QccErrorKind::ExpectedParenth);

        let s3 = "0q(1, a)";
        let qbit3 = s3.parse::<Qbit>();
        assert!(qbit3.is_err());
        assert_eq!(qbit3.err().unwrap(), QccErrorKind::ExpectedAmpinQbit);

        let s4 = "0q(0.75 0.25)";
        let err_qbit = s4.parse::<Qbit>();
        assert!(err_qbit.is_err());
        assert_eq!(err_qbit.err().unwrap(), QccErrorKind::ExpectedComma);
    }
}
