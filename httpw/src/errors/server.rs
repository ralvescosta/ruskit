use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpServerError {
    #[error("http port binding error")]
    PortBidingError,

    #[error("server startup error")]
    ServerStartupError,
}
