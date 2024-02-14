//! Static analyzer for qcc

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
}
