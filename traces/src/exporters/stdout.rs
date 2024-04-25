use crate::errors::TracesError;
use opentelemetry::{global, propagation::TextMapCompositePropagator};
use opentelemetry_sdk::{
    propagation::{BaggagePropagator, TraceContextPropagator},
    trace::{Config, TracerProvider},
};
use tracing::debug;

pub fn install(trace_configs: Config) -> Result<(), TracesError> {
    let exporter = opentelemetry_stdout::SpanExporter::default();

    let provider = TracerProvider::builder()
        .with_config(trace_configs)
        .with_simple_exporter(exporter)
        .build();

    global::set_tracer_provider(provider);

    global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]));

    debug!("traces::install stdout tracer installed");

    Ok(())
}
