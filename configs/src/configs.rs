use crate::{
    kafka::KafkaConfigs, AppConfigs, AwsConfigs, DynamoConfigs, HealthReadinessConfigs,
    IdentityServerConfigs, MQTTConfigs, MetricConfigs, PostgresConfigs, RabbitMQConfigs,
    SqliteConfigs, TraceConfigs,
};

#[derive(Debug, Clone, Default)]
pub struct Configs<T: DynamicConfigs> {
    pub app: AppConfigs,
    pub identity: IdentityServerConfigs,
    pub mqtt: MQTTConfigs,
    pub rabbitmq: RabbitMQConfigs,
    pub kafka: KafkaConfigs,
    pub metric: MetricConfigs,
    pub trace: TraceConfigs,
    pub postgres: PostgresConfigs,
    pub sqlite: SqliteConfigs,
    pub aws: AwsConfigs,
    pub dynamo: DynamoConfigs,
    pub health_readiness: HealthReadinessConfigs,

    pub dynamic: T,
}

pub trait DynamicConfigs: Default {
    fn load(&mut self);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty;
impl DynamicConfigs for Empty {
    fn load(&mut self) {}
}

impl<T> Configs<T>
where
    T: DynamicConfigs,
{
    pub fn rabbitmq_uri(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}{}",
            self.rabbitmq.user,
            self.rabbitmq.password,
            self.rabbitmq.host,
            self.rabbitmq.port,
            self.rabbitmq.vhost
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_app_addr() {
        let cfg = AppConfigs::default();

        assert_eq!(cfg.app_addr(), format!("{}:{}", cfg.host, cfg.port))
    }

    #[test]
    fn should_return_amqp_uri() {
        let cfg = Configs::<Empty>::default();

        assert_eq!(
            cfg.rabbitmq_uri(),
            format!(
                "amqp://{}:{}@{}:{}{}",
                cfg.rabbitmq.user,
                cfg.rabbitmq.password,
                cfg.rabbitmq.host,
                cfg.rabbitmq.port,
                cfg.rabbitmq.vhost
            )
        )
    }
}
