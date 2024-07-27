//! Configuration for Quale Analyzer
use crate::ast::Qast;
use crate::error::Result;

#[derive(Debug, Clone)]
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
        for _fn in ast {}
        Ok(())
    }
}

impl std::fmt::Display for AnalyzerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Analyzer Configuration
----------------------
{}: {}",
            self.src, self.status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_new_analyzer_config() -> Result<()> {
        let analyzer_config = AnalyzerConfig::new();
        assert_eq!(
            format!("{}", analyzer_config),
            "\nAnalyzer Configuration\n----------------------\n: false"
        );
        Ok(())
    }

    #[test]
    fn check_analyzer_config() -> Result<()> {
        let analyzer_config = AnalyzerConfig {
            src: "tmp".into(),
            status: true,
        };
        assert_eq!(
            format!("{}", analyzer_config),
            "
Analyzer Configuration
----------------------
tmp: true"
        );
        Ok(())
    }
}
