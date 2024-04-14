# qcc - compiler toolchain for quale language

`qcc` is the compiler for quale language. Building `qcc` is straightforward if
you have a functioning rust toolchain.

```bash
cargo build --release
```

To test the compiler, you can do

```bash
cargo test
```

And for installing the compiler in your machine, run

```bash
cargo install --path .
```

This will install `qcc` binary in `$HOME/.cargo/bin/`. Run `qcc --help` for
available options.
