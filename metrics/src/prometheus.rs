use configs::{Configs, DynamicConfigs};

pub fn setup<T>(_cfg: &Configs<T>) -> Result<(), ()>
where
    T: DynamicConfigs,
{
    Ok(())
}
