fn foo() : qbit {
    let q = 0q(0, 1);  // represent a qubit in zero state simply as 0
    return q;
}

fn main() {
    let choice : qbit = foo();
    if choice == 0 {
        let _ = print(0);
    } else {
        let _ = print(1);
    }
}


