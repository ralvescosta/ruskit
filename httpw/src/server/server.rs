use super::types::AppConfig;
use crate::middlewares;
use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use env::Config;
use errors::http_server::HttpServerError;
use std::sync::Arc;
use tracing::error;

pub struct HttpwServerImpl {
    services: Vec<AppConfig>,
    addr: String,
}

impl HttpwServerImpl {
    pub fn new(cfg: &Config) -> Arc<HttpwServerImpl> {
        Arc::new(HttpwServerImpl {
            services: vec![],
            addr: cfg.app_addr(),
        })
    }
}

impl HttpwServerImpl {
    pub fn register(mut self, service: AppConfig) -> Self {
        self.services.push(service);
        self
    }

    pub async fn server(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let services = self.services.to_vec();
            move || {
                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .wrap(actix_middleware::Logger::default());

                for svc in services.clone() {
                    app = app.configure(svc);
                }

                app.default_service(web::to(middlewares::not_found::not_found))
            }
        })
        .bind(&self.addr)
        .map_err(|e| {
            error!(
                error = e.to_string(),
                "error to binding the http server addr"
            );
            HttpServerError::HttpPortBindError {}
        })?
        .run()
        .await
        .map_err(|e| {
            error!(error = e.to_string(), "error to start http server");
            HttpServerError::ServerError {}
        })?;

        Ok(())
    }

    pub async fn simple_server(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let services = self.services.to_vec();
            move || {
                let mut app = App::new();

                for svc in services.clone() {
                    app = app.configure(svc);
                }

                app
            }
        })
        .bind(&self.addr)
        .map_err(|e| {
            error!(
                error = e.to_string(),
                "error to binding the http server addr"
            );
            HttpServerError::HttpPortBindError {}
        })?
        .run()
        .await
        .map_err(|e| {
            error!(error = e.to_string(), "error to start http server");
            HttpServerError::ServerError {}
        })?;

        Ok(())
    }
}
