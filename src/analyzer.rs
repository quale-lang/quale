//! Static analyzer for qcc

pub(crate) struct AnalyzerConfig {
    pub(crate) status: bool,
    pub(crate) src: String,
}

impl AnalyzerConfig {
    pub(crate) fn new() -> Self {
        AnalyzerConfig {
            status: false,
            src: "".to_string(),
        }
    }
}
