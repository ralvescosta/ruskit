use configs::{Configs, DynamicConfigs};
use opentelemetry::KeyValue;
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, SdkMeterProvider},
    Resource,
};
use prometheus::Registry;
use std::sync::Arc;
use tracing::error;

use crate::errors::MetricsError;

pub fn install<T>(cfg: &Configs<T>) -> Result<(SdkMeterProvider, Arc<Registry>), MetricsError>
where
    T: DynamicConfigs,
{
    let registry = Registry::new();

    let exporter = match opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
    {
        Ok(e) => Ok(e),
        Err(err) => {
            error!(error = err.to_string(), "failure to create prom exporter");
            Err(MetricsError::ExporterProviderError)
        }
    }?;

    let provider = MeterProviderBuilder::default()
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", cfg.app.name.clone()),
            KeyValue::new("service.type", cfg.metric.service_type.clone()),
            KeyValue::new("environment", format!("{}", cfg.app.env)),
            KeyValue::new("library.language", "rust"),
        ]))
        .with_reader(exporter)
        .build();

    Ok((provider, Arc::new(registry)))
}
