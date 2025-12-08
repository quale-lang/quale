import complex_expr_lib::sin;
import complex_expr_lib::cos;

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
    let f2 = bar(e0 * cos(a) / nonce, -e1 * sin(a));
    return f2;
}

