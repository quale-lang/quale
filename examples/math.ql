fn factorial(n: f64): f64 {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

// These approximations for sin/cos will do just fine
// for x ∈ [-1,1] radians
fn sin(x: f64) : f64 {
    let cube = x*x*x;
    let fact = factorial(3);
    return x - (cube / fact);
}

fn cos(x: f64) : f64 {
    let sqre = x*x;
    let fact = factorial(2);
    return 1.0 - (sqre / fact);
}

fn exp(x: f64) : f64 {
    let e = 2.718;
    return e * x;
}

