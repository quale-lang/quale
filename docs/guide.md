# An Introduction to Quale Programming Language

Quale is a programming language for quantum computers. Which quantum computer,
you may ask. Well I must confess, programs written in quale do not run on a
quantum computer yet. I also don't have a quantum computer lying around.[^rpi]

There is an open standard called [OpenQASM](https://openqasm.com/) which acts as
an intermediate representation for high-level languages mapped onto a quantum
hardware. And this is what primitively quale is about. It is a high-level
language which gets translated to OpenQASM via its compiler `qcc`.

[^rpi]: "It is so easy and fun to write compilers when an actual hardware is
lying around." says my arm-based RPi from a dusty shelf.

## Installation

Installing quale is simple if you have a working rust toolchain. As the
compiler is writtern in Rust, you can fetch the source and directly build it
yourself.

```bash
git clone https://github.com/quale-lang/quale.git
cd quale
cargo build --release
cargo install --path .
```

The last command `cargo install` will install `qcc` binary in your
`$HOME/.cargo/bin/`.

## Using the compiler

The help page can be viewed by using `--help` or `-h` command-line option.

```bash
$ qcc --help
usage: qcc [options] <quale-file>
    --help              show this page      
    --dump-ast          print AST           
    --dump-ast-only     print AST without translating to assembly
    --dump-qasm         print OpenQASM IR   
    --analyze           run static analyzer 
    -O0                 disable optimizations (NA)
    -O1                 enable first-level optimizations (NA)
    -Og                 enable all optimizations (NA)
    -o                  compiled output     
```

The options are pretty self-explanatory but the `--analyze` and all of
optimization options don't do anything. They are placeholders for now. `--dump`
options print the requested IR on the standard output.

We will begin our quick introduction with quale by simulating a fair coin toss.
In classical computing, if we have to write a coin toss program, we will call a
random number generator (RNG) over the range 0 to 1 (both inclusive). And we
would assume 0 to represent heads and 1 to represent tails. But the fairness of
this coin toss is dependent upon the RNG we will use. We _simulate_ randomness in
classical computing relying on entropy of the system. The more noise we can
detect, the better RNG works. But in quantum computing, randomness comes in its
purest form. I won't go into the details of how it occurs. But remember that
putting a `qubit`, which is a datatype we will use often, in
<u>superposition</u> and subsequently measuring it can give us a purely random
result.

We will work on this problem stepwise. Firstly, the `main` function which
usually is the entrypoint in many languages is back. We also start with the
`main` function to write our code. As it happens generally, exactly one
`main` function should be reachable by the compiler, so that it can set the
entrypoint for execution in our resulting code.

```quale
fn main () {
    return 42;  // only a classical value can be returned
}
```

In quale, `main` belongs to one of the two categories of functions. It is a
<u>deterministic function</u>, which means that it can only return a classical
bit (or value). There can be functions of <u>non-deterministic</u> type which
may return a quantum value (`qubit`). A non-deterministic function can also
return classical value, but at least one of the arguments should be a quantum
value. This category is important to consider and its necessity stems from the
[no-cloning theorem](https://en.wikipedia.org/wiki/No-cloning_theorem) in
quantum computing. This theorem states that it is impossible to create an
independent and identical copy of a quantum state. So you can imagine, there are
certains things we aren't allowed to do in a non-deterministic function, like
consistently mutating a variable state in a loop.

To keep the syntax simple, there is no language-defined keyword or attribute to
distinguish between these two categories of functions. Quale's type system,
which is Peter Selinger's type system presented in his paper [1], handles
this discretion for us. This means that you can write correct code without
manually ensuring no-cloning stands for quantum values as long as the compiler
is happy.
 
Coming back to our fair coin toss example, we established that putting a qubit
in superposition and measuring it will give us a fairly picked choice between 0
and 1. And similarly to classical example, we will represent 0 with heads and 1
with tails. We can write this into our `main` function.

```quale
fn main() {
    let choice: bit = toss();
    if choice == 0 {
        print("Heads");
    } else {
        print("Tails");
    }
}
```

`toss` is a non-deterministic function which will return a fair choice to us. We
store the returned value in `choice` variable. `choice` is a classical type
value, so we can write it as `bit` (or `bool`).

Now let us introduce a non-deterministic function in our program.

```quale

fn toss() : qubit {...}
```

`toss` is returning a `qubit` value (the details of the function will be
expanded below). So, `toss` is a non-deterministic function. Also it is worth
mentioning that this qubit is returned but our variable `choice` _collapses_
this qubit into a bit and then stores it. This is the measurement part which we
talked about earlier. The measurement is simply a collapsing of the quantum
state into a classical state. Here, the measurement happens implicitly and is
kept hidden away from programmer's view point.

<u>Note</u>: The compiler upon seeing a mismatch where a qubit is used but a
bit was expected, puts a _measurement operator_ which takes in a qubit and
returns a classical bit. This mechanism provides simplicity for programmer that
they don't have to worry about measuring explicitly. The compiler will do this
job for them.

So a qubit in superposition when returned from toss, upon measurement will
either give us 0 or 1 with an equal probability. But how can we put a qubit in
superposition in the first place? For this we rely on the standard library for
quale which provides certain standard gates which can be applied to qubits in
function-like manner.

This `toss` function requires a two-step procedure to achieve what we aimed.
First we need to create a `qubit` in state 0. And then we put it in
superposition. Again as a matter of fact, we can put a qubit in superposition
by applying Hadamard gate to it.

```quale
fn toss() : qubit {
    let zero_state: qubit = 0;  // represent a qubit in zero state simply as 0
    let superpositioned = Hadamard(zero_state);
    superpositioned
}

fn main() {
    let choice: bit = toss();
    if choice == 0 {
        print("Heads");
    } else {
        print("Tails");
    }
}
```

`Hadamard` is another function (technically called a gate) that takes in a qubit
and returns a qubit. So its type would be `Hadamard: qubit -> qubit`. This will
be defined in standard library of quale. We will assume it as a black box for
now. The first `let` statement creates a qubit in state zero. We can represent 0
as either a classical bit 0 or a qubit in state zero (represented as |0〉). The
type determines whether it will be a bit or a qubit. The second `let` calls the
Hadamard function and receives a qubit in superposition.  Notice that,
`superpositioned` is inferred to be of qubit type (`toss`'s return type is qubit
and superpositioned is returned), so there wouldn't be a measurement operator
here.

This pretty much covers the program for a fair coin toss. We create a qubit in
state 0 and then put that qubit in superposition, followed by a measurement of
this superpositioned qubit which resulted in a 0 or 1 with exactly 0.5
probability.

Put the above code into a file `toss.ql` and you can dump its AST as `qcc
--dump-ast toss.ql`. This command will print the AST on stdout and also create a
`.s` file in the same place. This `.s` file is the readable format of OpenQASM
IR.

Before we move forward, it should be explained how these qubits are used in a
function call, because according to no-cloning theorem, we cannot possibly copy
it. :TODO:

# References

[1]: Selinger, P. and Valiron, B., 2009. Quantum lambda calculus. Semantic
techniques in quantum computation, pp.135-172.

