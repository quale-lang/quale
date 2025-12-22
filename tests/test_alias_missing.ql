alias H = Hadamard;

fn main(): !qbit {
    let q = 0q(0, 1);
    let x = H(q);

    return x;
}
