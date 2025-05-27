fn foo(transform: f64, q0: qbit) : qbit {
    let tmp: qbit = transform * q0;
    return transform * q0;
}

fn main() {
    let choice : qbit = foo();
}

