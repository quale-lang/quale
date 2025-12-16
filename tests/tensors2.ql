fn foo(x: f64) : f64 {
    return x;
}

fn bar(x: f64) : f64 {
    return x;
}

fn main() {
    let t1 = [[], []];
    let t2 = [[[], []], [[]]];
}

fn foo2() {
    let x = 42;
    let e0 = 2.718;
    let e1 = e0 * 2;
    let a = 0.707;

    let t1 = [];
    let t2 = [x];
    let t3 = [t2, t2];
    let t4 = [e0 * bar(a), -e1 * foo(a)];
    let t5 = [[]];
    let t7 = [[x]];
    let t8 = [[e0 * bar(a), -e1 * foo(a)], [e1 * foo(a),  e0 * bar(a)]];
    let t6 = [[], []]; // FIXME
    let t9 = [[[], []], [[]]]; // FIXME
    return t3;
}
