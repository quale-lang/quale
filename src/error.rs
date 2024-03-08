//! This file contains all error types for qcc.
//!
//! We have three types to represent errors in qcc.
//!
//! 1. `QccErrorKind`:  This is the bare-bones of error which only stores kind
//!    of the error.
//! 2. `QccErrorLoc`: It tags a `Location` along with the error kind, providing
//!    a complete picture of where the error was originated. This is an internal
//!    error representation, or commonly known as "bug reporter".
//! 3. `QccError`: This is the external error which is returned to any driver or
//!    caller. It has various From<> traits deriving from both kinds and
//!    location errors, often dropping the location and only carrying kind.
use crate::lexer::Location;
use std::error::Error;
use std::fmt::{Debug, Display};

pub(crate) type Result<T> = std::result::Result<T, QccError>;

// We require RefCell to gain interior mutability. There are cases like dealing
// with a substring in buffer, we can only infer partial information about its
// location. Consider the example of attribute parsing, where we can only know
// column index. Using RefCell allows us to mutate/append this information with
// richer information down the call stack.
pub(crate) type LocationRef = std::cell::RefCell<Location>;

#[derive(Debug, PartialEq)]
pub enum QccErrorKind {
    InvalidArgs,
    NoSuchArg,
    NoFile,
    UnexpectedAttr,
    ParseError,
    ExpectedFnForAttr,
}

impl Display for QccErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str((|kind: &Self| {
            use QccErrorKind::*;
            match kind {
                InvalidArgs => "invalid number of arguments",
                NoSuchArg => "no such argument",
                NoFile => "no such file",
                UnexpectedAttr => "unexpected attribute",
                ParseError => "could not parse source",
                ExpectedFnForAttr => "expected a function for attribute",
            }
        })(self))
    }
}

/// This is the main error which any stage processing returns. For example, the
/// parser returns it. We are only concerned with kind of an error.
#[derive(Debug, PartialEq)]
pub struct QccError(pub(crate) QccErrorKind);

impl Display for QccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "qcc: {}", self.0)
    }
}

impl Error for QccError {}

impl From<QccErrorKind> for QccError {
    fn from(value: QccErrorKind) -> Self {
        QccError(value)
    }
}

impl From<QccErrorLoc> for QccError {
    fn from(err: QccErrorLoc) -> Self {
        err.0
    }
}

impl From<std::io::Error> for QccError {
    fn from(_: std::io::Error) -> Self {
        Self(QccErrorKind::NoFile)
    }
}

impl From<String> for QccError {
    fn from(_: String) -> Self {
        Self(QccErrorKind::NoFile)
    }
}

impl From<&str> for QccError {
    fn from(_: &str) -> Self {
        Self(QccErrorKind::NoFile)
    }
}

/// This is an internal error representation, most commonly known as "bug
/// reporting". This doesn't show up at the end of stage processing, like
/// parsing. It is only used by the parser, say, for reporting errors in the
/// compilation unit.
#[derive(Debug, PartialEq)]
pub struct QccErrorLoc(QccError, LocationRef);

impl QccErrorLoc {
    pub(crate) fn new(kind: QccErrorKind, loc: Location) -> Self {
        Self(QccError(kind), LocationRef::new(loc.into()))
    }

    pub(crate) fn get_error(&self) -> &QccError {
        &self.0
    }

    pub(crate) fn get_loc(&self) -> LocationRef {
        self.1.clone()
    }

    pub(crate) fn set_error(&mut self, err: QccError) {
        self.0 = err;
    }

    pub(crate) fn set_loc(&mut self, loc: Location) {
        self.1 = LocationRef::new(loc.into());
    }

    // TODO
    pub(crate) fn set_path(&mut self, path: &str) {}

    /// Takes a mutable reference `QccErrorLoc` and replaces its row in
    /// location.
    pub(crate) fn set_row(&mut self, row: usize) {
        self.1.replace(Location::new(
            &self.1.borrow_mut().path(),
            row,
            self.1.borrow_mut().col(),
        ));
    }

    pub(crate) fn set_col(&mut self, col: usize) {
        let loc = self.1.take();
        let new_loc = Location::new(loc.path().as_str(), loc.row(), col);
        self.1.replace(new_loc);
    }
}

impl Display for QccErrorLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Is RefCell access correct?
        write!(f, "{} {}", self.0, self.1.borrow())
    }
}

impl From<QccErrorKind> for QccErrorLoc {
    fn from(kind: QccErrorKind) -> Self {
        Self(QccError(kind), LocationRef::new(Default::default()))
    }
}

impl From<QccError> for QccErrorLoc {
    fn from(err: QccError) -> Self {
        Self(err, LocationRef::new(Default::default()))
    }
}

impl From<(QccError, LocationRef)> for QccErrorLoc {
    fn from(err: (QccError, LocationRef)) -> Self {
        Self(err.0, err.1)
    }
}

impl From<(QccErrorKind, Location)> for QccErrorLoc {
    fn from(err: (QccErrorKind, Location)) -> Self {
        Self(err.0.into(), LocationRef::new(err.1.into()))
    }
}

impl From<(QccError, Location)> for QccErrorLoc {
    fn from(err: (QccError, Location)) -> Self {
        Self(err.0, LocationRef::new(err.1.into()))
    }
}

impl From<(QccErrorLoc, Location)> for QccErrorLoc {
    fn from(err: (QccErrorLoc, Location)) -> Self {
        err.0 .1.replace(err.1);
        err.0
    }
}

impl Error for QccErrorLoc {}

impl From<std::io::Error> for QccErrorLoc {
    fn from(_: std::io::Error) -> Self {
        Self(
            QccError(QccErrorKind::NoFile),
            LocationRef::new(Default::default()),
        )
    }
}

impl From<String> for QccErrorLoc {
    fn from(_: String) -> Self {
        Self(
            QccError(QccErrorKind::NoFile),
            LocationRef::new(Default::default()),
        )
    }
}

impl From<&str> for QccErrorLoc {
    fn from(_: &str) -> Self {
        Self(
            QccError(QccErrorKind::NoFile),
            LocationRef::new(Default::default()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_errors() -> Result<()> {
        use QccErrorKind::*;

        let e1: Result<()> = Err(QccError(UnexpectedAttr));
        match e1 {
            Ok(_) => unreachable!(),
            Err(ref e) => assert_eq!(e.to_string(), "qcc: unexpected attribute"),
        }

        let e2: Result<()> = Err(QccError(NoFile));
        match e2 {
            Ok(_) => unreachable!(),
            Err(ref e) => assert_eq!(e.to_string(), "qcc: no such file"),
        }
        Ok(())
    }
}
