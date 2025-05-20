// For small values
fn sin(x: f64) : f64 {
    x
}

// For small values
fn cos(x: f64) : f64 {
    1 - x
}

fn exp(x: f64) : f64 {
    2.718 ** x
}

fn U (theta: f64, phi: f64, lambda: f64, q0: qbit) : qbit {
    // U (θ,φ,λ) = [ε cos(θ/2)  -ε'sin(θ/2)]
    //             [ε'sin(θ/2)   ε cos(θ/2)]
    //               where ε = e^(-i(φ+λ)/2) and ε' = e^(-i(φ-λ)/2)
    let e0 = exp ((phi + lambda) / 2);
    let e1 = exp ((phi - lambda) / 2);
    let a  = theta / 2;

    // This equation is the bread and butter for our transformations.
    let transform = [[e0 * cos(a), -e1 * sin(a)], [e1 * sin(a),  e0 * cos(a)]];

    // return the transformed qubit
    return transform * q0;
}

fn Hadamard(q: qbit) : qbit {
    let pi: f64 = 3.14;
    return U(pi/2, 0, 0, q);
}

fn toss() : qbit {
    let zero_state = 0q(0, 1);  // represent a qubit in zero state simply as 0
    let superpositioned = Hadamard(zero_state);
    superpositioned
}

fn main() {
    let choice : qbit = toss();
    if choice == 0 {
        print("Heads");
    } else {
        print("Tails");
    }
}

