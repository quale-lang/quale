//! A rudimentary type system for qcc.
//!
//! Read more on quantum language type systems.

use crate::error::QccErrorKind;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd)]
pub(crate) enum Type {
    // Don't change the order. Partial ordering in place.
    #[default]
    Bottom,
    Bit,
    Qbit,
    Rad,
    F64,
}

impl Type {
    #[inline]
    pub(crate) fn bigtype(t1: Type, t2: Type) -> Type {
        return if t1 > t2 { t1 } else { t2 };
    }

    #[inline]
    pub(crate) fn smalltype(t1: Type, t2: Type) -> Type {
        return if t1 < t2 { t1 } else { t2 };
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bottom => write!(f, "<bottom>"),
            Self::Rad => write!(f, "radians"),
            Self::Qbit => write!(f, "qubit"),
            Self::Bit => write!(f, "bit"),
            Self::F64 => write!(f, "float64"),
        }
    }
}

impl std::str::FromStr for Type {
    type Err = QccErrorKind; // at this point, we can only infer the kind of
                             // error, location cannot be determined here, but
                             // can be tagged along down the call stack.

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(match s {
            "rad" => Self::Rad,
            "qbit" => Self::Qbit,
            "bit" => Self::Bit,
            "f64" => Self::F64,
            _ => Err(QccErrorKind::UnexpectedType)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_types() {
        assert!(Type::F64 > Type::Qbit);
    }
}
