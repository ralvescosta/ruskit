use crate::errors::MetricsError;
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    runtime,
};

pub fn install() -> Result<SdkMeterProvider, MetricsError> {
    let exporter = opentelemetry_stdout::MetricsExporter::default();

    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();

    Ok(MeterProviderBuilder::default().with_reader(reader).build())
}
