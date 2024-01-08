use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub enum TraceExporterKind {
    #[default]
    Stdout,
    OtlpGrpc,
}

impl FromStr for TraceExporterKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "otlp" | "otlp-grpc" | "grpc" => Ok(TraceExporterKind::OtlpGrpc),
            _ => Ok(TraceExporterKind::Stdout),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceConfigs {
    ///Default: false
    pub enable: bool,
    ///Default: ExportKind::Stdout
    pub exporter: TraceExporterKind,
    ///Default: localhost
    pub host: String,
    ///Default: api-key
    pub header_access_key: String,
    ///Default: key
    pub access_key: String,
    pub service_type: String,
    ///Default: 30s
    pub export_timeout: u64,
    ///Default: 60s
    pub export_interval: u64,
    ///Default: 0.8
    pub export_rate_base: f64,
}

impl Default for TraceConfigs {
    fn default() -> Self {
        Self {
            enable: false,
            host: Default::default(),
            exporter: Default::default(),
            header_access_key: Default::default(),
            access_key: Default::default(),
            service_type: Default::default(),
            export_timeout: 30,
            export_interval: 60,
            export_rate_base: 0.8,
        }
    }
}
