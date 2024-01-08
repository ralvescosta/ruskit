use crate::errors::MetricsError;
use opentelemetry_sdk::{
    metrics::{MeterProvider, PeriodicReader},
    runtime,
};

pub fn install() -> Result<MeterProvider, MetricsError> {
    let exporter = opentelemetry_stdout::MetricsExporter::default();

    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();

    Ok(MeterProvider::builder().with_reader(reader).build())
}
