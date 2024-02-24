//! Attributes: Function definitions can have certain attributes associated to
//! them. What are these attributes and what they function isn't defined right
//! now.
use crate::error::{QccError, QccErrorKind, Result};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub(crate) enum Attribute {
    Deter,
    #[default]
    NonDeter,
}

impl std::str::FromStr for Attribute {
    type Err = QccError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let attr = match s {
            "deter" => Self::Deter,
            "nondeter" => Self::NonDeter,
            _ => Err(QccErrorKind::UnexpectedAttr)?,
        };
        Ok(attr)
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Deter => write!(f, "deter"),
            Attribute::NonDeter => write!(f, "nondeter"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Attributes(pub(crate) Vec<Attribute>);

impl std::str::FromStr for Attributes {
    type Err = QccError;

    /// Assuming we have a list of attributes in the form: #[attr1, attr2, ...]
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let s = s.trim_start_matches("#[").trim_end_matches(']');
        let attrs: Vec<&str> = s.split_terminator(',').map(|x| x.trim()).collect();

        let mut parsed: Self = Default::default();
        for attr in attrs {
            let parsed_attr = attr.parse::<Attribute>();
            match parsed_attr {
                Ok(a) => parsed.0.push(a),
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(parsed)
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().fold(true, |first, elem| {
            // FIXME: perhaps needs try_for_*
            if !first {
                write!(f, ", ");
            }
            write!(f, "{}", elem);
            false
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_attrs() {
        use Attribute::*;

        let s = "#[deter, nondeter]";
        let attrs = s.parse::<Attributes>().unwrap();
        assert_eq!(attrs, Attributes(vec![Deter, NonDeter]));

        let s = "#[nondeter, unknown]";
        let err = s.parse::<Attributes>().err().unwrap();
        assert!(err == QccError(QccErrorKind::UnexpectedAttr));
    }
}
