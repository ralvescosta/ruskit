use actix_web::http::header::HeaderMap;
use opentelemetry::propagation::Extractor;

pub struct HTTPExtractor<'a> {
    headers: &'a HeaderMap,
}

impl<'a> HTTPExtractor<'a> {
    pub fn new(headers: &'a HeaderMap) -> Self {
        HTTPExtractor { headers }
    }
}

impl<'a> Extractor for HTTPExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|header| header.as_str()).collect()
    }
}
