mod aws_client;
mod aws_client_builder;
mod dummy_client;

pub mod errors;
#[cfg(test)]
pub mod mocks;
#[cfg(feature = "mocks")]
pub mod mocks;
mod types;

pub use aws_client::AwsSecretClient;
pub use aws_client_builder::AwsSecretClientBuilder;
pub use dummy_client::DummyClient;
pub use types::SecretClient;
