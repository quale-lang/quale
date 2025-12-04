fn foo() : qbit {
    let x : qbit = 0q(0, 1);
    return x;
}

fn main() {
    let choice = foo();
    if choice == 0 {
        let x = 42;
        let _ = print(0);
    } else {
        let x = 2;
        let _ = print(1);
    }

    if 1 != 2 {
        let x = 42;
    } else {
        let x = 32;
    }
}
