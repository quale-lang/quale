//! Configuration file for compilation session in qcc.
use crate::analyzer::AnalyzerConfig;
use crate::opt::OptConfig;

#[derive(Debug)]
pub struct Config {
    pub optimizer: OptConfig,
    pub analyzer: AnalyzerConfig,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            optimizer: OptConfig::new(),
            analyzer: AnalyzerConfig::new(),
        }
    }
}
