use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HealthReadinessError {
    #[error("postgres connection error")]
    PostgresError,

    #[error("rabbitmq connection error")]
    RabbitMqError,

    #[error("mqtt broker connection error")]
    MqttError,

    #[error("health readiness server error")]
    ServerError,
}
