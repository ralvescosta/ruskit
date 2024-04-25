use actix_web::web::ServiceConfig;
use std::sync::Mutex;

pub mod extra;
pub mod extractors;
pub mod handlers;
pub mod middlewares;
pub mod viewmodels;

pub struct CustomServiceConfigure {
    pub f: Mutex<Box<dyn FnMut(&mut ServiceConfig) + Send + Sync>>,
}

impl CustomServiceConfigure {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut ServiceConfig) + Send + Sync + 'static,
    {
        Self {
            f: Mutex::new(Box::new(f)),
        }
    }
}
