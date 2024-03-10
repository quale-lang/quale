//! Configuration for Quale Analyzer
use crate::ast::Qast;
use crate::error::Result;

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

    pub fn analyze(&self, ast: &Qast) -> Result<()> {
        for _fn in ast.iter() {}
        Ok(())
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
