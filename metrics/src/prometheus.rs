use env::{Configs, DynamicConfig};

pub fn setup<T>(_cfg: &Configs<T>) -> Result<(), ()>
where
    T: DynamicConfig,
{
    Ok(())
}
