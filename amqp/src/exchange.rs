use crate::errors::AmqpError;
use lapin::types::{AMQPValue, LongString, ShortString};
use std::collections::BTreeMap;

pub const AMQP_HEADERS_DELAYED_EXCHANGE_TYPE: &str = "x-delayed-type";

/// Enumeration of possible AMQP exchange kinds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ExchangeKind {
    /// A direct exchange.
    #[default]
    Direct,
    /// A fanout exchange.
    Fanout,
    /// A topic exchange.
    Topic,
    /// A headers exchange.
    Headers,
    /// A custom exchange that enables message delay.
    XMessageDelayed,
}

impl TryInto<lapin::ExchangeKind> for ExchangeKind {
    type Error = AmqpError;

    /// Attempts to convert this `ExchangeKind` to a `lapin::ExchangeKind`.
    ///
    /// # Errors
    ///
    /// If the conversion fails, an `AmqpError` is returned. The error variants are:
    ///
    /// * `DeclareExchangeError` - Failure to declare the exchange.
    fn try_into(self) -> Result<lapin::ExchangeKind, AmqpError> {
        match self {
            ExchangeKind::Direct => Ok(lapin::ExchangeKind::Direct),
            ExchangeKind::Fanout => Ok(lapin::ExchangeKind::Fanout),
            ExchangeKind::Headers => Ok(lapin::ExchangeKind::Headers),
            ExchangeKind::Topic => Ok(lapin::ExchangeKind::Topic),
            ExchangeKind::XMessageDelayed => {
                Ok(lapin::ExchangeKind::Custom("x-delayed-message".to_owned()))
            }
        }
    }
}

/// A struct representing an AMQP exchange definition.
///
/// # Example
///
/// ```no_run
/// use std::collections::BTreeMap;
/// use lapin::{
///     message::AMQPValue,
///     types::{ExchangeDefinition, ExchangeKind, ShortString, LongString},
/// };
/// use amqp::exchange::ExchangeDefinition;
///
/// fn main() {
///     // Create a new exchange definition with the name "my_exchange".
///     let exchange_def = ExchangeDefinition::new("my_exchange")
///         .durable()
///         .direct();
///
///     // Add a parameter to the exchange definition.
///     let mut params = BTreeMap::new();
///     params.insert(
///         ShortString::from("my_param"),
///         AMQPValue::LongString(LongString::from("my_value")),
///     );
///     let exchange_def_with_params = exchange_def.params(params);
///
///     // Print the exchange definition.
///     println!("{:?}", exchange_def_with_params);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ExchangeDefinition<'ex> {
    pub(crate) name: &'ex str,
    pub(crate) kind: &'ex ExchangeKind,
    pub(crate) delete: bool,
    pub(crate) durable: bool,
    pub(crate) passive: bool,
    pub(crate) internal: bool,
    pub(crate) no_wait: bool,
    pub(crate) params: BTreeMap<ShortString, AMQPValue>,
}

impl<'ex> ExchangeDefinition<'ex> {
    pub fn new(name: &'ex str) -> ExchangeDefinition<'ex> {
        ExchangeDefinition {
            name,
            kind: &ExchangeKind::Direct,
            delete: false,
            durable: false,
            passive: false,
            internal: false,
            no_wait: false,
            params: BTreeMap::default(),
        }
    }

    pub fn kind(mut self, kind: &'ex ExchangeKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn direct(mut self) -> Self {
        self.kind = &ExchangeKind::Direct;
        self
    }

    pub fn fanout(mut self) -> Self {
        self.kind = &ExchangeKind::Fanout;
        self
    }

    pub fn direct_delead(mut self) -> Self {
        self.kind = &ExchangeKind::XMessageDelayed;
        self.params.insert(
            ShortString::from(AMQP_HEADERS_DELAYED_EXCHANGE_TYPE),
            AMQPValue::LongString(LongString::from("direct")),
        );
        self
    }

    pub fn fanout_delead(mut self) -> Self {
        self.kind = &ExchangeKind::XMessageDelayed;
        self.params.insert(
            ShortString::from(AMQP_HEADERS_DELAYED_EXCHANGE_TYPE),
            AMQPValue::LongString(LongString::from("fanout")),
        );
        self
    }

    pub fn params(mut self, params: BTreeMap<ShortString, AMQPValue>) -> Self {
        self.params = params;
        self
    }

    pub fn param(mut self, key: ShortString, value: AMQPValue) -> Self {
        self.params.insert(key, value);
        self
    }

    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    pub fn durable(mut self) -> Self {
        self.durable = true;
        self
    }

    pub fn passive(mut self) -> Self {
        self.passive = self.passive;
        self
    }

    pub fn internal(mut self) -> Self {
        self.internal = true;
        self
    }

    pub fn no_wait(mut self) -> Self {
        self.no_wait = true;
        self
    }
}

pub struct ExchangeBinding {}
