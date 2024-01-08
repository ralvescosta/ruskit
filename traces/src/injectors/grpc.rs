use opentelemetry::{
    global::{self},
    propagation::Injector,
    Context,
};

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

pub fn inject(ctx: &Context, meta: &mut tonic::metadata::MetadataMap) {
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&ctx, &mut GRPCInjector(meta))
    });
}
