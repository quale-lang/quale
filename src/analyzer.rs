//! Static analyzer for qcc
use crate::ast::Qast;

#[derive(Debug)]
pub struct AnalyzerConfig {
    pub(crate) status: bool,
    pub src: String,
}

impl AnalyzerConfig {
    pub(crate) fn new() -> Self {
        AnalyzerConfig {
            status: false,
            src: "".into(),
        }
    }

    pub(crate) fn analyze(&self, ast: &Qast) {
        println!("Analyzing ...{}", self.src);
    }
}

impl std::fmt::Display for AnalyzerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Analyzer Configuration
-----------------------
{}: {}",
            self.src, self.status
        )
    }
}

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
