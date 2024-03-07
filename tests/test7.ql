#[deter, nondeter]
fn main() {
  foo()
}

#[nondeter, unknown_attr, deter]
fn foo() {
  42
}
 
