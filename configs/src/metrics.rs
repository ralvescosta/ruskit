use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub enum MetricExporterKind {
    #[default]
    Stdout,
    OtlpGrpc,
    Prometheus,
}

impl FromStr for MetricExporterKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "otlp" | "otlp-grpc" | "grpc" => Ok(MetricExporterKind::OtlpGrpc),
            "prom" | "prometheus" => Ok(MetricExporterKind::Prometheus),
            _ => Ok(MetricExporterKind::Stdout),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricConfigs {
    ///Default: false
    pub enable: bool,
    ///Default: ExportKind::Stdout
    pub exporter: MetricExporterKind,
    ///Only used with OTLP
    ///
    ///Default: localhost
    pub host: String,
    ///Only used with OTLP
    ///
    ///Default: api-key
    pub header_access_key: String,
    ///Only used with OTLP
    ///
    ///Default: key
    pub access_key: String,
    pub service_type: String,
    ///Only used with OTLP
    ///
    ///Default: 30s
    pub export_timeout: u64,
    ///Only used with OTLP
    ///
    ///Default: 60s
    pub export_interval: u64,
    ///Only used with OTLP
    ///
    ///Default: 0.8
    pub export_rate_base: f64,
}

impl Default for MetricConfigs {
    fn default() -> Self {
        Self {
            enable: false,
            exporter: Default::default(),
            host: Default::default(),
            header_access_key: Default::default(),
            access_key: Default::default(),
            service_type: Default::default(),
            export_timeout: 30,
            export_interval: 60,
            export_rate_base: 0.8,
        }
    }
}
