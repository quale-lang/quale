module lib {

fn bar() : qbit {
    return 0q(1.0, 0.0);
}

fn sin(r: f64) : f64 {
    return (r / 180);
}

fn cos(r: f64) : f64 {
    return (r / 90);
}

}

import lib::sin;
import lib::cos;
import lib::far;
import timbuktu::sin;

#[nondeter]
fn new (b: bit) {
    let q = 0q(1, 0);
    return q; // to construct a qbit we should only require at least one bit value
          // and its associated probability amplitude
}

fn bar(x: f64, y: f64) : f64 {
    return (x + y) / 42;
}

fn main() {
    let a: f64 = 3.14;
    let e0: f64 = 1;
    let nonce = a;
    let e1 = e0;
    let f2 : bit = bar(e0 * cos(a) / nonce, -e1 * sin(a));
    return f2;
}

