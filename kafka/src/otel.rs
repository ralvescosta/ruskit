use std::str::FromStr;

use opentelemetry::{
    global::BoxedTracer,
    trace::{
        Span, SpanContext, SpanId, SpanKind, TraceContextExt, TraceFlags, TraceId, TraceState,
        Tracer,
    },
    Context,
};
use rdkafka::message::{BorrowedHeaders, Header, Headers, OwnedHeaders};
use tracing::error;

const SUPPORTED_VERSION: u8 = 0;
const TRACEPARENT_HEADER: &str = "traceparent";
const TRACESTATE_HEADER: &str = "tracestate";
const MAX_VERSION: u8 = 254;

pub fn new_ctx(topic: &str, msg_type: &str, tracer: &BoxedTracer) -> Context {
    let span = tracer
        .span_builder(format!("{}::{}", topic, msg_type))
        .with_kind(SpanKind::Consumer)
        .start(tracer);

    Context::current_with_span(span)
}

pub fn inject_context(
    ctx: &Context,
    topic: &str,
    msg_type: &str,
    tracer: &BoxedTracer,
    kafka_headers: OwnedHeaders,
) -> OwnedHeaders {
    let span = ctx.span();
    let mut span_context = span.span_context().to_owned();

    if !span_context.is_valid() {
        let new_span = tracer
            .span_builder(format!("{}::{}", topic, msg_type))
            .with_kind(SpanKind::Consumer)
            .start(tracer);

        span_context = new_span.span_context().to_owned();
    }

    let header_value = format!(
        "{:02x}-{}-{}-{:02x}",
        SUPPORTED_VERSION,
        span_context.trace_id(),
        span_context.span_id(),
        span_context.trace_flags() & TraceFlags::SAMPLED
    );

    kafka_headers
        .insert(Header {
            key: TRACEPARENT_HEADER,
            value: Some(&header_value),
        })
        .insert(Header {
            key: TRACESTATE_HEADER,
            value: Some(&span_context.trace_state().header()),
        })
}

pub fn extract_context(kafka_headers: &BorrowedHeaders) -> Result<Context, ()> {
    let Some((header_value, stats)) = extract_trace_from_header(kafka_headers) else {
        return Err(());
    };

    let parts = header_value.split_terminator('-').collect::<Vec<&str>>();
    // Ensure parts are not out of range.
    if parts.len() < 4 {
        return Err(());
    }

    // Ensure version is within range, for version 0 there must be 4 parts.
    let version = u8::from_str_radix(parts[0], 16).map_err(|_| ())?;
    if version > MAX_VERSION || version == 0 && parts.len() != 4 {
        return Err(());
    }

    // Ensure trace id is lowercase
    if parts[1].chars().any(|c| c.is_ascii_uppercase()) {
        return Err(());
    }

    // Parse trace id section
    let trace_id = TraceId::from_hex(parts[1]).map_err(|_| ())?;

    // Ensure span id is lowercase
    if parts[2].chars().any(|c| c.is_ascii_uppercase()) {
        return Err(());
    }

    // Parse span id section
    let span_id = SpanId::from_hex(parts[2]).map_err(|_| ())?;

    // Parse trace flags section
    let opts = u8::from_str_radix(parts[3], 16).map_err(|_| ())?;

    // Ensure opts are valid for version 0
    if version == 0 && opts > 2 {
        return Err(());
    }

    // Build trace flags clearing all flags other than the trace-context
    // supported sampling bit.
    let trace_flags = TraceFlags::new(opts) & TraceFlags::SAMPLED;

    let trace_state: TraceState =
        TraceState::from_str(&stats).unwrap_or_else(|_| TraceState::default());

    // create context
    let span_context = SpanContext::new(trace_id, span_id, trace_flags, true, trace_state);

    Ok(Context::new().with_remote_span_context(span_context))
}

fn extract_trace_from_header<'e>(kafka_headers: &'e BorrowedHeaders) -> Option<(&'e str, &'e str)> {
    let mut trace_parent = "";
    let mut trace_state = "";
    let mut founded = 0;

    for h in kafka_headers.iter() {
        if h.key.eq(TRACEPARENT_HEADER) {
            if h.value.is_none() {
                continue;
            }

            trace_parent = match std::str::from_utf8(h.value.unwrap()) {
                Ok(v) => v,
                Err(err) => {
                    error!(error = err.to_string(), "key conversion to utf8 error");
                    continue;
                }
            };

            founded += 1;
        }

        if h.key.eq(TRACESTATE_HEADER) {
            if h.value.is_none() {
                continue;
            }

            trace_state = match std::str::from_utf8(h.value.unwrap()) {
                Ok(v) => v,
                Err(err) => {
                    error!(error = err.to_string(), "key conversion to utf8 error");
                    continue;
                }
            };

            founded += 1;
        }

        if founded == 2 {
            break;
        }
    }

    if trace_parent.is_empty() || trace_state.is_empty() {
        return None;
    }

    Some((trace_parent, trace_state))
}
