//! QAST is an abstract representation for quale language.
use crate::lexer::Location;
use crate::types::Type;

#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    /* Eof      = 0, // is replaced by None, Option<Token> is used. */
    Identifier = -1,
    Literal = -2,
}

// Design of QAST
// --------------
// We will target OpenQASM and our high-level IR for now is simply a classic
// AST.
pub struct Qast {
    functions: Vec<FunctionAST>,
}

impl Qast {
    pub(crate) fn new(functions: Vec<FunctionAST>) -> Self {
        Self { functions }
    }

    pub(crate) fn append(&mut self, function: FunctionAST) -> &mut Self {
        self.functions.push(function);
        self
    }
}

pub(crate) struct FunctionAST {
    name: Ident,
    location: Location,
    // description: String,
    input_type: Type,
    output_type: Type,
}

/// A type for representing identifiers of all kinds. It includes
/// language-specific keywords and also variable names.
pub(crate) type Ident = String;
