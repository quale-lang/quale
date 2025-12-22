import complex_expr_lib::sin;
import complex_expr_lib::cos;

alias S = sin;
alias C = cos;
// alias F = foo; // FIXME

fn foo(q0: qbit) : qbit { return q0; }

fn main() {
    let x = S(1);
    let y = C(0);
    return x + y;
}
