//! Attributes: Function definitions can have certain attributes associated to
//! them. What are these attributes and what they function isn't defined right
//! now.

#[derive(Default, Copy, Clone)]
pub(crate) enum Attribute {
  Deter,
  #[default]
  NonDeter,
}

impl std::str::FromStr for Attribute {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let attr = match s {
      "deter" => Self::Deter,
      "nondeter" => Self::NonDeter,
      _ => Err(format!("qcc: unexpected attribute [{}]", s))?,
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

#[derive(Default, Clone)]
pub(crate) struct Attributes(pub(crate) Vec<Attribute>);

impl std::str::FromStr for Attributes {
  type Err = Box<dyn std::error::Error>;

  /// Assuming we have a list of attributes in the form: #[attr1, attr2, ...]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let s = s.trim_start_matches("#[").trim_end_matches(']');
    let attrs: Vec<&str> = s.split_terminator(',').map(|x| x.trim()).collect();
    Ok(Attributes(
      attrs
        .iter()
        .map(|x| x.parse::<Attribute>().unwrap())
        .collect(),
    ))
  }
}

impl std::fmt::Display for Attributes {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.iter().fold(true, |first, elem| {
      // FIXME: Returning type.
      if !first {
        write!(f, ", ");
      }
      write!(f, "{}", elem);
      false
    });
    Ok(())
  }
}
