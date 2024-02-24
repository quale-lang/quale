//! Optimizer for qcc.

#[derive(Debug)]
pub struct OptConfig {
    pub level: u8, // 0, 1, 2
}

impl OptConfig {
    pub fn new(level: u8) -> Self {
        OptConfig { level }
    }
}

impl Default for OptConfig {
    fn default() -> Self {
        Self::new(0)
    }
}

impl std::fmt::Display for OptConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Optimizer Configuration
-----------------------
Stage: O{}",
            self.level
        )
    }
}
