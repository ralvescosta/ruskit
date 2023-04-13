use super::selectors::OTLPTemporalitySelector;
use configs::{Configs, DynamicConfigs};
use opentelemetry::{
    global, runtime,
    sdk::{metrics::selectors, Resource},
    KeyValue,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use std::{error::Error, time::Duration};
use tonic::metadata::MetadataMap;
use tracing::{debug, error};

pub fn setup<T>(cfg: &Configs<T>) -> Result<(), Box<dyn Error>>
where
    T: DynamicConfigs,
{
    if !cfg.otlp.enable_metrics {
        debug!("metrics::setup skipping metrics export setup");
        return Ok(());
    }

    debug!("metrics::setup configure metrics...");

    let mut map = MetadataMap::with_capacity(3);
    match cfg.otlp.key.parse() {
        Ok(p) => {
            map.insert("api-key", p);
            Ok(())
        }
        Err(e) => {
            error!(error = e.to_string(), "err mapping otlp key from config");
            Err(e)
        }
    }?;

    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::inexpensive(),
            OTLPTemporalitySelector::default(),
            runtime::Tokio,
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&cfg.otlp.host)
                .with_timeout(Duration::from_secs(cfg.otlp.export_timeout))
                .with_protocol(Protocol::Grpc)
                .with_metadata(map),
        )
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", cfg.app.name.clone()),
            KeyValue::new("service.type", cfg.otlp.service_type.clone()),
            KeyValue::new("environment", format!("{}", cfg.app.env)),
            KeyValue::new("library.language", "rust"),
        ]))
        .with_period(Duration::from_secs(cfg.otlp.metrics_export_interval))
        .with_timeout(Duration::from_secs(cfg.otlp.export_timeout))
        .build()?;

    global::set_meter_provider(provider);

    debug!("metrics::setup installed");

    Ok(())
}
