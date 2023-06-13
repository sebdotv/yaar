use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fs, str};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub profiles: IndexMap<String, Profile>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// output_key -> edid
    pub outputs: IndexMap<String, String>,
    /// output_key -> output_mode
    pub setup: IndexMap<String, OutputMode>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum OutputMode {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "primary")]
    Primary,
    #[serde(rename = "secondary")]
    Secondary,
}

pub fn load_config() -> Config {
    let cfg_str = fs::read_to_string("config.yaml").expect("failed to read YAML config file");
    let cfg: Config = serde_yaml::from_str(&cfg_str).expect("failed to parse config");
    cfg
}
