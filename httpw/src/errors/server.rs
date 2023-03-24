use thiserror::Error;

#[derive(Debug, Error)]
pub enum HTTPServerError {
    #[error("http port binding error")]
    PortBidingError,

    #[error("server startup error")]
    ServerStartupError,
}
