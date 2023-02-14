use super::types::RouteConfig;
use crate::middlewares;
use crate::{errors::HttpServerError, middlewares};
use actix_web::{
    middleware as actix_middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use actix_web_opentelemetry::{RequestMetricsBuilder, RequestTracing};
use auth::jwt_manager::JwtManager;
use env::AppConfig as AppEnv;
use env::AppConfig;
use errors::http_server::HttpServerError;
use opentelemetry::global;
use std::sync::Arc;
use tracing::error;

pub struct HttpwServerImpl {
    services: Vec<AppConfig>,
    addr: String,
}

impl HttpwServerImpl {
    pub fn new(cfg: &AppEnv) -> Arc<HttpwServerImpl> {
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

    pub async fn start(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let services = self.services.to_vec();
            move || {
                let meter = global::meter("actix_web");

                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .wrap(RequestTracing::new())
                    .wrap(RequestMetricsBuilder::new().build(meter));

                if let Some(jwt_manager) = jwt_manager.clone() {
                    app = app.app_data::<Data<Arc<dyn JwtManager + Send + Sync>>>(web::Data::<
                        Arc<dyn JwtManager + Send + Sync>,
                    >::new(
                        jwt_manager.clone(),
                    ));
                }

                for svc in services.clone() {
                    app = app.configure(svc);
                }

                app.route("/health", web::get().to(health_handler))
                    .default_service(web::to(middlewares::not_found::not_found))
                    .wrap(actix_middleware::Logger::default())
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

        global::shutdown_tracer_provider();

        Ok(())
    }
}

async fn health_handler() -> impl Responder {
    HttpResponse::Ok().body("health")
}
