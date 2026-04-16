use std::sync::Arc;

use openssh::Session;
use ratatui::crossterm::event::Event;

use crate::{HostConfig, remote::host_stats::HostStats};

pub enum AppAction {
    Input(Event),
    Connected(String, Arc<Session>),
    ConnectionFailed(String, String),
    StatsFetched(String, HostStats),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct HostState {
    pub name: String,
    pub config: Option<HostConfig>,
    pub connection_status: ConnectionStatus,
    pub session: Option<Arc<Session>>,
    pub stats: Option<HostStats>,
    pub prev_cpu_total: u64,
    pub prev_cpu_idle: u64,
    pub cpu_usage: Vec<f64>,
    pub mem_total: u64,
    pub mem_used: Vec<u64>,
    pub disk_total: u64,
    pub disk_used: u64,
    pub prev_net_rx: u64,
    pub prev_net_tx: u64,
    pub net_rx_rate: Vec<f64>,
    pub net_tx_rate: Vec<f64>,
    pub process_scroll: usize,
    pub failed_units_scroll: usize,
}

#[derive(Debug, Clone, Default)]
pub struct App {
    pub running: bool,
    pub hosts: Vec<HostState>,
    pub focused_host: usize,
    pub host_scroll: usize,
}
