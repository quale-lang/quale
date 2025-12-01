fn pseudo_random() : f64 {
    return 42;
}

fn main() {
    let choice = pseudo_random();

    if choice == 0 {
        let _ = print("Heads");
    } else if choice == 1 {
        let _ = print("Tails");
    } else if choice == 2 {
        let _ = print("Don't know");
    } else {
        let x = 42;
    }
}
