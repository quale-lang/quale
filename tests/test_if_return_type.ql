fn unfair_toss(b: bit) : qbit {
    return if b {
        return 0q(0, 1);
    } else {
        return 0q(1, 0);
    };
}

fn main() {
    let choice = unfair_toss(1);
}
