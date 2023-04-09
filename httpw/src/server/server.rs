use crate::errors::HTTPServerError;
use actix_web::web::ServiceConfig;
use actix_web::{
    http::KeepAlive,
    middleware as actix_middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer as ActixHttpServer, Responder,
};
use actix_web_opentelemetry::{RequestMetricsBuilder, RequestTracing};
use auth::jwt_manager::JwtManager;
use configs::AppConfigs;
use health_readiness::{
    controller::health_handler,
    {HealthChecker, HealthReadinessService, HealthReadinessServiceImpl},
};
use http_components::middlewares;
use opentelemetry::global;
use std::{sync::Arc, time::Duration};
use tracing::error;
use utoipa::openapi::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub type RouteConfig = fn(cfg: &mut ServiceConfig);

pub struct HTTPServer {
    addr: String,
    services: Vec<RouteConfig>,
    openapi: Option<OpenApi>,
    jwt_manager: Option<Arc<dyn JwtManager>>,
    health_check: Option<Arc<dyn HealthReadinessService>>,
}

impl HTTPServer {
    pub fn new(cfg: &AppConfigs) -> HTTPServer {
        HTTPServer {
            addr: cfg.app_addr(),
            services: vec![],
            openapi: None,
            jwt_manager: None,
            health_check: None,
        }
    }
}

impl HTTPServer {
    pub fn register(mut self, service: RouteConfig) -> Self {
        self.services.push(service);
        self
    }

    pub fn jwt_manager(mut self, manager: Arc<dyn JwtManager>) -> Self {
        self.jwt_manager = Some(manager);
        self
    }

    pub fn openapi(mut self, openapi: &OpenApi) -> Self {
        self.openapi = Some(openapi.to_owned());
        self
    }

    pub fn health_check(mut self, service: Arc<dyn HealthReadinessService>) -> Self {
        self.health_check = Some(service);
        self
    }

    pub async fn start(&self) -> Result<(), HTTPServerError> {
        ActixHttpServer::new({
            let services = self.services.to_vec();
            let jwt_manager = self.jwt_manager.clone();
            let openapi = self.openapi.clone();
            let health_check_service = match self.health_check.clone() {
                Some(check) => check,
                _ => HealthReadinessServiceImpl::new(),
            };

            move || {
                let meter = global::meter("actix_web");

                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .wrap(RequestTracing::new())
                    .wrap(RequestMetricsBuilder::new().build(meter))
                    .app_data::<Data<Arc<dyn HealthReadinessService>>>(Data::<Arc<dyn HealthReadinessService>>::new(health_check_service.clone()));

                if let Some(jwt_manager) = jwt_manager.clone() {
                    app = app.app_data::<Data<Arc<dyn JwtManager>>>(
                        Data::<Arc<dyn JwtManager>>::new(jwt_manager.clone()),
                    );
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

                app.service(health_handler)
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
            HTTPServerError::PortBidingError {}
        })?
        .run()
        .await
        .map_err(|e| {
            error!(error = e.to_string(), "error to start http server");
            HTTPServerError::ServerStartupError {}
        })?;

        global::shutdown_tracer_provider();

        Ok(())
    }
}
