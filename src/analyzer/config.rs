//! Configuration for Quale Analyzer
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
