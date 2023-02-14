use super::types::RouteConfig;
use crate::{errors::HttpServerError, middlewares};
use actix_web::{
    middleware as actix_middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use actix_web_opentelemetry::{RequestMetricsBuilder, RequestTracing};
use auth::jwt_manager::JwtManager;
use env::AppConfig;
use opentelemetry::global;
use std::sync::Arc;
use tracing::error;

pub struct HttpwServerImpl {
    services: Vec<RouteConfig>,
    jwt_manager: Option<Arc<dyn JwtManager + Send + Sync>>,
    addr: String,
    // openapi_file_path: Option<String>,
}

impl HttpwServerImpl {
    pub fn new(cfg: &AppConfig) -> HttpwServerImpl {
        HttpwServerImpl {
            services: vec![],
            addr: cfg.app_addr(),
            jwt_manager: None,
            // openapi_file_path: None,
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

    // pub fn openapi_file_path(mut self, file_path: String) -> Self {
    //     self.openapi_file_path = Some(file_path);
    //     self
    // }

    pub async fn start(&self) -> Result<(), HttpServerError> {
        HttpServer::new({
            let services = self.services.to_vec();
            let jwt_manager = self.jwt_manager.clone();
            // let openapi_file = self.openapi_file_path.clone();

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

                // if let Some(_openapi) = openapi_file.clone() {
                // let spec = swagger_ui::swagger_spec_file!("../../swagger-ui/examples/openapi.json");
                // let config = swagger_ui::Config::default();
                //
                // let app =  app.service(
                //   scope("/v1/doc")
                //     .configure(actix_web_swagger_ui::swagger(spec, config))
                // );
                //
                // }

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
