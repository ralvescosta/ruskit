mod configs;
mod configs_builder;
mod environment;

pub const SECRET_PREFIX: &str = "!";
pub const SECRET_PREFIX_TO_DECODE: &str = "!!";

pub use configs::Config;
pub use configs_builder::ConfigBuilder;
pub use environment::Environment;
