//! This file contains all error types for qcc.
use std::error::Error;
use std::fmt::{Debug, Display};

pub(crate) type Result<T> = std::result::Result<T, QccError>;

#[derive(Debug, PartialEq)]
pub(crate) enum QccErrorKind {
    InvalidArgs,
    NoSuchArg,
    NoFile,
    UnexpectedAttr,
    ExpectedAttr,
}

impl QccErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        use QccErrorKind::*;

        match *self {
            InvalidArgs => "invalid number of arguments",
            NoSuchArg => "no such argument",
            NoFile => "no such file",
            UnexpectedAttr => "unexpected attribute",
            ExpectedAttr => "expected #[attribute]",
        }
    }
}

impl Display for QccErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, PartialEq)]
pub struct QccError(pub(crate) QccErrorKind);

impl Display for QccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "qcc: {}", self.0)
    }
}

impl From<QccErrorKind> for QccError {
    fn from(value: QccErrorKind) -> Self {
        QccError(value)
    }
}

impl Error for QccError {}

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
