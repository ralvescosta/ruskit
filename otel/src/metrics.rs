use log::debug;
use opentelemetry::{sdk::metrics::selectors, util::tokio_interval_stream};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use std::time::Duration;
use tonic::metadata::MetadataMap;

use env::Config;

pub fn setup(cfg: &Config) -> Result<(), ()> {
    debug!("telemetry::install metrics...");

    let mut map = MetadataMap::with_capacity(3);
    map.insert("api-key", cfg.otlp_key.parse().unwrap());

    let meter = opentelemetry_otlp::new_pipeline()
        .metrics(tokio::spawn, tokio_interval_stream)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(cfg.otlp_host)
                .with_timeout(Duration::from_secs(5))
                .with_protocol(Protocol::Grpc)
                .with_metadata(map),
        )
        .with_period(Duration::from_secs(5))
        .with_timeout(Duration::from_secs(10))
        .with_aggregator_selector(selectors::simple::Selector::Exact)
        .build()
        .map_err(|_| ())?;

    meter.provider();

    debug!("telemetry::metrics installed");

    Ok(())
}
