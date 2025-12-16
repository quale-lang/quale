fn foo(q0: qbit) : qbit {
    let q1 = 2 * q0;
    return q1;
}

fn bar(q0: qbit) : qbit {
    let q1 = q0 * 2;
    return q1;
}

fn main() {
    let x = foo();
    let y = bar();
}
