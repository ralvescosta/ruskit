use crate::errors::HTTPServerError;
use actix_web::{
    http::KeepAlive,
    middleware::{self as actix_middleware, Logger},
    web::{self, Data},
    App, HttpServer as ActixHttpServer,
};
use configs::AppConfigs;
use health_readiness::{HealthReadinessService, HealthReadinessServiceImpl};
use http_components::{
    handlers::health_handler,
    middlewares::{
        self,
        otel::{HTTPOtelMetrics, HTTPOtelTracing},
    },
    CustomServiceConfigure,
};
use opentelemetry::global;
use std::{sync::Arc, time::Duration};
use tracing::error;
#[cfg(feature = "openapi")]
use utoipa::openapi::OpenApi;
#[cfg(feature = "openapi")]
use utoipa_swagger_ui::SwaggerUi;

pub struct HTTPServer {
    addr: String,
    services: Vec<Arc<CustomServiceConfigure>>,
    #[cfg(feature = "openapi")]
    openapi: Option<OpenApi>,
    health_check: Option<Arc<dyn HealthReadinessService>>,
}

impl HTTPServer {
    pub fn new(cfg: &AppConfigs) -> HTTPServer {
        HTTPServer {
            addr: cfg.app_addr(),
            services: vec![],
            #[cfg(feature = "openapi")]
            openapi: None,
            health_check: None,
        }
    }
}

impl HTTPServer {
    pub fn custom_configure(mut self, s: CustomServiceConfigure) -> Self {
        self.services.push(Arc::new(s));
        self
    }

    #[cfg(feature = "openapi")]
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
            #[cfg(feature = "openapi")]
            let openapi = self.openapi.clone();

            let health_check_service = match self.health_check.clone() {
                Some(check) => check,
                _ => Arc::new(HealthReadinessServiceImpl::default()),
            };

            let services = self.services.clone();

            move || {
                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .wrap(HTTPOtelTracing::new())
                    .wrap(HTTPOtelMetrics::new())
                    .app_data(middlewares::deserializer::handler())
                    .app_data(Data::<dyn HealthReadinessService>::from(
                        health_check_service.clone(),
                    ))
                    .service(health_handler);

                let services = services.clone();
                app = app.configure(move |config| {
                    for svc in services.clone() {
                        let cl = svc.clone();
                        let mut f = cl.f.lock().unwrap();
                        f(config);
                    }
                });

                #[cfg(feature = "openapi")]
                if openapi.is_some() {
                    app = app.service(
                        SwaggerUi::new("/docs/{_:.*}")
                            .url("/docs/openapi.json", openapi.clone().unwrap()),
                    );
                }

                app.default_service(web::to(middlewares::not_found::not_found))
                    .wrap(Logger::default())
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
