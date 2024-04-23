// deterministic function
// This function will be the entrypoint for program, and will be responsible
// for collapsing of quantum states through measurement operator.
#[deter]
fn main () {
    // #[deter] this is comment
    foo()
}

// non-deterministic quantum function
#[nondeter]
fn foo () {
    -   42.4265
}
