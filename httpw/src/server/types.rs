use actix_web::web::ServiceConfig;

pub type AppConfig = fn(cfg: &mut ServiceConfig);
