use super::types::RouteConfig;
use crate::{errors::HttpServerError, middlewares};
use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use auth::jwt_manager::JwtManager;
use env::AppConfig;
use std::sync::Arc;
use tracing::error;

pub struct HttpwServerImpl {
    services: Vec<RouteConfig>,
    jwt_manager: Option<Arc<dyn JwtManager + Send + Sync>>,
    addr: String,
}

impl HttpwServerImpl {
    pub fn new(cfg: &AppConfig) -> HttpwServerImpl {
        HttpwServerImpl {
            services: vec![],
            addr: cfg.app_addr(),
            jwt_manager: None,
        }
    }
}

impl HttpwServerImpl {
    pub fn register(mut self, service: RouteConfig) -> Self {
        self.services.push(service);
        self
    }

    pub fn jwt_manager(mut self, manager: Arc<dyn JwtManager + Send + Sync>) -> Self {
        self.jwt_manager = Some(manager);
        self
    }

    pub async fn start(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let services = self.services.to_vec();
            let jwt_manager = self.jwt_manager.clone();

            move || {
                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config());

                if let Some(jwt_manager) = jwt_manager.clone() {
                    app = app.app_data(web::Data::<Arc<dyn JwtManager + Send + Sync>>::new(
                        jwt_manager,
                    ));
                }

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
