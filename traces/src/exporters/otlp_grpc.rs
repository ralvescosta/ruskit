use crate::errors::TracesError;
use configs::{Configs, DynamicConfigs};
use opentelemetry::{global, propagation::TextMapCompositePropagator};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    propagation::{BaggagePropagator, TraceContextPropagator},
    runtime,
    trace::Config,
};
use std::time::Duration;
use tonic::metadata::{Ascii, MetadataKey, MetadataMap};
use tracing::{debug, error};

pub fn install<T>(cfg: &Configs<T>, trace_configs: Config) -> Result<(), TracesError>
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
            Err(TracesError::ConversionError)
        }
    }?;

    let mut map = MetadataMap::with_capacity(2);
    map.insert(key, value);

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&cfg.trace.host)
        .with_protocol(Protocol::Grpc)
        .with_timeout(Duration::from_secs(cfg.trace.export_timeout))
        .with_metadata(map);

    match opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(trace_configs)
        .with_exporter(exporter)
        .install_batch(runtime::Tokio)
    {
        Err(err) => {
            error!(error = err.to_string(), "failure to install otlp tracing");
            Err(TracesError::ExporterProviderError)
        }
        _ => {
            global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
                Box::new(TraceContextPropagator::new()),
                Box::new(BaggagePropagator::new()),
            ]));

            debug!("traces::install otlp tracer installed");

            Ok(())
        }
    }
}
