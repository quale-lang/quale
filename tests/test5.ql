// This is a comment.
// And this is a follow-up.
// This is a comment.
#[deter, nondeter]
fn main (input: qbit) : bit {
  if input.is_ok() {
    return reinterpret_cast<known>(input);
  } else {
    return err();
  }
}

