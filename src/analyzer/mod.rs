//! Static analyzer for qcc
pub mod config;

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use crate::parser::Parser;

    #[test]
    fn check_analyzer() -> Result<()> {
        let path = "tests/test1.ql".into();
        let args = vec![path];
        let parser: Parser = Default::default();
        if let Some(config) = parser.parse_cmdline(args)? {
            let ast = parser.parse(&config.analyzer.src)?;
            println!("{ast}");
            config.analyzer.analyze(&ast);
        }
        Ok(())
    }
}
