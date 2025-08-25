import math::sin;
import math::cos;
import math::exp;

// import math::idk;    // unknown import should throw error
// import toss::toss;   // cyclic import should throw error

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

