use super::selectors::OTLPTemporalitySelector;
use configs::{Configs, DynamicConfigs};
use opentelemetry::{global, runtime, sdk::Resource, KeyValue};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use std::{error::Error, time::Duration};
use tonic::metadata::MetadataMap;
use tracing::{debug, error};

pub fn setup<T>(cfg: &Configs<T>) -> Result<(), Box<dyn Error>>
where
    T: DynamicConfigs,
{
    if !cfg.metric.enable {
        debug!("metrics::setup skipping metrics export setup");
        return Ok(());
    }

    debug!("metrics::setup configure metrics...");

    let mut map = MetadataMap::with_capacity(3);
    match cfg.metric.key.parse() {
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
        .metrics(runtime::Tokio)
        .with_temporality_selector(OTLPTemporalitySelector::default())
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&cfg.metric.host)
                .with_timeout(Duration::from_secs(cfg.metric.export_timeout))
                .with_protocol(Protocol::Grpc)
                .with_metadata(map),
        )
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", cfg.app.name.clone()),
            KeyValue::new("service.type", cfg.metric.service_type.clone()),
            KeyValue::new("environment", format!("{}", cfg.app.env)),
            KeyValue::new("library.language", "rust"),
        ]))
        .with_period(Duration::from_secs(cfg.metric.export_interval))
        .with_timeout(Duration::from_secs(cfg.metric.export_timeout))
        .build()?;

    global::set_meter_provider(provider);

    debug!("metrics::setup installed");

    Ok(())
}
