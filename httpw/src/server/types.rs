use actix_web::web::ServiceConfig;

pub type RouteConfig = fn(cfg: &mut ServiceConfig);
