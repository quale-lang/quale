fn main() {
    let x = 1 != 2;
    let y = 3 == 3;

    let b0 = 1 < 2;
    let b1 = 2 > 3;
    let b2 = 1 <= 2;
    let b3 = 4 >= 5;

    let a0 = x < y;
    let a1 = x > y;
    let a2 = x <= y;
    let a3 = x >= y;

    let w = a0 == a0;
    let z = a0 != a1;

    let mix0 = a0 == 1;
    let mix1 = 1 != a0;
    let mix2 = a0 < 1;
    let mix3 = 1 > a0;
    let mix4 = a0 <= 1;
    let mix5 = 1 >= w;

    return a0 + a1 + a2 + a3;
}
