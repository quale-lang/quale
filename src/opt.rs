//! Optimizer for qcc.

pub(crate) struct OptConfig {
    pub(crate) level: u8, // 0, 1, 2
}

impl OptConfig {
    pub(crate) fn new() -> Self {
        OptConfig { level: 0 }
    }
}
