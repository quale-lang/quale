#[cfg(test)]
use pretty_assertions::assert_eq;
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
                assert_eq!(format!("{}", ast), $repr, "AST did not match for {}", $path);
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
fn test_ast_gen() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: error, but how to reflect this in
    // macro? We can do Result<&str, QccError>.
    // The commented out tests are those which fail compilation (either at
    // lexing/parsing stage, or at type checking stage).
    // test!("tests/attr-panic.ql", "");

    test!("tests/complex_expr.ql",
"|_ complex_expr_lib			// @complex_expr_lib.ql:1:1
  |_ fn complex_expr_lib$bar () : qubit		// @complex_expr_lib.ql:2:4
    |_ 0q1_0

  |_ fn complex_expr_lib$sin (r: float64) : float64		// @complex_expr_lib.ql:6:4
    |_ (r: float64 / 180)

  |_ fn complex_expr_lib$cos (r: float64) : float64		// @complex_expr_lib.ql:10:4
    |_ (r: float64 / 90)

|_ complex_expr			// @complex_expr.ql:1:1
  |_ fn [[nondeter]] complex_expr$new (b: bit) : qubit		// @complex_expr.ql:5:4
    |_ q: qubit = 0q1_0
    |_ q: qubit

  |_ fn complex_expr$bar (x: float64, y: float64) : float64		// @complex_expr.ql:11:4
    |_ ((x: float64 + y: float64) / 42)

  |_ fn complex_expr$main () : float64		// @complex_expr.ql:15:4
    |_ a: float64 = 3.14
    |_ e0: float64 = 1
    |_ nonce: float64 = a: float64
    |_ e1: float64 = e0: float64
    |_ f2: float64 = complex_expr$bar: float64 (((e0: float64 * complex_expr_lib$cos: float64 (a: float64)) / nonce: float64), (-e1: float64 * complex_expr_lib$sin: float64 (a: float64)))
    |_ f2: float64

");

    // test!("tests/expected-attr.ql", "");

    // test!("tests/let-as-expr.ql", "");

    test!("tests/let_both_typed.ql",
"|_ let_both_typed			// @let_both_typed.ql:1:1
  |_ fn let_both_typed$foo () : qubit		// @let_both_typed.ql:1:4
    |_ q: qubit = 0q0_1

  |_ fn let_both_typed$main () : <bottom>		// @let_both_typed.ql:6:4
    |_ choice: qubit = let_both_typed$foo: qubit ()
    |_ (choice == 0)


");

    // test!("tests/let-fn-call.ql", "");

    test!("tests/no_eof.ql",
"|_ no_eof			// @no_eof.ql:1:1
  |_ fn [[nondeter]] no_eof$main (param: qubit) : float64		// @no_eof.ql:2:20
    |_ 0

");

    test!("tests/only_whitespaces_no_eof.ql",
"|_ only_whitespaces_no_eof			// @only_whitespaces_no_eof.ql:1:1
");

    test!("tests/qbit_float.ql",
"|_ qbit_float			// @qbit_float.ql:1:1
  |_ fn qbit_float$foo (q0: qubit) : qubit		// @qbit_float.ql:1:4
    |_ q1: qubit = (2 * q0: qubit)
    |_ q1: qubit

  |_ fn qbit_float$bar (q0: qubit) : qubit		// @qbit_float.ql:6:4
    |_ q1: qubit = (q0: qubit * 2)
    |_ q1: qubit

  |_ fn qbit_float$main () : qubit		// @qbit_float.ql:11:4
    |_ x: qubit = qbit_float$foo: qubit ()
    |_ y: qubit = qbit_float$bar: qubit ()

");

    // test!("tests/tabbed-comments-fn.ql", "");

    test!("tests/tabbed_comments.ql",
"|_ _foo			// @tabbed_comments.ql:1:1
|_ tabbed_comments			// @tabbed_comments.ql:1:1
");

    // test!("tests/tensors.ql", "");

    test!("tests/tensors2.ql",
"|_ tensors2			// @tensors2.ql:1:1
  |_ fn tensors2$foo (x: float64) : float64		// @tensors2.ql:1:4
    |_ x: float64

  |_ fn tensors2$bar (x: float64) : float64		// @tensors2.ql:5:4
    |_ x: float64

  |_ fn tensors2$main () : <bottom>		// @tensors2.ql:9:4
    |_ t1 = [[], []]
    |_ t2 = [[[], []], [[]]]

  |_ fn tensors2$foo2 () : float64		// @tensors2.ql:14:4
    |_ x: float64 = 42
    |_ e0: float64 = 2.718
    |_ e1: float64 = (e0: float64 * 2)
    |_ a: float64 = 0.707
    |_ t1 = []
    |_ t2: float64 = [x: float64]
    |_ t3: float64 = [t2: float64, t2: float64]
    |_ t4: float64 = [(e0: float64 * tensors2$bar: float64 (a: float64)), (-e1: float64 * tensors2$foo: float64 (a: float64))]
    |_ t5 = [[]]
    |_ t7: float64 = [[x: float64]]
    |_ t8: float64 = [[(e0: float64 * tensors2$bar: float64 (a: float64)), (-e1: float64 * tensors2$foo: float64 (a: float64))], [(e1: float64 * tensors2$foo: float64 (a: float64)), (e0: float64 * tensors2$bar: float64 (a: float64))]]
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

    test!("tests/test_if.ql",
"|_ test_if			// @test_if.ql:1:1
  |_ fn test_if$foo () : qubit		// @test_if.ql:1:4
    |_ x: qubit = 0q0_1
    |_ x: qubit

  |_ fn test_if$main () : float64		// @test_if.ql:6:4
    |_ choice: qubit = test_if$foo: qubit ()
    |_ (choice == 0)
      |_ True
        |_ x: float64 = 42
      |_ False
        |_ x: float64 = 2

    |_ (1 != 2)
      |_ True
        |_ x: float64 = 42
      |_ False
        |_ x: float64 = 32


");

    test!("tests/test_else_if.ql",
"|_ test_else_if			// @test_else_if.ql:1:1
  |_ fn test_else_if$pseudo_random () : float64		// @test_else_if.ql:1:4
    |_ 42

  |_ fn test_else_if$main () : float64		// @test_else_if.ql:5:4
    |_ choice: float64 = test_else_if$pseudo_random: float64 ()
    |_ (choice == 0)
      |_ True
        |_ x: float64 = 42
      |_ False
        |_ (choice == 1)
      |_ True
        |_ x: float64 = 41
      |_ False
        |_ (choice == 2)
      |_ True
        |_ x: float64 = 40
      |_ False
        |_ x: float64 = 0




");

    test!("tests/test_if_return_type.ql",
"|_ test_if_return_type			// @test_if_return_type.ql:1:1
  |_ fn test_if_return_type$unfair_toss (b: bit) : qubit		// @test_if_return_type.ql:1:4
    |_ b
      |_ True
        |_ 0q0_1
      |_ False
        |_ 0q1_0


  |_ fn test_if_return_type$main () : qubit		// @test_if_return_type.ql:9:4
    |_ choice: qubit = test_if_return_type$unfair_toss: qubit (1)

");

    // test!("tests/test_if_return_mismatch.ql", "");

    test!("tests/test_binary_expressions.ql",
"|_ test_binary_expressions			// @test_binary_expressions.ql:1:1
  |_ fn test_binary_expressions$main () : float64		// @test_binary_expressions.ql:1:4
    |_ x: float64 = (1 != 2)
    |_ y: float64 = (3 == 3)
    |_ b0: float64 = (1 < 2)
    |_ b1: float64 = (2 > 3)
    |_ b2: float64 = (1 <= 2)
    |_ b3: float64 = (4 >= 5)
    |_ a0: float64 = (x: float64 < y: float64)
    |_ a1: float64 = (x: float64 > y: float64)
    |_ a2: float64 = (x: float64 <= y: float64)
    |_ a3: float64 = (x: float64 >= y: float64)
    |_ w: float64 = (a0: float64 == a0: float64)
    |_ z: float64 = (a0: float64 != a1: float64)
    |_ mix0: float64 = (a0: float64 == 1)
    |_ mix1: float64 = (1 != a0: float64)
    |_ mix2: float64 = (a0: float64 < 1)
    |_ mix3: float64 = (1 > a0: float64)
    |_ mix4: float64 = (a0: float64 <= 1)
    |_ mix5: float64 = (1 >= w: float64)
    |_ (a0: float64 + (a1: float64 + (a2: float64 + a3: float64)))

");

    test!("tests/test_empty.ql",
"|_ test_empty			// @test_empty.ql:1:1
");

    // test!("tests/test_functions.ql.ql", "");

    // test!("tests/test_duplicate_functions.ql", "");

    // test!("tests/test_incomplete_fn.ql.ql", "");

    test!("tests/test_alias.ql",
"|_ complex_expr_lib			// @complex_expr_lib.ql:1:1
  |_ fn complex_expr_lib$bar () : qubit		// @complex_expr_lib.ql:2:4
    |_ 0q1_0

  |_ fn complex_expr_lib$sin (r: float64) : float64		// @complex_expr_lib.ql:6:4
    |_ (r: float64 / 180)

  |_ fn complex_expr_lib$cos (r: float64) : float64		// @complex_expr_lib.ql:10:4
    |_ (r: float64 / 90)

|_ test_alias			// @test_alias.ql:1:1
  |_ fn test_alias$foo (q0: qubit) : qubit		// @test_alias.ql:8:4
    |_ q0: qubit

  |_ fn test_alias$main () : float64		// @test_alias.ql:10:4
    |_ x: float64 = test_alias$S: float64 (1)
    |_ y: float64 = test_alias$C: float64 (0)
    |_ (x: float64 + y: float64)

  |_ fn test_alias$S (x0: float64) : float64		// @test_alias.ql:4:4
    |_ complex_expr_lib$sin: float64 (x0: float64)

  |_ fn test_alias$C (x0: float64) : float64		// @test_alias.ql:5:4
    |_ complex_expr_lib$cos: float64 (x0: float64)

");

    // test!("tests/test_alias_missing.ql.ql.ql", "");

    test!("tests/factorial.ql",
"|_ factorial			// @factorial.ql:1:1
  |_ fn factorial$factorial (n: float64) : float64		// @factorial.ql:1:4
    |_ (n <= 1)
      |_ True
        |_ 1
      |_ False
        |_ (n: float64 * factorial$factorial: float64 ((n: float64 - 1)))


");

    test!("tests/test_import.ql",
"|_ complex_expr_lib			// @complex_expr_lib.ql:1:1
  |_ fn complex_expr_lib$bar () : qubit		// @complex_expr_lib.ql:2:4
    |_ 0q1_0

  |_ fn complex_expr_lib$sin (r: float64) : float64		// @complex_expr_lib.ql:6:4
    |_ (r: float64 / 180)

  |_ fn complex_expr_lib$cos (r: float64) : float64		// @complex_expr_lib.ql:10:4
    |_ (r: float64 / 90)

|_ test_import			// @test_import.ql:1:1
  |_ fn test_import$main () : float64		// @test_import.ql:4:4
    |_ x: float64 = 2
    |_ complex_expr_lib$sin: float64 (x: float64)

");

    test!("examples/toss.ql",
"|_ math			// @math.ql:1:1
  |_ fn math$factorial (n: float64) : float64		// @math.ql:1:4
    |_ (n <= 1)
      |_ True
        |_ 1
      |_ False
        |_ (n: float64 * math$factorial: float64 ((n: float64 - 1)))


  |_ fn math$sin (x: float64) : float64		// @math.ql:11:4
    |_ cube: float64 = (x: float64 * (x: float64 * x: float64))
    |_ fact: float64 = math$factorial: float64 (3)
    |_ (x: float64 - (cube: float64 / fact: float64))

  |_ fn math$cos (x: float64) : float64		// @math.ql:17:4
    |_ sqre: float64 = (x: float64 * x: float64)
    |_ fact: float64 = math$factorial: float64 (2)
    |_ (1 - (sqre: float64 / fact: float64))

  |_ fn math$exp (x: float64) : float64		// @math.ql:23:4
    |_ e: float64 = 2.718
    |_ (e: float64 * x: float64)

|_ std			// @std.ql:1:1
  |_ fn std$U (theta: float64, phi: float64, lambda: float64, q0: qubit) : qubit		// @std.ql:5:4
    |_ e0: float64 = math$exp: float64 (((phi: float64 + lambda: float64) / 2))
    |_ e1: float64 = math$exp: float64 (((phi: float64 - lambda: float64) / 2))
    |_ a: float64 = (theta: float64 / 2)
    |_ transform: float64 = [[(e0: float64 * math$cos: float64 (a: float64)), (-e1: float64 * math$sin: float64 (a: float64))], [(e1: float64 * math$sin: float64 (a: float64)), (e0: float64 * math$cos: float64 (a: float64))]]
    |_ (transform: float64 * q0: qubit)

  |_ fn std$Hadamard (q: qubit) : qubit		// @std.ql:20:4
    |_ pi: float64 = 3.14
    |_ std$U: qubit ((pi: float64 / 2), 0, 0, q: qubit)

|_ toss			// @toss.ql:1:1
  |_ fn toss$toss () : qubit		// @toss.ql:5:4
    |_ zero_state: qubit = 0q0_1
    |_ superpositioned: qubit = toss$H: qubit (zero_state: qubit)
    |_ superpositioned: qubit

  |_ fn toss$main () : <bottom>		// @toss.ql:11:4
    |_ choice: qubit = toss$toss: qubit ()
    |_ (choice == 0)


  |_ fn toss$H (x0: qubit) : qubit		// @toss.ql:3:4
    |_ std$Hadamard: qubit (x0: qubit)

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
