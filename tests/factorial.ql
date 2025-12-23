fn factorial(n: f64): f64 {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}
