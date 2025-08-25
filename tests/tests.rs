use qcc::assert_eq_any;
use qcc::codegen::{qasm, Translator};
use qcc::error::QccErrorKind;
use qcc::inference::infer;
use qcc::parser::Parser;

#[macro_export]
macro_rules! test {
    ($path:expr, $repr:expr) => {{
        println!(">> Testing {} ...", $path);

        let mut parser = Parser::new(vec![$path])?.unwrap();
        let config = parser.get_config();
        let mut ast = parser.parse(&config.analyzer.src)?;

        match infer(&mut ast) {
            Ok(_) => {
                // TODO: write own macro to assert ast, so that diff is better
                // presented at test failure. The stdlib macros just dump left
                // and right strings without showing the diff.
                assert_eq!(format!("{}", ast), $repr);
            }
            Err(err) => {
                assert_eq_any!(err, [QccErrorKind::TypeError]);
            }
        }
    }};
}

// Always keep a newline between two test macro invocations.
#[rustfmt::skip]
#[test]
fn test_ast_gen() -> Result<(), Box<dyn std::error::Error>>  {
    // TODO: error, but how to reflect this in
    // macro? We can do Result<&str, QccError>.
    // The commented out tests are those which fail compilation (either at
    // lexing/parsing stage, or at type checking stage).
    // test!("tests/attr-panic.ql", "");

//     // FIXME: This is failing cargo test *randomly*.
//     // This is a big bug where wrong types are being inferred. So based on the
//     // code execution either one kind of type or other is inferred. Ambiguity in
//     // type inference is due to the lack of support for measurement operator. So
//     // if a function is returning a qubit, then the caller expression may be of
//     // type qubit, or it may be of a concrete type if lhs is already typed
//     // before. This should be fixed once the codegen will support addition of
//     // measurement operator.
//     test!("tests/complex-expr.ql",
// "|_ lib			// @complex-expr.ql:1:1
//   |_ fn bar () : qubit		// @complex-expr.ql:3:4
//     |_ 0q1_0

//   |_ fn sin (r: float64) : float64		// @complex-expr.ql:7:4
//     |_ (r: float64 / 180)

//   |_ fn cos (r: float64) : float64		// @complex-expr.ql:11:4
//     |_ (r: float64 / 90)

// |_ complex_expr			// @complex-expr.ql:1:1
//   |_ fn [[nondeter]] new (b: bit) : qubit		// @complex-expr.ql:21:4
//     |_ q: qubit = 0q1_0
//     |_ q: qubit

//   |_ fn bar (x: float64, y: float64) : float64		// @complex-expr.ql:27:4
//     |_ ((x: float64 + y: float64) / 42)

//   |_ fn main () : qubit		// @complex-expr.ql:31:4
//     |_ a: float64 = 3.14
//     |_ e0: float64 = 1
//     |_ nonce: float64 = a: float64
//     |_ e1: float64 = e0: float64
//     |_ f2: qubit = bar: qubit (((e0: float64 * lib$cos: float64 (a: float64)) / nonce: float64), (-e1: float64 * lib$sin: float64 (a: float64)))
//     |_ f2: qubit

// ");

    // test!("tests/expected-attr.ql", "");

    // test!("tests/let-as-expr.ql", "");

    test!("tests/let-both-typed.ql",
"|_ let_both_typed			// @let-both-typed.ql:1:1
  |_ fn let_both_typed$foo () : qubit		// @let-both-typed.ql:1:4
    |_ q: qubit = 0q0_1

  |_ fn let_both_typed$main () : qubit		// @let-both-typed.ql:6:4
    |_ choice: qubit = let_both_typed$foo: qubit ()

");

    // test!("tests/let-fn-call.ql", "");

    test!("tests/no-eof.ql",
"|_ no_eof			// @no-eof.ql:1:1
  |_ fn [[nondeter]] no_eof$main (param: qubit) : float64		// @no-eof.ql:2:20
    |_ 0

");

    test!("tests/only-whitespaces-no-eof.ql",
"|_ only_whitespaces_no_eof			// @only-whitespaces-no-eof.ql:1:1
");

    test!("tests/qbit-float.ql",
"|_ qbit_float			// @qbit-float.ql:1:1
  |_ fn qbit_float$foo (q0: qubit) : qubit		// @qbit-float.ql:1:4
    |_ q1: qubit = (2 * q0: qubit)

  |_ fn qbit_float$bar (q0: qubit) : qubit		// @qbit-float.ql:6:4
    |_ q1: qubit = (q0: qubit * 2)

  |_ fn qbit_float$main () : qubit		// @qbit-float.ql:11:4
    |_ x: qubit = qbit_float$foo: qubit ()
    |_ y: qubit = qbit_float$bar: qubit ()

");

    // test!("tests/tabbed-comments-fn.ql", "");

    test!("tests/tabbed-comments.ql",
"|_ _foo			// @tabbed-comments.ql:1:1
|_ tabbed_comments			// @tabbed-comments.ql:1:1
");

    // test!("tests/tensors.ql", "");

    test!("tests/tensors2.ql",
"|_ tensors2			// @tensors2.ql:1:1
  |_ fn tensors2$sin (x: float64) : float64		// @tensors2.ql:1:4

  |_ fn tensors2$cos (x: float64) : float64		// @tensors2.ql:5:4

  |_ fn tensors2$main () : <bottom>		// @tensors2.ql:9:4
    |_ t1 = [[], []]
    |_ t2 = [[[], []], [[]]]

  |_ fn tensors2$foo () : float64		// @tensors2.ql:14:4
    |_ x: float64 = 42
    |_ e0: float64 = 2.718
    |_ e1: float64 = (e0: float64 * 2)
    |_ a: float64 = 0.707
    |_ t1 = []
    |_ t2: float64 = [x: float64]
    |_ t3: float64 = [t2: float64, t2: float64]
    |_ t4: float64 = [(e0: float64 * tensors2$cos: float64 (a: float64)), (-e1: float64 * tensors2$sin: float64 (a: float64))]
    |_ t5 = [[]]
    |_ t7: float64 = [[x: float64]]
    |_ t8: float64 = [[(e0: float64 * tensors2$cos: float64 (a: float64)), (-e1: float64 * tensors2$sin: float64 (a: float64))], [(e1: float64 * tensors2$sin: float64 (a: float64)), (e0: float64 * tensors2$cos: float64 (a: float64))]]
    |_ t6 = [[], []]
    |_ t9 = [[[], []], [[]]]
    |_ t3: float64

");

    // test!("tests/test1.ql", "");

    // test!("tests/test2.ql", "");

    test!("tests/test3.ql",
"|_ test3			// @test3.ql:1:1
");

    test!("tests/test4.ql",
"|_ test4			// @test4.ql:1:1
");

    // test!("tests/test5.ql", "");

    // test!("tests/test6.ql", "");

    // test!("tests/test7.ql", "");

    // test!("tests/test8.ql", "");

    test!("tests/test9.ql",
"|_ test9			// @test9.ql:1:1
  |_ fn test9$main () : float64		// @test9.ql:1:4
    |_ x: float64 = -42

");

    // test!("tests/test10.ql", "");

    test!("tests/test11.ql",
"|_ test11			// @test11.ql:1:1
  |_ fn test11$main () : float64		// @test11.ql:3:4
    |_ x: float64 = 42
    |_ y: float64 = (x: float64 - 99)
    |_ y: float64

");

    test!("tests/test12.ql",
"|_ test12			// @test12.ql:1:1
  |_ fn test12$create_new_state (b: bit) : qubit		// @test12.ql:1:4
    |_ q: qubit = b: bit
    |_ q: qubit

");

    test!("tests/test13.ql",
"|_ test13			// @test13.ql:1:1
  |_ fn test13$foo (transform: float64, q0: qubit) : qubit		// @test13.ql:1:4
    |_ tmp: qubit = (transform: float64 * q0: qubit)
    |_ (transform: float64 * q0: qubit)

  |_ fn test13$main () : qubit		// @test13.ql:6:4
    |_ choice: qubit = test13$foo: qubit ()

");

    test!("examples/toss.ql",
"|_ toss			// @toss.ql:1:1
  |_ fn toss$sin (x: float64) : float64		// @toss.ql:2:4

  |_ fn toss$cos (x: float64) : float64		// @toss.ql:7:4

  |_ fn toss$exp (x: float64) : float64		// @toss.ql:11:4

  |_ fn toss$U (theta: float64, phi: float64, lambda: float64, q0: qubit) : qubit		// @toss.ql:15:4
    |_ e0: float64 = toss$exp: float64 (((phi: float64 + lambda: float64) / 2))
    |_ e1: float64 = toss$exp: float64 (((phi: float64 - lambda: float64) / 2))
    |_ a: float64 = (theta: float64 / 2)
    |_ transform: float64 = [[(e0: float64 * toss$cos: float64 (a: float64)), (-e1: float64 * toss$sin: float64 (a: float64))], [(e1: float64 * toss$sin: float64 (a: float64)), (e0: float64 * toss$cos: float64 (a: float64))]]
    |_ (transform: float64 * q0: qubit)

  |_ fn toss$Hadamard (q: qubit) : qubit		// @toss.ql:30:4
    |_ pi: float64 = 3.14
    |_ toss$U: qubit ((pi: float64 / 2), 0, 0, q: qubit)

  |_ fn toss$toss () : qubit		// @toss.ql:35:4
    |_ zero_state: qubit = 0q0_1
    |_ superpositioned: qubit = toss$Hadamard: qubit (zero_state: qubit)

  |_ fn toss$main () : qubit		// @toss.ql:41:4
    |_ choice: qubit = toss$toss: qubit ()

");
    Ok(())
}

#[test]
fn compile() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./tests")?;

    // TODO: Design a macro for initializing a compile session and also have
    // traits to assert arount ast values. This way, we can simply do:
    // macro!(./tests/test1.ql, "description of ast" /*to match against*/)
    for p in paths {
        let path = p.unwrap().path().into_os_string().into_string().unwrap();
        if !path.ends_with(".ql") {
            continue;
        }

        let args = vec![path.as_str()];

        let mut parser = Parser::new(args)?.unwrap();
        let config = parser.get_config();

        match parser.parse(&config.analyzer.src) {
            Ok(mut ast) => {
                match infer(&mut ast) {
                    Ok(_) => {}
                    Err(err) => {
                        assert_eq_any!(err, [QccErrorKind::TypeError]);
                        continue;
                    }
                }

                match qasm::QasmModule::translate(ast) {
                    Ok(_) => {}
                    Err(err) => assert_eq_any!(err, [QccErrorKind::TranslationError]),
                }
            }

            Err(err) => assert_eq_any!(
                err,
                [
                    QccErrorKind::NoFile,
                    QccErrorKind::CmdlineErr,
                    QccErrorKind::UnknownImport,
                    QccErrorKind::LexerError,
                    QccErrorKind::ParseError
                ]
            ),
        }
    }

    Ok(())
}

#[test]
fn cmdline() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./tests")?;

    for p in paths {
        let path = p.unwrap().path().into_os_string().into_string().unwrap();
        if !path.ends_with(".ql") {
            continue;
        }
        let args = vec![path.as_str(), "-O2", "--analyze"];
        let _config = Parser::parse_cmdline(args)?.unwrap();
    }

    Ok(())
}

#[test]
fn non_existing_src() -> Result<(), Box<dyn std::error::Error>> {
    let path = "./tests/test-non-existent.ql";
    let args = vec![path, "--analyze"];
    match Parser::new(args) {
        Ok(_) => unreachable!(),
        Err(err) => assert_eq!(err, QccErrorKind::CmdlineErr.into()),
    }
    Ok(())
}

#[test]
fn analyzer() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./tests")?;

    for p in paths {
        let path = p.unwrap().path().into_os_string().into_string().unwrap();
        if !path.ends_with(".ql") {
            continue;
        }
        let args = vec![path.as_str(), "--analyze"];
        let mut parser = Parser::new(args)?.unwrap();
        let config = parser.get_config();

        match parser.parse(&config.analyzer.src) {
            Ok(ast) => {
                config.analyzer.analyze(&ast)?;
            }
            Err(err) => assert_eq_any!(
                err,
                [
                    QccErrorKind::NoFile,
                    QccErrorKind::CmdlineErr,
                    QccErrorKind::UnknownImport,
                    QccErrorKind::LexerError,
                    QccErrorKind::ParseError
                ]
            ),
        }
    }

    Ok(())
}

#[test]
fn check_output_directives() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./tests")?;
    for p in paths {
        let path = p.unwrap().path().into_os_string().into_string().unwrap();
        let path = path.as_str();
        let temp = "temp.s";

        {
            let arg = vec!["-o", temp, path];
            let parser = Parser::new(arg)?.unwrap();
            let config = parser.get_config();
            assert_eq!(config.analyzer.src, path);
            assert_eq!(config.optimizer.asm, temp);
        }

        {
            let arg = vec![path, "-o", temp];
            let parser = Parser::new(arg)?.unwrap();
            let config = parser.get_config();
            assert_eq!(config.analyzer.src, path);
            assert_eq!(config.optimizer.asm, temp);
        }

        {
            let arg = vec![path];
            let parser = Parser::new(arg)?.unwrap();
            let config = parser.get_config();
            assert_eq!(config.analyzer.src, path);
            assert_eq!(config.optimizer.asm, path.replace(".ql", ".s"));
        }

        {
            let arg = vec![path, "-o", "-o", temp];
            let parser = Parser::new(arg)?.unwrap();
            let config = parser.get_config();
            assert_eq!(config.analyzer.src, path);
            assert_eq!(config.optimizer.asm, temp);
        }

        {
            let arg = vec![path, "-o"];
            let parser = Parser::new(arg)?.unwrap();
            let config = parser.get_config();
            assert_eq!(config.analyzer.src, path);
            assert_eq!(config.optimizer.asm, path.replace(".ql", ".s"));
        }

        break;
    }
    Ok(())
}

#[test]
fn check_package() -> Result<(), Box<dyn std::error::Error>> {
    let main = "./tests/package/toss.ql";
    let args = vec![main];

    let mut parser = Parser::new(args)?.unwrap();
    let config = parser.get_config();

    match parser.parse(&config.analyzer.src) {
        Ok(mut ast) => {
            match infer(&mut ast) {
                Ok(_) => {}
                Err(err) => {
                    assert_eq_any!(err, [QccErrorKind::TypeError]);
                }
            }

            match qasm::QasmModule::translate(ast) {
                Ok(_) => {}
                Err(err) => assert_eq_any!(err, [QccErrorKind::TranslationError]),
            }
        }

        Err(err) => assert_eq_any!(
            err,
            [
                QccErrorKind::NoFile,
                // TODO: How to reference that package is throwing CyclicImport
                // error.
                // QccErrorKind::UnknownImport,
                // QccErrorKind::ParseError
            ]
        ),
    }

    Ok(())
}
