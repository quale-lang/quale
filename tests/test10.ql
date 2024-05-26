module lib {

fn square(x: f64) : f64 {
    return x * x;
}

}

import lib::square;
import lib::sqrt; // Error
import nolib::square; // Error

fn main() {
    let x = square(42);
    return x;
}
