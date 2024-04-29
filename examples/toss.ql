fn toss() : qbit {
    let zero_state: qbit = 0;  // represent a qubit in zero state simply as 0
    let superpositioned = Hadamard(zero_state);
    superpositioned
}

fn main() {
    let choice: bit = toss();
    if choice == 0 {
        print("Heads");
    } else {
        print("Tails");
    }
}

