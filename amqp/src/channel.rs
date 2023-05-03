use crate::errors::AmqpError;
use configs::{Configs, DynamicConfigs};
use lapin::{types::LongString, Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use tracing::{debug, error};

/// Creates a new AMQP connection and channel.
///
/// # Arguments
///
/// * `cfg`: A reference to the `Configs` struct containing configurations loaded from the .env file.
///
/// # Generic type
///
/// * `T`: A type that implements the `DynamicConfigs` trait.
///
/// # Returns
///
/// Returns a `Result` containing a tuple of two `Arc` references: one to the AMQP connection and one to the channel.
/// If an error occurs during the connection or channel creation, an `AmqpError` is returned.
///
/// # Example
///
/// ```no_run
/// use amqp::{errors::AmqpError, channel::new_amqp_channel};
/// use configs::Empty;
/// use configs_builder::ConfigsBuilder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), AmqpError> {
///     // Read configs from .env file
///     let configs = ConfigsBuilder::new().build::<Empty>().await?
///
///     // Create a new connection and channel.
///     let (conn, channel) = new_amqp_channel(&configs).await?;
/// }
/// ```
///
pub async fn new_amqp_channel<T>(
    cfg: &Configs<T>,
) -> Result<(Arc<Connection>, Arc<Channel>), AmqpError>
where
    T: DynamicConfigs,
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
            Ok((Arc::new(conn), Arc::new(c)))
        }
        Err(err) => {
            error!(error = err.to_string(), "error to create the channel");
            Err(AmqpError::ChannelError {})
        }
    }
}
