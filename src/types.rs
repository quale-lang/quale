//! A rudimentary type system for qcc.
//!
//! Read more on quantum language type systems.

use crate::error::QccErrorKind;

#[derive(Default, PartialEq)]
pub(crate) enum Type {
    Known,
    #[default]
    Unknown,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Known => write!(f, "<known>"),
            Self::Unknown => write!(f, "<unknown-type>"),
        }
    }
}

impl std::str::FromStr for Type {
    type Err = QccErrorKind; // at this point, we can only infer the kind of
                             // error, location cannot be determined here, but
                             // can be tagged along down the call stack.

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(match s {
            "known" => Self::Known,
            "unknown" => Self::Unknown,
            // _ => Err(QccErrorKind::UnexpectedType)?,
            _ => Self::Unknown,
        })
    }
}
