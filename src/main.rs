use std::{
    fs,
    process::{self, Command, ExitStatus},
};

use serde::Deserialize;

use crate::app::App;

pub mod app;
pub mod remote;
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

    let exit_code = match App::new(config.hosts).start().await {
        Ok(Some(host)) => match launch_ssh_session(&host) {
            Ok(status) => status.code().unwrap_or(1),
            Err(err) => {
                eprintln!("Error starting SSH session: {err}");
                1
            }
        },
        Ok(None) => 0,
        Err(err) => {
            eprintln!("Application error: {err:?}");
            1
        }
    };

    process::exit(exit_code);
}

fn launch_ssh_session(host: &HostConfig) -> std::io::Result<ExitStatus> {
    let mut command = Command::new("ssh");

    if let Some(port) = host.port {
        command.arg("-p").arg(port.to_string());
    }

    if let Some(identity_file) = &host.identity_file
        && !identity_file.is_empty()
    {
        command.arg("-i").arg(identity_file);
    }

    let destination = match &host.user {
        Some(user) if !user.is_empty() => format!("{user}@{}", host.address),
        _ => host.address.clone(),
    };

    command.arg(destination).status()
}
