use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::Deserialize;
use crate::thirdparty::wiz;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub wiz: wiz::Config,
}

impl Config {
    pub fn parse<T: AsRef<Path>>(path: T) -> anyhow::Result<Self> {
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;
        let config:Self = toml::from_str(&content)?;

        Ok(config)
    }
}