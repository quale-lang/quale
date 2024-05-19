module lib {

fn bar(x: f64, y: f64) : f64 {
    return 0;
}

fn sin(r: f64) : f64 {
    return (r / 180);
}

fn cos(r: f64) : f64 {
    return (r / 90);
}

}

fn bar(x: f64, y: f64) : f64 {
    return (x + y) / 42;
}

fn main() {
    let a: rad = 3.14;
    let e0: f64 = 1;
    let nonce = a;
    let e1 = e0;
    let f2: f64 = bar(e0 * cos(a) / nonce, -e1 * sin(a));
    return f2;
}

