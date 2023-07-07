#[cfg(feature = "validator")]
mod validate;

#[cfg(feature = "validator")]
pub use validate::body_validator;
