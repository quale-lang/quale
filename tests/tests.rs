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
        let args = vec![path];
        let parser: Parser = Default::default();
        if let Some(config) = parser.parse_cmdline(args)? {
            let ast = parser.parse(&config.analyzer.src)?;
            println!("{ast}");
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
        let args = vec![path, "-O2".into(), "--analyze".into()];
        let parser: Parser = Default::default();
        let _opts = parser.parse_cmdline(args)?;
    }

    Ok(())
}

#[test]
fn non_existing_src() -> Result<(), Box<dyn std::error::Error>> {
    let path = String::from("./tests/test-non-existent.ql");
    let args = vec![path.clone(), "--analyze".into()];
    let parser: Parser = Default::default();
    let _parsed = parser.parse_cmdline(args).unwrap_err();
    // FIXME:
    /* assert_eq!(parsed, Err(format!("{path} doesn't exist"))); */
    /* assert_eq!(parse_cmdline(args)?, */
    /*     Err(format!("{path} doesn't exist"))?); */
    /* if let Some(config) = parse_cmdline(args)? { */
    /*     let _ast = parser.parse(&config.analyzer.src)?; */
    /* } */
    Ok(())
}
