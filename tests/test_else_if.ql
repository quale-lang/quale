fn pseudo_random() : f64 {
    return 42;
}

fn main() {
    let choice = pseudo_random();

    if choice == 0 {
        let x = 42;
    } else if choice == 1 {
        let x = 41;
    } else if choice == 2 {
        let x = 40;
    } else {
        let x = 0;
    }
}
