//! Optimizer for qcc.

#[derive(Debug)]
pub struct OptConfig {
    pub level: u8, // 0, 1, 2
}

impl OptConfig {
    pub fn new() -> Self {
        OptConfig { level: 0 }
    }
}

impl Default for OptConfig {
    fn default() -> Self {
        Self::new()
    }
}
