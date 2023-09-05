#[cfg(feature = "mqtt")]
use crate::mqtt::MqttHealthChecker;
#[cfg(feature = "postgres")]
use crate::postgres::PostgresHealthChecker;
#[cfg(feature = "rabbitmq")]
use crate::rabbitmq::RabbitMqHealthChecker;
use crate::{
    controller, errors::HealthReadinessError, HealthChecker, HealthReadinessService,
    HealthReadinessServiceImpl,
};
use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use configs::HealthReadinessConfigs;
#[cfg(feature = "postgres")]
use deadpool_postgres::Pool;
use http_components::middlewares;
#[cfg(feature = "rabbitmq")]
use lapin::Connection;
#[cfg(feature = "mqtt")]
use paho_mqtt::AsyncClient;
use std::sync::Arc;
use tracing::{debug, error};

pub struct HealthReadinessServer {
    checkers: Vec<Arc<dyn HealthChecker>>,
    addr: String,
    enable: bool,
}

impl HealthReadinessServer {
    pub fn new(cfg: &HealthReadinessConfigs) -> HealthReadinessServer {
        HealthReadinessServer {
            checkers: vec![],
            addr: cfg.health_readiness_addr(),
            enable: cfg.enable,
        }
    }

    #[cfg(feature = "mqtt")]
    pub fn mqtt(mut self, client: Arc<AsyncClient>) -> Self {
        self.checkers.push(MqttHealthChecker::new(client));
        return self;
    }

    #[cfg(feature = "rabbitmq")]
    pub fn rabbitmq(mut self, conn: Arc<Connection>) -> Self {
        self.checkers.push(RabbitMqHealthChecker::new(conn));
        return self;
    }

    #[cfg(feature = "postgres")]
    pub fn postgres(mut self, pool: Arc<Pool>) -> Self {
        self.checkers.push(PostgresHealthChecker::new(pool));
        return self;
    }

    pub fn dynamodb(self) -> Self {
        return self;
    }
}

impl HealthReadinessServer {
    pub async fn run(&self) -> Result<(), HealthReadinessError> {
        if !self.enable {
            debug!("skipping health readiness server");
            return Ok(());
        }

        HttpServer::new({
            let health_readiness_service = HealthReadinessServiceImpl::new(self.checkers.clone());

            move || {
                App::new()
                    .wrap(actix_middleware::Compress::default())
                    .wrap(middlewares::headers::config())
                    .wrap(middlewares::cors::config())
                    .wrap(actix_middleware::Logger::default())
                    //
                    .app_data(web::Data::<Arc<dyn HealthReadinessService>>::new(
                        health_readiness_service.clone(),
                    ))
                    //
                    //  Health route
                    //
                    .service(controller::health_handler)
                    .default_service(web::to(middlewares::not_found::not_found))
            }
        })
        .bind(&self.addr)
        .map_err(|e| {
            error!(error = e.to_string(), "error to bind health readiness addr");
            HealthReadinessError::ServerError {}
        })?
        .workers(1)
        .run()
        .await
        .map_err(|e| {
            error!(
                error = e.to_string(),
                "error to run health readiness server"
            );
            HealthReadinessError::ServerError {}
        })?;

        Ok(())
    }
}
