use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TracesError {
    #[error("internal error")]
    InternalError,

    #[error("this exporter requires specific features")]
    InvalidFeaturesError,

    #[error("conversion error")]
    ConversionError,

    #[error("failure to create the exporter provide")]
    ExporterProviderError,
}
