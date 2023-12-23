// deterministic function
// This function will be the entrypoint for program, and will be responsible
// for collapsing of quantum states through measurement operator.
#[deter]
main () {
    foo()
}

// non-deterministic quantum function
#[nondeter]
foo () {
    42
}
