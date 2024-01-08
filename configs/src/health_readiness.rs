#[derive(Debug, Clone)]
pub struct HealthReadinessConfigs {
    ///Default: 8888
    pub port: u64,
    ///Default: false
    pub enable: bool,
}

impl Default for HealthReadinessConfigs {
    fn default() -> Self {
        Self {
            port: 8888,
            enable: false,
        }
    }
}

impl HealthReadinessConfigs {
    pub fn health_readiness_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}
