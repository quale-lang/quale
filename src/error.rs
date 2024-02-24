//! This file contains all error types for qcc.
use std::error::Error;
use std::fmt::{Debug, Display};

// TODO: Box<> will be replaced by QccError,
pub(crate) type Result<T> = std::result::Result<T, QccError>;

// pub(crate) trait Error: Debug + Display {}

// pub fn eprintln<T>(result: Result<T>) {
//     match result {
//         Ok(_) => unreachable!(),
//         Err(e) => eprintln!("qcc: {e}"),
//     }
// }

#[derive(Debug)]
pub(crate) enum QccErrorKind {
    NoFile,
    AttributeMissing,
}

impl QccErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        use QccErrorKind::*;

        match *self {
            NoFile => "no such file",
            AttributeMissing => "missing attribute",
        }
    }
}

impl Display for QccErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// TODO: Custom error type.
#[derive(Debug)]
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

        let e1: Result<()> = Err(QccError(AttributeMissing));
        match e1 {
            Ok(_) => unreachable!(),
            Err(ref e) => assert_eq!(e.to_string(), "qcc: missing attribute"),
        }

        let e2: Result<()> = Err(QccError(NoFile));
        match e2 {
            Ok(_) => unreachable!(),
            Err(ref e) => assert_eq!(e.to_string(), "qcc: no such file"),
        }
        Ok(())
    }
}
