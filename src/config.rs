//! Configuration file for compilation session in qcc.
use crate::analyzer::config::*;
use crate::optimizer::config::*;

#[derive(Debug)]
pub struct Config {
    pub(crate) dump_ast: bool,
    pub(crate) dump_qasm: bool,
    pub analyzer: AnalyzerConfig,
    pub optimizer: OptConfig,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            dump_ast: false,
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
