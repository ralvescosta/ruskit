# Hedro Rust Crates

[![ci](https://github.com/ralvescosta/ruskit/actions/workflows/ci.yml/badge.svg)](https://github.com/ralvescosta/ruskit/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/ralvescosta/ruskit/branch/main/graph/badge.svg?token=6EAILKZFDO)](https://codecov.io/gh/ralvescosta/ruskit)

**Ruskit** is a collection of useful crates. Every time I started a new project I always needed to configure the same things: *Environment, Logging, Telemetry, Metrics, Custom Errors, RabbitMQ Topology with retry police* and so on. Because of that I decided to create this project to concentrate all these default settings for a web projects.


:warning::construction: **Work In Progress** :construction::warning:

[Table of content]()

  - [crates]()
    - [amqp](https://github.com/ralvescosta/ruskit/tree/main/amqp)
    - [env](https://github.com/ralvescosta/ruskit/tree/main/env)
    - [errors](https://github.com/ralvescosta/ruskit/tree/main/errors)
    - [health_readiness](https://github.com/ralvescosta/ruskit/tree/main/health_readiness)
    - [httpw](https://github.com/ralvescosta/ruskit/tree/main/httpw)
    - [logging](https://github.com/ralvescosta/ruskit/tree/main/logging)
    - [metrics](https://github.com/ralvescosta/ruskit/tree/main/metrics)
    - [mqtt](https://github.com/ralvescosta/ruskit/tree/main/mqtt)
    - [postgres](https://github.com/ralvescosta/ruskit/tree/main/postgres)
    - [secrets_manager](https://github.com/ralvescosta/ruskit/tree/main/secrets_manager)
    - [traces](https://github.com/ralvescosta/ruskit/tree/main/traces)
  - [Requirements](#requirements)
  - [Get Started](#get-started)
  - [Todo](#todo)

## Requirements

- The mqtt crate use the paho client crate, because of that we need to install some Linux utilities to allow us to work with the paho client crate.

```
sudo apt install libssl-dev build-essential cmake pkg-config llvm-dev libclang-dev clang mosquitto-dev libmosquitto-dev
```


## Get Started

To use one of these crates just add to your Cargo.toml

```toml
env = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.0.1" }
logging = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.0.1"  }
errors = { git = "ssh://git@github.com/ralvescosta/ruskit.git",  rev = "v1.0.1"  }
```

Where rev = "v1.0.0" is the tag name.

  
## Todo

- [] AMQP Support to Stream
- [] Graceful shotdown strategy
  - [] Graceful shotdown for RabbitMQ
  - [] Graceful shotdown for MQTT
  - [] Graceful shotdown for gRPC
  - [] Graceful shotdown for HTTP
- [] Create custom middleware strategy to httpw crate
- [] Create OpenAPI render
- [] Improve Logger in httpw crate