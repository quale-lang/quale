//! Configuration file for compilation session in qcc.
use crate::analyzer::AnalyzerConfig;
use crate::opt::OptConfig;

pub(crate) struct Config {
    pub(crate) optimizer: OptConfig,
    pub(crate) analyzer: AnalyzerConfig,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            optimizer: OptConfig::new(),
            analyzer: AnalyzerConfig::new(),
        }
    }
}
