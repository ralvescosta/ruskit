# Rust Kit

[![pipeline](https://github.com/ralvescosta/ruskit/actions/workflows/pipeline.yml/badge.svg)](https://github.com/ralvescosta/ruskit/actions/workflows/pipeline.yml) [![codecov](https://codecov.io/gh/ralvescosta/ruskit/branch/main/graph/badge.svg?token=6EAILKZFDO)](https://codecov.io/gh/ralvescosta/ruskit)

**Ruskit** is a collection of useful crates. Every time I started a new project I always needed to configure the same things: *Environment, Logging, Telemetry, Metrics, Custom Errors, RabbitMQ Topology with retry police* and so on. Because of that I decided to create this project to concentrate all these default settings for a web projects.


:warning::construction: **Work In Progress** :construction::warning:

## Table of content

- [Crates](#table-of-content)
  - [Environment](https://github.com/ralvescosta/ruskit/tree/main/env)
  - [Logging](https://github.com/ralvescosta/ruskit/tree/main/logging)
  - [Opentelemetry](https://github.com/ralvescosta/ruskit/tree/main/otel)
  - [Errors](https://github.com/ralvescosta/ruskit/tree/main/errors)
  - [MQTT](https://github.com/ralvescosta/ruskit/tree/main/mqtt)
  - [RabbitMQ](https://github.com/ralvescosta/ruskit/tree/main/amqp)
  - [Actix-web Middlewares](https://github.com/ralvescosta/ruskit/tree/main/httpw)
  - [PostgreSQL Connection](https://github.com/ralvescosta/ruskit/tree/main/postgres)

- [Get Started](#get-started)


## Get Started

To use one of these crates just add to your Cargo.toml

```toml
env = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.0.1" }
logging = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.0.1"  }
errors = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.0.1"  }
```

Where rev = "v1.0.0" is the tag name.