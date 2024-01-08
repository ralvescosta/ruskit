use opentelemetry_sdk::metrics::{data::Temporality, reader::TemporalitySelector, InstrumentKind};

/// - Cumulative temporality means that successive data points repeat the starting timestamp. For example, from start time T0, cumulative data points cover time ranges (T0, T1], (T0, T2], (T0, T3], and so on.
///
/// - Delta temporality means that successive data points advance the starting timestamp. For example, from start time T0, delta data points cover time ranges (T0, T1], (T1, T2], (T2, T3], and so on.
///
/// - The use of cumulative temporality for monotonic sums is common, exemplified by Prometheus. Systems based in cumulative monotonic sums are naturally simpler, in terms of the cost of adding reliability. When collection fails intermittently, gaps in the data are naturally averaged from cumulative measurements. Cumulative data requires the sender to remember all previous measurements, an “up-front” memory cost proportional to cardinality.
///
/// - The use of delta temporality for metric sums is also common, exemplified by Statsd. There is a connection between OpenTelemetry tracing, in which a Span event commonly is translated into two metric events (a 1-count and a timing measurement). Delta temporality enables sampling and supports shifting the cost of cardinality outside of the process.
#[derive(Clone, Default)]
pub struct OTLPTemporalitySelector;

impl TemporalitySelector for OTLPTemporalitySelector {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        match kind {
            InstrumentKind::UpDownCounter | InstrumentKind::ObservableUpDownCounter => {
                Temporality::Cumulative
            }
            _ => Temporality::Delta,
        }
    }
}
