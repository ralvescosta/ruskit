use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Internal Error")]
    InternalError,

    #[error("str conversion error")]
    ConversionError(),
}
