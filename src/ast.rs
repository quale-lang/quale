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

// Design of Qast
// --------------
// We will target OpenQASM and our high-level IR for now is simply a classic
// AST.
#[derive(Default)]
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

    pub(crate) fn append_function(
        &mut self,
        name: Ident,
        location: Location,
        attrs: Attributes,
    ) -> &mut Self {
        self.append(FunctionAST::new(name, location, attrs))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FunctionAST> + '_ {
        self.functions.iter()
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
    params: Vec<(Ident, Type)>,
    input_type: Type,
    output_type: Type,
    attrs: Attributes,
}

impl FunctionAST {
    pub(crate) fn new(name: Ident, location: Location, attrs: Attributes) -> Self {
        Self {
            name,
            location,
            params: Default::default(),
            input_type: Default::default(),
            output_type: Default::default(),
            attrs,
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
        if self.attrs.0.len() == 0 {
            write!(
                f,
                "fn {} ({}) -> {} {{  // {}\n}}",
                self.name, self.input_type, self.output_type, self.location
            )?;
        } else {
            write!(
                f,
                "fn [[{}]] {} ({}) -> {} {{  // {}\n}}",
                self.attrs, self.name, self.input_type, self.output_type, self.location
            )?;
        }
        Ok(())
    }
}

/// A type for representing identifiers of all kinds. It includes
/// language-specific keywords and also variable names.
pub(crate) type Ident = String;
