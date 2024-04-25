use crate::errors::HTTPServerError;
use actix_web::{
    http::KeepAlive,
    middleware as actix_middleware,
    web::{self, Data},
    App, HttpServer as ActixHttpServer,
};
use configs::{Configs, DynamicConfigs};
use health_readiness::{HealthReadinessService, HealthReadinessServiceImpl};
#[cfg(feature = "prometheus")]
use http_components::handlers::PrometheusMetricsHandler;
use http_components::{handlers::health_handler, middlewares, CustomServiceConfigure};
use opentelemetry::global;
#[cfg(feature = "prometheus")]
use prometheus::Registry;
use std::{sync::Arc, time::Duration};
use tracing::{error, info};

pub struct TinyHTTPServer {
    addr: String,
    enabled: bool,
    services: Vec<Arc<CustomServiceConfigure>>,
    health_check: Option<Arc<dyn HealthReadinessService>>,
    #[cfg(feature = "prometheus")]
    metrics_registry: Option<Arc<Registry>>,
}

impl TinyHTTPServer {
    pub fn new<T>(cfg: &Configs<T>) -> TinyHTTPServer
    where
        T: DynamicConfigs,
    {
        TinyHTTPServer {
            addr: cfg.health_readiness.health_readiness_addr(),
            enabled: cfg.health_readiness.enable,
            services: vec![],
            health_check: None,
            #[cfg(feature = "prometheus")]
            metrics_registry: None,
        }
    }
}

impl TinyHTTPServer {
    pub fn custom_configure(mut self, s: CustomServiceConfigure) -> Self {
        self.services.push(Arc::new(s));
        self
    }

    pub fn health_check(mut self, service: Arc<dyn HealthReadinessService>) -> Self {
        self.health_check = Some(service);
        self
    }

    #[cfg(feature = "prometheus")]
    pub fn metrics_registry(mut self, registry: Arc<Registry>) -> Self {
        self.metrics_registry = Some(registry);
        self
    }

    pub async fn start(&self) -> Result<(), HTTPServerError> {
        if !self.enabled {
            info!("skipping health http server!");
            return Ok(());
        }

        match ActixHttpServer::new({
            let health_check_service = match self.health_check.clone() {
                Some(check) => check,
                _ => Arc::new(HealthReadinessServiceImpl::default()),
            };
            let services = self.services.clone();

            #[cfg(feature = "prometheus")]
            let metrics_registry = self.metrics_registry.clone();

            move || {
                let mut app = App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .app_data(Data::<Arc<dyn HealthReadinessService>>::new(
                        health_check_service.clone(),
                    ));

                let services = services.clone();
                app = app.configure(move |config| {
                    for svc in services.clone() {
                        let cl = svc.clone();
                        let mut f = cl.f.lock().unwrap();
                        f(config);
                    }
                });

                #[cfg(feature = "prometheus")]
                if metrics_registry.is_some() {
                    let handler = PrometheusMetricsHandler::new(metrics_registry.clone().unwrap());
                    app = app.configure(move |config| {
                        config.service(web::resource("/metrics").route(web::get().to(handler)));
                    });
                }

                app.service(health_handler)
                    .default_service(web::to(middlewares::not_found::not_found))
                    .wrap(actix_middleware::Logger::default())
            }
        })
        .shutdown_timeout(60)
        .keep_alive(KeepAlive::Timeout(Duration::new(2, 0)))
        .bind(&self.addr)
        {
            Ok(server) => match server.run().await {
                Err(err) => {
                    global::shutdown_tracer_provider();
                    error!(error = err.to_string(), "error to start http server");
                    Err(HTTPServerError::ServerStartupError {})
                }
                _ => Ok(()),
            },
            Err(err) => {
                global::shutdown_tracer_provider();
                error!(
                    error = err.to_string(),
                    "error to binding the http server addr"
                );
                Err(HTTPServerError::PortBidingError {})
            }
        }
    }
}
