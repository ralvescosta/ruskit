use std::sync::Arc;

use super::types::RouteConfig;
use crate::{authentication::token_validation, errors::HttpServerError, middlewares};
use actix_web::{
    middleware as actix_middleware,
    web::{self, ServiceConfig},
    App, FromRequest, Handler, HttpServer, Responder,
};
use auth::{dummy_middleware::DummyMiddleware, AuthMiddleware};
use env::AppConfig as AppEnv;
use tracing::error;

pub struct HttpwServerImpl {
    services: Vec<RouteConfig>,
    auth_strategy: Arc<dyn AuthMiddleware + Send + Sync>,
    addr: String,
}

impl HttpwServerImpl {
    pub fn new(cfg: &AppEnv) -> HttpwServerImpl {
        HttpwServerImpl {
            services: vec![],
            auth_strategy: DummyMiddleware::new(),
            addr: cfg.app_addr(),
        }
    }
}

impl HttpwServerImpl {
    pub fn register(mut self, service: RouteConfig) -> Self {
        self.services.push(service);
        self
    }

    pub fn auth_strategy(mut self, strategy: Arc<dyn AuthMiddleware + Send + Sync>) -> Self {
        self.auth_strategy = strategy.clone();
        self
    }

    pub async fn start(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let services = self.services.to_vec();
            let auth_strategy = self.auth_strategy.clone();

            move || {
                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .app_data(web::Data::<Arc<dyn AuthMiddleware>>::new(
                        auth_strategy.clone(),
                    ));

                for svc in services.clone() {
                    app = app.configure(svc);
                }

                app.default_service(web::to(middlewares::not_found::not_found))
                    .wrap(actix_middleware::Logger::default())
            }
        })
        .bind(&self.addr)
        .map_err(|e| {
            error!(
                error = e.to_string(),
                "error to binding the http server addr"
            );
            HttpServerError::PortBidingError {}
        })?
        .run()
        .await
        .map_err(|e| {
            error!(error = e.to_string(), "error to start http server");
            HttpServerError::ServerStartupError {}
        })?;

        Ok(())
    }
}
