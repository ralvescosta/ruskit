use std::sync::Arc;

use super::types::AppConfig;
use crate::{authentication::token_validation, errors::HttpServerError, middlewares};
use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use auth::{dummy_middleware::DummyMiddleware, AuthMiddleware};
use env::AppConfig as AppEnv;
use tracing::error;

pub struct HttpwServerImpl {
    services_without_auth: Vec<AppConfig>,
    services_with_auth: Vec<AppConfig>,
    with_auth: bool,
    auth_strategy: Arc<dyn AuthMiddleware + Send + Sync>,
    addr: String,
}

impl HttpwServerImpl {
    pub fn new(cfg: &AppEnv) -> HttpwServerImpl {
        HttpwServerImpl {
            services_without_auth: vec![],
            services_with_auth: vec![],
            with_auth: false,
            auth_strategy: DummyMiddleware::new(),
            addr: cfg.app_addr(),
        }
    }
}

impl HttpwServerImpl {
    pub fn register(mut self, service: AppConfig) -> Self {
        if self.with_auth {
            self.services_with_auth.push(service);
        } else {
            self.services_without_auth.push(service);
        }
        self
    }

    pub fn auth_strategy(mut self, strategy: Arc<dyn AuthMiddleware + Send + Sync>) -> Self {
        self.with_auth = true;
        self.auth_strategy = strategy.clone();
        self
    }

    pub async fn start(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let with_auth = self.with_auth;
            let services_without_auth = self.services_without_auth.to_vec();
            let services_with_auth = self.services_with_auth.to_vec();
            let auth_strategy = self.auth_strategy.clone();

            move || {
                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .app_data(web::Data::<Arc<dyn AuthMiddleware>>::new(
                        auth_strategy.clone(),
                    ));

                for svc in services_without_auth.clone() {
                    app = app.configure(svc);
                }

                if with_auth {
                    // let auth_mid = HttpAuthentication::bearer(token_validation::validator);
                    // app = app.wrap(actix_middleware::Logger::default());
                }

                for svc in services_with_auth.clone() {
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
