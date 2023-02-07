use super::types::AppConfig;
use crate::middlewares;
use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use env::AppConfig as AppEnv;
use errors::http_server::HttpServerError;
use tracing::error;

pub struct HttpwServerImpl {
    services_without_auth: Vec<AppConfig>,
    services_with_auth: Vec<AppConfig>,
    with_auth: bool,
    addr: String,
}

impl HttpwServerImpl {
    pub fn new(cfg: &AppEnv) -> HttpwServerImpl {
        HttpwServerImpl {
            services_without_auth: vec![],
            services_with_auth: vec![],
            with_auth: false,
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

    pub async fn start(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let with_auth = self.with_auth;
            let services_without_auth = self.services_without_auth.to_vec();
            let services_with_auth = self.services_with_auth.to_vec();

            move || {
                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .wrap(actix_middleware::Logger::default());

                for svc in services_without_auth.clone() {
                    app = app.configure(svc);
                }

                if with_auth {
                    //apply auth strategy
                }

                for svc in services_with_auth.clone() {
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
            HttpServerError::ServerError {}
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
