// Negate operator is unsupported.
// fn BooleanFlip(b: bool) : bool {
//     return !b;
// }

fn BooleanConstant(b: bool) : bool {
    return b;
}

fn True() : bool {
    return true;
}

fn False() : bool {
    return false;
}

fn main() : bool {
    let x = True();
    let y = False();
    if x == y {
        let _ = BooleanConstant(x);
    } else {
        let _ = BooleanConstant(y);
    }

    return x + y;
}
