use qcc::assert_eq_any;
use qcc::codegen::{qasm, Translator};
use qcc::error::QccErrorKind;
use qcc::inference::infer;
use qcc::parser::Parser;

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

            Err(err) => assert_eq_any!(err, [QccErrorKind::LexerError, QccErrorKind::ParseError]),
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
        Err(err) => assert_eq!(err, QccErrorKind::NoFile.into()),
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
            Err(err) => assert_eq_any!(err, [QccErrorKind::LexerError, QccErrorKind::ParseError]),
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
