# Ruskit

:warning::construction: **Work In Progress** :construction::warning:

[![ci](https://github.com/ralvescosta/ruskit/actions/workflows/ci.yml/badge.svg)](https://github.com/ralvescosta/ruskit/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/ralvescosta/ruskit/branch/main/graph/badge.svg?token=6EAILKZFDO)](https://codecov.io/gh/ralvescosta/ruskit)

[![GitHub License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ralvescosta/ruskit/blob/main/LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/ralvescosta/ruskit.svg)](https://github.com/ralvescosta/ruskit/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/ralvescosta/ruskit.svg)](https://github.com/ralvescosta/ruskit/network)
[![GitHub Issues](https://img.shields.io/github/issues/ralvescosta/ruskit.svg)](https://github.com/ralvescosta/ruskit/issues)

Ruskit is an open-source project written in Rust that provides a set of powerful abstractions and utilities for building cloud-native applications. It simplifies common tasks related to logging, observability, messaging, databases, health checks, and more. Whether you're a seasoned Rust developer or just getting started with cloud-native development, Ruskit aims to streamline your workflow and make your applications more reliable and efficient.

[Table of content]()

  - [Features](#features)
  - [Getting started](#getting-started)
  - [Documentation](#documentation)
  - [License](#license)
  - [Support](#support)
  

# Features

- **Logging:** Easily configure and use structured logging in your Rust applications, improving debugging and monitoring capabilities.

- **OpenTelemetry and Tracing:** Integration with OpenTelemetry for distributed tracing, helping you gain insights into your application's performance.

- **OpenTelemetry and Metrics:** Seamless integration with Prometheus for collecting and exposing application metrics.

- **Messaging:** Support for MQTT and RabbitMQ, making it straightforward to implement messaging patterns in your applications.

- **Database:** Integration with PostgreSQL for efficient data storage and retrieval.

- **Health Checks:** Implement health checks to ensure the reliability of your services and applications.

- **Migrator:** Simplify database schema migrations to manage changes in your data models.

- **HTTP Server:** Simplify the way to create a production ready HTTP Servers with OpenAPI, health check, metrics and more.

# Getting Started

To get started with Ruskit, follow these steps:

1- **Installing Linux packages:**  Some crates require some additional packages to work pronely

```bash
sudo apt install libssl-dev build-essential cmake pkg-config llvm-dev libclang-dev clang libmosquitto-dev sqlite3
```

2- **Add in our cargo.toml:** To use one of these crates just add to your Cargo.toml:

```toml
env = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.25.0" }
logging = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.25.0"  }
errors = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.25.0"  }
```

# Documentation

For detailed documentation and usage examples, please visit our [ruskit example repository](https://github.com/ralvescosta/ruskit_examples)

# License

Ruskit is released under the MIT License. See [LICENSE](https://github.com/ralvescosta/ruskit/blob/main/LICENSE) for more details.

# Support

If you have questions, encounter issues, or want to discuss ideas, please open an issue on the [GitHub Issues](https://github.com/ralvescosta/ruskit/issues) page.