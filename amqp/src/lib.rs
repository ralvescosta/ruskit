//! # RabbitMQ Abstraction Library
//!
//! This is a library for abstracting away the details of using RabbitMQ with Rust. It provides a
//! higher-level API for interacting with RabbitMQ, including channels, exchanges, queues, and
//! topologies.
//!
//! ## Usage
//!
//! To use this library, add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! amqp = { git = "ssh://git@github.com/ralvescosta/ruskit.git", rev = "vX.XX.X" }
//! ```
//!
//! Then, add the following to your Rust code:
//!
//! ```rust
//! use amqp::{channel, exchange, queue};
//! ```
//!
//! ## Features
//!
//! - `channel`: Provides an abstraction for RabbitMQ channels.
//! - `dispatcher`: Provides a dispatcher for consuming messages from RabbitMQ.
//! - `errors`: Defines error types for this library.
//! - `exchange`: Provides an abstraction for RabbitMQ exchanges.
//! - `publisher`: Provides a publisher for publishing messages to RabbitMQ.
//! - `queue`: Provides an abstraction for RabbitMQ queues.
//! - `topology`: Provides an abstraction for RabbitMQ topology management.
//!
//! ## Testing and Mocking
//!
//! This library provides a set of mock objects for testing and mocking. These are only available if
//! the `mocks` feature is enabled. To enable the `mocks` feature, add the following to your
//! `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! amqp = { git = "ssh://git@github.com/ralvescosta/ruskit.git", features = ["mocks"], rev = "vX.XX.X" }
//! ```
//!
//! The following mock objects are available:
//!
//! - `MockPublisher`: A mock publisher for use in testing.
//! - `MockConsumerHandler`: A mock consumer handler for use in testing.
//!
//! ## Examples
//!
//! Examples of how to use this library can be found in the documentation for each module.
//!
//! ## References
//!
//! - [RabbitMQ Tutorials](https://www.rabbitmq.com/getstarted.html)
//! - [RabbitMQ Rust Client](https://github.com/amqp-rs/lapin)
//!
//! ## License
//!
//! This library is licensed under the MIT License. See the LICENSE file for more information.
//!
//! # Modules
//!
//! - `channel`: Provides an abstraction for RabbitMQ channels.
//! - `dispatcher`: Provides a dispatcher for consuming messages from RabbitMQ.
//! - `errors`: Defines error types for this library.
//! - `exchange`: Provides an abstraction for RabbitMQ exchanges.
//! - `publisher`: Provides a publisher for publishing messages to RabbitMQ.
//! - `queue`: Provides an abstraction for RabbitMQ queues.
//! - `topology`: Provides an abstraction for RabbitMQ topology management.
mod consumer;

pub mod channel;
pub mod dispatcher;
pub mod errors;
pub mod exchange;
pub mod otel;
pub mod publisher;
pub mod queue;
pub mod topology;

#[cfg(test)]
pub use dispatcher::MockConsumerHandler;
#[cfg(feature = "mocks")]
pub use dispatcher::MockConsumerHandler;
#[cfg(test)]
pub use publisher::MockPublisher;
#[cfg(feature = "mocks")]
pub use publisher::MockPublisher;
