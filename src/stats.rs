use std::time::Duration;

/// Statistics about the hunt operation
#[derive(Debug, Clone)]
pub struct HuntStats {
    pub files_total: usize,
    pub keys_total: usize,
    pub unused_keys_count: usize,
    pub duration: Duration,
}

impl HuntStats {
    /// Get formatted duration
    pub fn formatted_duration(&self) -> String {
        let millis = self.duration.as_millis();
        if millis < 1000 {
            format!("{}ms", millis)
        } else {
            format!("{:.2}s", millis as f64 / 1000.0)
        }
    }
}
