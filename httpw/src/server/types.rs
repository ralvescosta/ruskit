use actix_web::web::ServiceConfig;

pub type AppConfig = fn(cfg: &mut ServiceConfig);

pub struct Auth {
    permission: Option<String>,
}

pub struct Route {
    path: String,
    method: String,
    auth: Option<Auth>,
}

pub struct RoutesConfig {
    actix_cfg: fn(cfg: &mut ServiceConfig),
    routes: Vec<Route>,
}
