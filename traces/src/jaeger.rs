use configs::{Configs, DynamicConfigs};
use std::error::Error;

pub fn setup<T>(_cfg: &Configs<T>) -> Result<(), Box<dyn Error>>
where
    T: DynamicConfigs,
{
    Ok(())
}
