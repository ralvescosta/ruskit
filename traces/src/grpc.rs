use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    propagation::{Extractor, Injector},
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

pub struct GRPCInjector<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> GRPCInjector<'a> {
    pub fn new(m: &'a mut tonic::metadata::MetadataMap) -> GRPCInjector<'a> {
        GRPCInjector(m)
    }
}

impl<'a> Injector for GRPCInjector<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

pub fn span(meta: &tonic::metadata::MetadataMap, tracer: &BoxedTracer) -> (Context, BoxedSpan) {
    let ctx = global::get_text_map_propagator(|prop| prop.extract(&GRPCExtractor(meta)));
    let span = tracer.start_with_context("gRPC", &ctx);
    (ctx, span)
}

pub fn inject(ctx: &Context, meta: &mut tonic::metadata::MetadataMap) {
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&ctx, &mut GRPCInjector(meta))
    });
}
