# Hedro Rust Crates

[![pipeline](https://github.com/ralvescosta/ruskit/actions/workflows/pipeline.yml/badge.svg)](https://github.com/ralvescosta/ruskit/actions/workflows/pipeline.yml)

[Table of content]()

  - [crates]()
    - [amqp](https://github.com/ralvescosta/ruskit/tree/main/amqp)
    - [env](https://github.com/ralvescosta/ruskit/tree/main/env)
    - [errors](https://github.com/ralvescosta/ruskit/tree/main/errors)
    - [httpw](https://github.com/ralvescosta/ruskit/tree/main/httpw)
    - [logging](https://github.com/ralvescosta/ruskit/tree/main/logging)
    - [metrics](https://github.com/ralvescosta/ruskit/tree/main/metrics)
    - [mqtt](https://github.com/ralvescosta/ruskit/tree/main/mqtt)
    - [postgres](https://github.com/ralvescosta/ruskit/tree/main/postgres)
    - [secrets_manager](https://github.com/ralvescosta/ruskit/tree/main/secrets_manager)
    - [traces](https://github.com/ralvescosta/ruskit/tree/main/traces)
  - [Requirements](#requirements)
  - [How can i use these crates](#how-can-i-use-these-crates)
  - [Configure Github Actions to use one of than](#configure-github-actions-to-use-one-of-than)
  - [Todo](#todo)

## Requirements

- The mqtt crate use the paho client crate, because of that we need to install some Linux utilities to allow us to work with the paho client crate.

```
sudo apt install libssl-dev build-essential cmake pkg-config llvm-dev libclang-dev clang mosquitto-dev libmosquitto-dev
```

## How can i use these crates?

- First of all you need to configure a ssh key in your github account. For linux users:

  - execute this command:

    ```bash
      ssh-keygen -t ed25519 -C "YOUR-GITHUB-EMAIL"
    ```
  - In sequence you will need to informe the path and the filename to create your new pair of ssh keys. I rely recommend to create a file called github_ed25519 inside the folder .shh like this:

    > /home/$USER/.ssh/github_ed25519

  - Before that you need to provide a password. Be careful, because you will need to use the same password to validate your ssh section.

  - Now you have been created a new ssh file. Now you need to registre the ssh in your ssh-agent.

  - Check if you have and ssh-agent running:

    ```bash
      eval `ssh-agent -s`
    ```

  - The command above will give you a return like this:

    > Agent pid 7696

  - If you have an agente running, just add the new certificate:

    ```bash
      ssh-add
    ```
  - Now you will need to configure the ssh pub key in your github account: Following [this tutorial](https://docs.github.com/pt/authentication/connecting-to-github-with-ssh/adding-a-new-ssh-key-to-your-github-account)

  - To validate if your computer can create a ssh section with github, run the command bellow:

    ```bash
      ssh -T git@github.com
    ```
  
  - The expected output:

    > Hi GITHUB_USER! You've successfully authenticated, but GitHub does not provide shell access.

  - When your ssh was correctly configured, you can use this repo like a crate.
  

## Configure Github Actions to use this repository like a crate

- Create ssh without password:
  ```bash
    ssh-keygen -t ed25519 -C "git@github.com/hedrosistemas/ruskit.git"
  ```

- In the repository that you will use these crate, configure a secrete called **RUSKIT_SSH_PRIVATE_KEY** and put the private key created earlier.

- In theses repository configure the public key inside "Deploy Keys".

- In the repository that you will use these crate, inside the Github Action workflow add the SSH Agent:
  ```yaml
    - name: SSH Agent
      uses: webfactory/ssh-agent@v0.5.4
      with:
        ssh-private-key: |
          ${{ secrets.RUSKIT_SSH_PRIVATE_KEY }}
  ```

## Todo

- [x] AMQP Support to no durable queue
- [] AMQP Support to Stream
- [x] Support to AWS IoT Core
- [] Graceful shotdown strategy
  - [] Graceful shotdown for RabbitMQ
  - [] Graceful shotdown for MQTT
  - [] Graceful shotdown for gRPC
  - [] Graceful shotdown for HTTP
- [] Create custom middleware strategy to httpw crate
- [] Improve process metrics, study proc file to get better metrics
- [] Create OpenAPI render
- [] Improve Logger in httpw crate