// This is a comment

#[attributes]
main <T1, T2> (param1: T1, param2: T1) : T2 {
    return param1<T2> + param2<T2>
}
