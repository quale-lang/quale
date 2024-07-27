//! Configuration file for compilation session in qcc.
use crate::analyzer::config::*;
use crate::optimizer::config::*;

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) dump_ast: bool,
    pub(crate) dump_ast_only: bool,
    pub(crate) dump_qasm: bool,
    pub analyzer: AnalyzerConfig,
    pub optimizer: OptConfig,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            dump_ast: false,
            dump_ast_only: false,
            dump_qasm: false,
            optimizer: OptConfig::new(),
            analyzer: AnalyzerConfig::new(),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.analyzer, self.optimizer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_config() {
        let config = Config::new();
        assert!(!config.dump_ast);
        assert!(!config.dump_ast_only);
        assert!(!config.dump_qasm);
        assert_eq!(
            format!("{}", config.analyzer),
            "
Analyzer Configuration
-----------------------
: false"
        );

        assert_eq!(
            format!("{}", config.optimizer),
            "
Optimizer Configuration
-----------------------
Stage: O0"
        );

        assert_eq!(
            format!("{}", config),
            "
Analyzer Configuration
-----------------------
: false

Optimizer Configuration
-----------------------
Stage: O0"
        );
    }
}
