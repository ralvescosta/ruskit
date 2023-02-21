use super::types::RouteConfig;
use crate::{errors::HttpServerError, middlewares};
use actix_web::{
    http::KeepAlive,
    middleware as actix_middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use actix_web_opentelemetry::{RequestMetricsBuilder, RequestTracing};
use auth::jwt_manager::JwtManager;
use env::AppConfig;
use opentelemetry::global;
use std::{sync::Arc, time::Duration};
use tracing::error;
use utoipa::openapi::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct HttpwServerImpl {
    services: Vec<RouteConfig>,
    jwt_manager: Option<Arc<dyn JwtManager + Send + Sync>>,
    addr: String,
    openapi: Option<OpenApi>,
}

impl HttpwServerImpl {
    pub fn new(cfg: &AppConfig) -> HttpwServerImpl {
        HttpwServerImpl {
            services: vec![],
            addr: cfg.app_addr(),
            jwt_manager: None,
            openapi: None,
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

    pub fn openapi(mut self, openapi: &OpenApi) -> Self {
        self.openapi = Some(openapi.to_owned());
        self
    }

    pub async fn start(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let services = self.services.to_vec();
            let jwt_manager = self.jwt_manager.clone();
            let openapi = self.openapi.clone();

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

                if openapi.is_some() {
                    app = app.service(
                        SwaggerUi::new("/docs/{_:.*}")
                            .url("/docs/openapi.json", openapi.clone().unwrap()),
                    );
                }

                app.route("/health", web::get().to(health_handler))
                    .default_service(web::to(middlewares::not_found::not_found))
                    .wrap(actix_middleware::Logger::default())
            }
        })
        .shutdown_timeout(60)
        .keep_alive(KeepAlive::Timeout(Duration::new(2, 0)))
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

        global::shutdown_tracer_provider();

        Ok(())
    }
}

async fn health_handler() -> impl Responder {
    HttpResponse::Ok().body("health")
}
