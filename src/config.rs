//! Configuration file for compilation session in qcc.
use crate::analyzer::AnalyzerConfig;
use crate::opt::OptConfig;

#[derive(Debug)]
pub struct Config {
    pub analyzer: AnalyzerConfig,
    pub optimizer: OptConfig,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            optimizer: Default::default(),
            analyzer: AnalyzerConfig::new(),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.analyzer, self.optimizer)
    }
}
