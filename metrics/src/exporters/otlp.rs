use super::selectors::OTLPTemporalitySelector;
use crate::errors::MetricsError;
use configs::{Configs, DynamicConfigs};
use opentelemetry::KeyValue;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, runtime, Resource};
use std::time::Duration;
use tonic::metadata::{Ascii, MetadataKey, MetadataMap};
use tracing::error;

pub fn install<T>(cfg: &Configs<T>) -> Result<SdkMeterProvider, MetricsError>
where
    T: DynamicConfigs,
{
    let key: MetadataKey<Ascii> = match cfg.trace.header_access_key.clone().parse() {
        Ok(key) => key,
        Err(_) => {
            error!("failure to convert cfg.trace.header_key");
            MetadataKey::<Ascii>::from_bytes("api-key".as_bytes()).unwrap()
        }
    };

    let value = match cfg.trace.access_key.parse() {
        Ok(value) => Ok(value),
        Err(_) => {
            error!("failure to convert cfg.trace.header_value");
            Err(MetricsError::ConversionError)
        }
    }?;

    let mut map = MetadataMap::with_capacity(2);
    map.insert(key, value);

    let provider = match opentelemetry_otlp::new_pipeline()
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
        .build()
    {
        Ok(p) => Ok(p),
        Err(err) => {
            error!(
                error = err.to_string(),
                "failure to create exporter provider"
            );
            Err(MetricsError::ExporterProviderError)
        }
    }?;

    Ok(provider)
}
