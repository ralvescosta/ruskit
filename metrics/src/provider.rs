use crate::errors::MetricsError;
use configs::{Configs, DynamicConfigs, MetricExporterKind};
use prometheus::Registry;
use std::sync::Arc;
use tracing::{debug, error};

#[cfg(any(feature = "otlp", feature = "prometheus", feature = "stdout"))]
use crate::exporters;

#[cfg(any(feature = "otlp", feature = "prometheus", feature = "stdout"))]
use opentelemetry::global;

pub fn init<T>(cfg: &Configs<T>) -> Result<Option<Arc<Registry>>, MetricsError>
where
    T: DynamicConfigs,
{
    if !cfg.metric.enable {
        debug!("metrics::init skipping metrics export setup");
        return Ok(None);
    }

    debug!("metrics::init configure metrics...");

    match cfg.metric.exporter {
        MetricExporterKind::OtlpGrpc => {
            #[cfg(feature = "otlp")]
            {
                global::set_meter_provider(exporters::otlp::install(cfg)?);
                debug!("metrics::init otlp installed");
                Ok(None)
            }

            #[cfg(not(feature = "otlp"))]
            {
                error!("otlp metrics required to configure features = [otlp]");
                Err(MetricsError::InvalidFeaturesError)
            }
        }
        MetricExporterKind::Prometheus => {
            #[cfg(feature = "prometheus")]
            {
                let (provider, r) = exporters::prom::install(cfg)?;
                global::set_meter_provider(provider);
                debug!("metrics::init prometheus installed");
                Ok(Some(r))
            }

            #[cfg(not(feature = "prometheus"))]
            {
                error!("prometheus metrics required to configure features = [prometheus]");
                Err(MetricsError::InvalidFeaturesError)
            }
        }
        MetricExporterKind::Stdout => {
            #[cfg(feature = "stdout")]
            {
                global::set_meter_provider(exporters::stdout::install()?);
                debug!("metrics::init stdout installed");
                Ok(None)
            }

            #[cfg(not(feature = "stdout"))]
            {
                error!("stdout metrics required to configure features = [stdout]");
                Err(MetricsError::InvalidFeaturesError)
            }
        }
    }
}
