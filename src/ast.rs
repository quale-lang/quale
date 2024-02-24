//! QAST is an abstract representation for quale language.
use crate::attributes::Attributes;
use crate::lexer::Location;
use crate::types::Type;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Token {
    /* Eof      = 0, // is replaced by None, Option<Token> is used. */
    Identifier = -1,
    Literal = -2,
    Attribute = -3,
    Function = -4,
}

// // TODO
// pub(crate) enum Kind {}
// /// Some(_) => A valid `Kind`
// /// None => EOF
// pub(crate) type NewToken = Option<Kind>;

// Design of Qast
// --------------
// We will target OpenQASM and our high-level IR for now is simply a classic
// AST.
#[derive(Default)]
pub struct Qast {
    functions: Vec<FunctionAST>,
}

impl Qast {
    pub(crate) fn append(&mut self, function: FunctionAST) -> &mut Self {
        self.functions.push(function);
        self
    }

    pub(crate) fn append_function(
        &mut self,
        name: Ident,
        location: Location,
        attrs: Attributes,
    ) -> &mut Self {
        self.append(FunctionAST::new(name, location, attrs))
    }
}

impl std::fmt::Display for Qast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for function in &self.functions {
            writeln!(f, "{}", function)?;
        }
        Ok(())
    }
}

pub(crate) struct FunctionAST {
    name: Ident,
    location: Location,
    // description: String,
    input_type: Type,
    output_type: Type,
    attrs: Attributes,
}

impl FunctionAST {
    pub(crate) fn new(name: Ident, location: Location, attrs: Attributes) -> Self {
        Self {
            name,
            location,
            input_type: Default::default(),
            output_type: Default::default(),
            attrs,
        }
    }
}

impl std::fmt::Display for FunctionAST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.attrs.0.len() == 0 {
            write!(
                f,
                "fn {} ({}) -> {} {{\t// {}\n}}",
                self.name, self.input_type, self.output_type, self.location
            )?;
        } else {
            write!(
                f,
                "fn [[{}]] {} ({}) -> {} {{\t// {}\n}}",
                self.attrs, self.name, self.input_type, self.output_type, self.location
            )?;
        }
        Ok(())
    }
}

/// A type for representing identifiers of all kinds. It includes
/// language-specific keywords and also variable names.
pub(crate) type Ident = String;
