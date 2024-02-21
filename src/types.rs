//! A rudimentary type system for qcc.
//!
//! Read more on quantum language type systems.

#[derive(Default)]
pub(crate) struct Type {}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "<unknown-type>")
  }
}
