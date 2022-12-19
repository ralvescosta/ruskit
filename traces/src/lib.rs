///deprecated
pub mod amqp;

pub mod jaeger;
pub mod otlp;

use env::{Config, Environment};
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{
    global::BoxedTracer,
    sdk::trace::Sampler,
    trace::{SpanKind, Tracer},
    Context,
};
use std::borrow::Cow;

fn get_sampler(cfg: &Config) -> Sampler {
    if cfg.app.env == Environment::Local {
        return Sampler::AlwaysOn;
    }

    let sampler = Sampler::TraceIdRatioBased(0.8);
    return Sampler::ParentBased(Box::new(sampler));
}

pub fn span_ctx(tracer: &BoxedTracer, kind: SpanKind, name: &str) -> Context {
    let span = tracer
        .span_builder(Cow::from(name.to_owned()))
        .with_kind(kind)
        .start(tracer);

    Context::current_with_span(span)
}

pub fn trace_id(ctx: &Context) -> String {
    let span = ctx.span();

    if span.is_recording() {
        let span_ctx = span.span_context();

        return span_ctx.trace_id().to_string();
    }

    String::new()
}

pub fn span_id(ctx: &Context) -> String {
    let span = ctx.span();

    if span.is_recording() {
        let span_ctx = span.span_context();

        return span_ctx.span_id().to_string();
    }

    String::new()
}
