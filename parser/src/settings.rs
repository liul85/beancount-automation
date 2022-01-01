use std::{collections::HashMap, env};

use anyhow::{anyhow, Result};
use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub currency: String,
    pub accounts: HashMap<String, String>,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let mut s = Config::default();
        let config = match env::var("CONFIG") {
            Ok(v) => v,
            Err(_) => return Err(anyhow!("CONFIG env not set!")),
        };

        s.merge(File::from_str(config.as_str(), FileFormat::Toml))?;
        s.try_into().map_err(|e| e.into())
    }
}
