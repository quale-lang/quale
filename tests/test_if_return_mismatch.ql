fn mismatched_types(b: bit) : qbit {
    return if b {
        return 0q(0, 1);
    } else {
        return 32;
    };
}

fn main() {
    let x = mismatched_types(1);
}

