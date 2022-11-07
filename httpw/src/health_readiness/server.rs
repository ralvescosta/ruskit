use crate::middlewares;
use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use env::Config;

pub async fn server(cfg: &Config) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            //
            // usually register compress first
            //
            .wrap(actix_middleware::Compress::default())
            .wrap(middlewares::headers::config())
            .wrap(middlewares::cors::config())
            //
            // always register Actix Web Logger middleware last middleware
            //
            .wrap(actix_middleware::Logger::default())
            //
            // always register default handler the last handler
            //
            .default_service(web::to(middlewares::default::not_found))
    })
    .bind(cfg.app_addr())?
    .workers(1)
    .run()
    .await
}
