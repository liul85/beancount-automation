use std::{collections::HashMap, env};

use anyhow::Result;
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
        s.merge(File::from_str(
            env::var("CONFIG").unwrap().as_str(),
            FileFormat::Toml,
        ))
        .unwrap();
        s.try_into().map_err(|e| e.into())
    }
}
