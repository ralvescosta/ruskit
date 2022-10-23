use opentelemetry::{
    global,
    global::{BoxedSpan, BoxedTracer},
    trace::{
        Span, SpanContext, SpanId, SpanKind, TraceContextExt, TraceFlags, TraceId, TraceState,
        Tracer,
    },
    Context,
};
use std::borrow::Cow;

const TRACE_VERSION: u8 = 0;

pub struct Traceparent {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub trace_flags: TraceFlags,
}

///traceparent is compos from {trace-version}-{trace-id}-{parent-id}-{trace-flags}
impl Traceparent {
    pub fn from_string(traceparent: &str) -> Traceparent {
        if traceparent.is_empty() {
            return Traceparent::new_empty();
        }

        let splitted: Vec<&str> = traceparent.split("-").collect();

        if splitted.len() <= 3 {
            return Traceparent::new_empty();
        }

        let trace_id = TraceId::from_hex(&splitted[1].to_string()).unwrap();
        let span_id = SpanId::from_hex(&splitted[2].to_string()).unwrap();
        let trace_flags = TraceFlags::new(splitted[3].as_bytes()[0]);

        Traceparent {
            trace_id,
            span_id,
            trace_flags,
        }
    }

    pub fn string_from_ctx(ctx: &Context) -> String {
        let trace_id = ctx.get::<TraceId>().unwrap();
        let parent_id = ctx.get::<SpanId>().unwrap();
        let trace_flags = ctx.get::<TraceFlags>().unwrap();

        format!(
            "{:02x}-{:032x}-{:016x}-{:02x}",
            TRACE_VERSION, trace_id, parent_id, trace_flags
        )
    }

    fn new_empty() -> Traceparent {
        let tracer = global::tracer("empty");

        let span = tracer
            .span_builder("empty")
            .with_kind(SpanKind::Consumer)
            .start(&tracer);

        let span_ctx = span.span_context();

        Traceparent {
            trace_id: span_ctx.clone().trace_id(),
            span_id: span_ctx.clone().span_id(),
            trace_flags: span_ctx.clone().trace_flags(),
        }
    }
}

pub fn get_span(tracer: &BoxedTracer, traceparent: &str, span_name: &str) -> (Context, BoxedSpan) {
    let traceparent = Traceparent::from_string(traceparent);

    let ctx = Context::new().with_remote_span_context(SpanContext::new(
        traceparent.trace_id,
        traceparent.span_id,
        traceparent.trace_flags,
        true,
        TraceState::default(),
    ));

    let span = tracer
        .span_builder(Cow::from(span_name.to_owned()))
        .with_kind(SpanKind::Consumer)
        .start_with_context(tracer, &ctx);

    let ctx = ctx.with_value(traceparent.trace_id);
    let ctx = ctx.with_value(traceparent.span_id);

    (ctx.with_value(traceparent.trace_flags), span)
}
