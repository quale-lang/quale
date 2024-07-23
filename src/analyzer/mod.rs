//! Static analyzer for qcc
pub mod config;

#[cfg(test)]
mod tests {
    use crate::error::{QccErrorKind, Result};
    use crate::parser::Parser;
    use crate::{assert_eq_all, assert_eq_any};

    #[test]
    fn check_analyzer() -> Result<()> {
        let path = "tests/test1.ql".into();
        let args = vec![path];

        let parser = Parser::new(args);
        assert!(parser.is_ok());
        let parser = parser.unwrap();
        assert!(parser.is_some());
        let mut parser = parser.unwrap();

        let config = parser.get_config();
        let qast = parser.parse(&config.analyzer.src);
        assert!(qast.is_ok());
        let qast = qast.unwrap();
        assert!(config.analyzer.analyze(&qast).is_ok());

        Ok(())
    }
}
