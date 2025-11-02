use rand::Rng;
use std::time::Duration;

/// Election timeout configuration
#[derive(Debug, Clone)]
pub struct ElectionTimeoutConfig {
    pub min_ms: u64,
    pub max_ms: u64,
}

impl Default for ElectionTimeoutConfig {
    fn default() -> Self {
        Self {
            min_ms: 150,
            max_ms: 300,
        }
    }
}

impl ElectionTimeoutConfig {
    /// Generate a random election timeout
    pub fn generate_timeout(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let timeout_ms = rng.gen_range(self.min_ms..=self.max_ms);
        Duration::from_millis(timeout_ms)
    }
}

/// Heartbeat interval (should be << election timeout)
pub const HEARTBEAT_INTERVAL_MS: u64 = 50;

pub fn heartbeat_interval() -> Duration {
    Duration::from_millis(HEARTBEAT_INTERVAL_MS)
}

