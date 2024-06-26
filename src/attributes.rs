//! Attributes: Function definitions can have certain attributes associated to
//! them. What are these attributes and what they function isn't defined right
//! now.
use crate::error::{QccErrorKind, QccErrorLoc};
use crate::lexer::Location;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub(crate) enum Attribute {
    Deter,
    #[default]
    NonDeter,
}

impl std::str::FromStr for Attribute {
    type Err = QccErrorKind; // at this point, we can only infer the kind of
                             // error, location cannot be determined here, but
                             // can be tagged along down the call stack.

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(match s {
            "deter" => Self::Deter,
            "nondeter" => Self::NonDeter,
            _ => Err(QccErrorKind::UnexpectedAttr)?,
        })
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

impl Attributes {
    /// Check if object contains no attributes.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Push a single `Attribute` to the `Attributes` object.
    pub(crate) fn push(&mut self, attr: Attribute) {
        self.0.push(attr);
    }
}

impl std::str::FromStr for Attributes {
    type Err = QccErrorLoc; // we can only infer a partial location for this
                            // error (along with its kind, which we get from
                            // parsing single Attribute), so we return a
                            // LocationRef.

    /// Assuming we have a list of attributes in the form: #[attr1, attr2, ...]
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let mut col: usize = 0; // marks the column index location

        let s = s.trim_start_matches("#[").trim_end_matches(']');
        col += 2; // for '#['

        // FIXME: This will loose information if separator has more whitespaces.
        let attrs: Vec<&str> = s.split_terminator(',').map(|x| x.trim()).collect();

        let mut parsed: Self = Default::default();
        let mut first = true;

        for attr in attrs {
            if first {
                first = !first;
            }

            match attr.parse::<Attribute>() {
                Ok(a) => {
                    parsed.0.push(a);

                    if first {
                        col += attr.len();
                    } else {
                        col += 2 + attr.len();
                    }
                }
                Err(kind) => {
                    Err((kind, Location::new("", 0, col)))?;
                }
            }
        }

        Ok(parsed)
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attrs = self
            .0
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "{attrs}")
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
        assert!(err == (QccErrorKind::UnexpectedAttr, Location::new("", 0, 12)).into());
    }
}
