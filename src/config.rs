use std::path::{Path, PathBuf};
use std::{env, fs, str};

use indexmap::IndexMap;
use log::{debug, info};
use serde::{Deserialize, Serialize};

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
    let config_root_dir = get_config_root_dir();
    debug!("config_root_dir: {:?}", config_root_dir);

    let config_file = config_root_dir.join("yaar").join("config.yaml");
    info!("Loading user config from file: {:?}", config_file);

    let cfg_str = fs::read_to_string(config_file).expect("failed to read YAML config file");
    let cfg: Config = serde_yml::from_str(&cfg_str).expect("failed to parse config");
    cfg
}

fn get_config_root_dir() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME") // start with $XDG_CONFIG_HOME
        .and_then(|x| (!x.is_empty()).then_some(x)) // replace empty string with None
        .map(|x| Path::new(x.as_os_str()).to_owned()) // convert to PathBuf
        .unwrap_or_else(|| {
            // or fallback to $HOME/.config
            let home = env::var_os("HOME").expect("HOME env var not set");
            Path::new(home.as_os_str()).join(".config")
        })
}
