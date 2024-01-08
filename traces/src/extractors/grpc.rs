use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    propagation::Extractor,
    trace::Tracer,
    Context,
};

pub struct GRPCExtractor<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> GRPCExtractor<'a> {
    pub fn new(m: &'a tonic::metadata::MetadataMap) -> GRPCExtractor<'a> {
        GRPCExtractor(m)
    }
}

impl<'a> Extractor for GRPCExtractor<'a> {
    /// Get a value for a key from the MetadataMap.  If the value can't be converted to &str, returns None
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    /// Collect all the keys from the MetadataMap.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

pub fn span(meta: &tonic::metadata::MetadataMap, tracer: &BoxedTracer) -> (Context, BoxedSpan) {
    let ctx = global::get_text_map_propagator(|prop| prop.extract(&GRPCExtractor(meta)));
    let span = tracer.start_with_context("gRPC", &ctx);
    (ctx, span)
}
