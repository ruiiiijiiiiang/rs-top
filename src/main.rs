use std::{fs, process};

use serde::Deserialize;

use crate::app::App;

pub mod app;
pub mod host_stats;
pub mod tui;
pub mod util;

#[derive(Deserialize, Clone, Debug)]
pub struct HostConfig {
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
    pub address: String,
}

#[derive(Deserialize)]
struct Config {
    hosts: Vec<HostConfig>,
}

#[tokio::main]
async fn main() {
    let config_path = dirs::config_dir()
        .map(|mut p| {
            p.push("rs-top.toml");
            p
        })
        .expect("Could not determine config directory");

    let config_str = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        eprintln!("Error reading config file {:?}: {}", config_path, e);
        process::exit(1);
    });

    let config: Config = toml::from_str(&config_str).unwrap_or_else(|e| {
        eprintln!("Error parsing config file: {}", e);
        process::exit(1);
    });

    if config.hosts.is_empty() {
        eprintln!("Error: No hosts found in config file.");
        process::exit(1);
    }

    let _ = App::new(config.hosts).start().await;
    process::exit(0);
}
