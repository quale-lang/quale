module lib {
    extern fn bar(x: f64, y: f64) : f64;
    extern fn sin(r: rad) : rad;
    extern fn cos(r: rad) : rad;
}

fn bar(x: f64, y: f64) : f64 {
    return (x + y) / 42;
}

fn main() {
    let a: rad = 3.14;
    let e0: f64 = 1;
    let nonce: f64 = 10;
    let e1: f64 = e0;
    let f2 = bar(e0 * cos(a) / nonce, -e1 * sin(a));
    return f2;
}

