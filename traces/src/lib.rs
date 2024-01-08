pub mod errors;
pub mod exporters;
pub mod extractors;
pub mod injectors;
pub mod provider;

use configs::{Configs, DynamicConfigs, Environment};
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{
    global::BoxedTracer,
    trace::{SpanKind, Tracer},
    Context,
};
use opentelemetry_sdk::trace::Sampler;
use std::borrow::Cow;

fn get_sampler<T>(cfg: &Configs<T>) -> Sampler
where
    T: DynamicConfigs,
{
    if cfg.app.env == Environment::Local {
        return Sampler::AlwaysOn;
    }

    let sampler = Sampler::TraceIdRatioBased(cfg.trace.export_rate_base);
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
