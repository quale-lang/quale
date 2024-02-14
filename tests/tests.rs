
use qcc::parser::{parse_cmdline, parse_src};

#[test]
fn compile() -> Result<(), Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir("./tests")?;

    for p in paths {
        let path = p.unwrap().path().into_os_string().into_string().unwrap();
        if !path.ends_with(".ql") {
            continue;
        }
        let args = vec![path];
        if let Some(config) = parse_cmdline(args)? {
            let _ast = parse_src(&config.analyzer.src)?;
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
        let _opts = parse_cmdline(args)?;
    }

    Ok(())
}

#[test]
fn non_existing_src() -> Result<(), Box<dyn std::error::Error>> {
    let path = String::from("./tests/test-non-existent.ql");
    let args = vec![path.clone(), "--analyze".into()];
    let _parsed = parse_cmdline(args).unwrap_err();
    // FIXME:
    /* assert_eq!(parsed, Err(format!("{path} doesn't exist"))); */
    /* assert_eq!(parse_cmdline(args)?, */
    /*     Err(format!("{path} doesn't exist"))?); */
    /* if let Some(config) = parse_cmdline(args)? { */
    /*     let _ast = parse_src(&config.analyzer.src)?; */
    /* } */
    Ok(())
}
