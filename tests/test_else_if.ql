fn pseudo_random() : f64 {
    return 42;
}

fn main() {
    let choice = pseudo_random();

    if choice == 0 {
        let _ = print(0);
    } else if choice == 1 {
        let _ = print(1);
    } else if choice == 2 {
        let _ = print(2);
    } else {
        let x = 42;
    }
}
