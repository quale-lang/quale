//! Configuration for Quale optimizer.

#[derive(Debug)]
pub struct OptConfig {
    pub asm: String,
    pub level: u8, // 0, 1, 2
}

impl OptConfig {
    pub fn new() -> Self {
        OptConfig {
            asm: "".into(),
            level: 0,
        }
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
