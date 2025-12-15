import std::Hadamard;

fn toss() : qbit {
    let zero_state = 0q(0, 1);  // represent a qubit in zero state simply as 0
    let superpositioned = Hadamard(zero_state);
    superpositioned
}

fn main() {
    let choice = toss();
    if choice == 0 {
        print("Heads");
    } else {
        print("Tails");
    }
}

