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
        let mut parser = Parser::new(args)?.unwrap();

        let config = parser.get_config();
        let qast = parser.parse(&config.analyzer.src)?;
        println!("{qast}");
        config.analyzer.analyze(&qast)?;

        Ok(())
    }
}
