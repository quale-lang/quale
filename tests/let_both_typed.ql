fn foo() : qbit {
    let q = 0q(0, 1);  // represent a qubit in zero state simply as 0
    q
}

fn main() {
    let choice : qbit = foo();
    if choice == 0 {
        print("Heads");
    } else {
        print("Tails");
    }
}


