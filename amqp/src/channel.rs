use crate::errors::AmqpError;
use env::{Configs, DynamicConfig};
use lapin::{types::LongString, Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use tracing::{debug, error};

pub async fn new_amqp_channel<T>(cfg: &Configs<T>) -> Result<Arc<Channel>, AmqpError>
where
    T: DynamicConfig,
{
    debug!("creating amqp connection...");
    let options = ConnectionProperties::default()
        .with_connection_name(LongString::from(cfg.app.name.clone()));

    let uri = &cfg.amqp_uri();
    let conn = match Connection::connect(uri, options).await {
        Ok(c) => Ok(c),
        Err(err) => {
            error!(error = err.to_string(), "failure to connect");
            Err(AmqpError::ConnectionError {})
        }
    }?;
    debug!("amqp connected");

    debug!("creating amqp channel...");
    match conn.create_channel().await {
        Ok(c) => {
            debug!("channel created");
            Ok(Arc::new(c))
        }
        Err(err) => {
            error!(error = err.to_string(), "error to create the channel");
            Err(AmqpError::ChannelError {})
        }
    }
}
