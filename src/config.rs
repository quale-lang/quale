//! Configuration file for compilation session in qcc.
use crate::analyzer::config::*;
use crate::optimizer::config::*;

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) debug: bool,
    pub(crate) print_ast: bool,
    pub(crate) print_ast_only: bool,
    pub(crate) print_qasm: bool,
    pub(crate) version: &'static str,
    pub analyzer: AnalyzerConfig,
    pub optimizer: OptConfig,
}

impl Config {
    #[inline]
    fn version() -> &'static str {
        concat!(env!("CARGO_PKG_VERSION"), "+", env!("GIT_HASH"))
    }

    pub(crate) fn new() -> Self {
        Self {
            debug: false,
            print_ast: false,
            print_ast_only: false,
            print_qasm: false,
            version: Self::version(),
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
        assert!(!config.print_ast);
        assert!(!config.print_ast_only);
        assert!(!config.print_qasm);
        assert_eq!(
            format!("{}", config.analyzer),
            "
Analyzer Configuration
----------------------
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
----------------------
: false

Optimizer Configuration
-----------------------
Stage: O0"
        );
    }
}
