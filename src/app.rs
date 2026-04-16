use std::{error::Error, sync::Arc, time::Duration};

use openssh::{KnownHosts, Session, SessionBuilder};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use tokio::{sync::mpsc, time::interval};

use crate::{
    HostConfig,
    host_stats::HostStats,
    tui::{host_details::HostDetails, host_overview::HostOverview},
};

const INTERVAL: u64 = 2;
const MAX_HISTORY: usize = 200;

pub enum AppAction {
    Input(Event),
    Connected(String, Arc<Session>),
    ConnectionFailed(String, String),
    StatsFetched(String, HostStats),
}

#[derive(Debug, Clone, Default)]
pub struct HostState {
    pub name: String,
    pub config: Option<HostConfig>,
    pub status: String,
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

pub struct App {
    pub running: bool,
    pub hosts: Vec<HostState>,
    pub focused_host: usize,
    pub host_scroll: usize,
}

impl App {
    pub fn new(hosts: Vec<HostConfig>) -> Self {
        let current_user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        Self {
            running: true,
            hosts: hosts
                .into_iter()
                .map(|config| {
                    let user = config.user.as_deref().unwrap_or(&current_user);
                    let port = config.port.unwrap_or(22);
                    HostState {
                        name: format!("{}@{}:{}", user, config.address, port),
                        config: Some(config),
                        status: "Connecting...".to_string(),
                        ..Default::default()
                    }
                })
                .collect(),
            focused_host: 0,
            host_scroll: 0,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        color_eyre::install()?;
        let terminal = ratatui::init();

        let (tx, rx) = mpsc::channel(100);

        let current_user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

        for host in &self.hosts {
            let host_name = host.name.clone();
            let config = host.config.clone().unwrap();
            let tx = tx.clone();
            let default_user = current_user.clone();

            tokio::spawn(async move {
                let mut builder = SessionBuilder::default();

                let user = config.user.unwrap_or(default_user);
                builder.user(user);

                builder.port(config.port.unwrap_or(22));

                if let Some(identity_file) = config.identity_file
                    && !identity_file.is_empty()
                {
                    builder.keyfile(identity_file);
                }

                builder.known_hosts_check(KnownHosts::Strict);

                let session_res = builder.connect(&config.address).await;

                match session_res {
                    Ok(session) => {
                        let session = Arc::new(session);
                        let _ = tx
                            .send(AppAction::Connected(
                                host_name.clone(),
                                Arc::clone(&session),
                            ))
                            .await;

                        if let Ok(stats) = HostStats::fetch(session).await {
                            let _ = tx.send(AppAction::StatsFetched(host_name, stats)).await;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(AppAction::ConnectionFailed(host_name, e.to_string()))
                            .await;
                    }
                }
            });
        }

        let tx_event = tx.clone();
        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(100)).unwrap_or(false)
                    && let Ok(ev) = event::read()
                    && tx_event.send(AppAction::Input(ev)).await.is_err()
                {
                    break;
                }
            }
        });

        let res = self.run(terminal, tx, rx).await;

        ratatui::restore();

        res
    }

    async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        tx: mpsc::Sender<AppAction>,
        mut rx: mpsc::Receiver<AppAction>,
    ) -> Result<(), Box<dyn Error>> {
        let mut stats_interval = interval(Duration::from_secs(INTERVAL));
        stats_interval.tick().await;

        loop {
            tokio::select! {
                Some(action) = rx.recv() => {
                    self.update(action);
                }
                _ = stats_interval.tick() => {
                    for host in &self.hosts {
                        if let Some(session) = &host.session {
                            let session = Arc::clone(session);
                            let host_name = host.name.clone();
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                if let Ok(stats) = HostStats::fetch(session).await {
                                    let _ = tx.send(AppAction::StatsFetched(host_name, stats)).await;
                                }
                            });
                        }
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;

            if !self.running {
                for host in &mut self.hosts {
                    host.session = None;
                }
                break;
            }
        }

        Ok(())
    }

    fn update(&mut self, action: AppAction) {
        match action {
            AppAction::Input(event) => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Char('q') => self.running = false,
                        KeyCode::Tab => {
                            if !self.hosts.is_empty() {
                                self.focused_host = (self.focused_host + 1) % self.hosts.len();
                            }
                        }
                        KeyCode::BackTab => {
                            if !self.hosts.is_empty() {
                                if self.focused_host == 0 {
                                    self.focused_host = self.hosts.len() - 1;
                                } else {
                                    self.focused_host -= 1;
                                }
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if let Some(host) = self.hosts.get_mut(self.focused_host)
                                && let Some(stats) = &host.stats
                                && host.process_scroll < stats.processes.len().saturating_sub(1)
                            {
                                host.process_scroll = host.process_scroll.saturating_add(1);
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if let Some(host) = self.hosts.get_mut(self.focused_host) {
                                host.process_scroll = host.process_scroll.saturating_sub(1);
                            }
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            if let Some(host) = self.hosts.get_mut(self.focused_host) {
                                host.failed_units_scroll =
                                    host.failed_units_scroll.saturating_sub(1);
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if let Some(host) = self.hosts.get_mut(self.focused_host) {
                                host.failed_units_scroll =
                                    host.failed_units_scroll.saturating_add(1);
                            }
                        }
                        _ => {}
                    }
                }
            }
            AppAction::Connected(name, session) => {
                if let Some(host) = self.hosts.iter_mut().find(|h| h.name == name) {
                    host.status = "Connected".to_string();
                    host.session = Some(session);
                }
            }
            AppAction::ConnectionFailed(name, err) => {
                if let Some(host) = self.hosts.iter_mut().find(|h| h.name == name) {
                    host.status = format!("Failed: {}", err);
                }
            }
            AppAction::StatsFetched(name, stats) => {
                if let Some(host) = self.hosts.iter_mut().find(|h| h.name == name) {
                    if host.prev_cpu_total > 0 {
                        let total_diff = stats.cpu_total.saturating_sub(host.prev_cpu_total);
                        let idle_diff = stats.cpu_idle.saturating_sub(host.prev_cpu_idle);
                        if total_diff > 0 {
                            let usage = 100.0 * (1.0 - (idle_diff as f64 / total_diff as f64));
                            host.cpu_usage.push(usage);
                            if host.cpu_usage.len() > MAX_HISTORY {
                                host.cpu_usage.remove(0);
                            }
                        }
                    }
                    host.prev_cpu_total = stats.cpu_total;
                    host.prev_cpu_idle = stats.cpu_idle;

                    host.mem_total = stats.mem_total;
                    let used = stats.mem_total.saturating_sub(stats.mem_available);
                    host.mem_used.push(used);
                    if host.mem_used.len() > MAX_HISTORY {
                        host.mem_used.remove(0);
                    }

                    host.disk_total = stats.disk_total;
                    host.disk_used = stats.disk_used;

                    if host.prev_net_rx > 0 {
                        let rx_rate = (stats.net_rx.saturating_sub(host.prev_net_rx)) as f64
                            / INTERVAL as f64;
                        let tx_rate = (stats.net_tx.saturating_sub(host.prev_net_tx)) as f64
                            / INTERVAL as f64;
                        host.net_rx_rate.push(rx_rate);
                        if host.net_rx_rate.len() > MAX_HISTORY {
                            host.net_rx_rate.remove(0);
                        }
                        host.net_tx_rate.push(tx_rate);
                        if host.net_tx_rate.len() > MAX_HISTORY {
                            host.net_tx_rate.remove(0);
                        }
                    }
                    host.prev_net_rx = stats.net_rx;
                    host.prev_net_tx = stats.net_tx;

                    host.stats = Some(stats);
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let main_layout =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(frame.area());

        let host_area = main_layout[0];
        let item_height = 8;
        let visible_count = (host_area.height as usize / item_height).max(1);

        let mut scroll = self.host_scroll;
        if self.focused_host < scroll {
            scroll = self.focused_host;
        } else if self.focused_host >= scroll + visible_count {
            scroll = self.focused_host - visible_count + 1;
        }

        let has_scrollbar = self.hosts.len() > visible_count;
        let list_area = if has_scrollbar {
            Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).split(host_area)[0]
        } else {
            host_area
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..visible_count)
                    .map(|_| Constraint::Length(item_height as u16))
                    .collect::<Vec<_>>(),
            )
            .split(list_area);

        for i in 0..visible_count {
            let host_idx = scroll + i;
            if let Some(host) = self.hosts.get(host_idx) {
                let focused = host_idx == self.focused_host;
                frame.render_widget(HostOverview::new(host, focused), chunks[i]);
            }
        }

        if has_scrollbar {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));
            let mut scrollbar_state = ScrollbarState::new(self.hosts.len()).position(scroll);
            frame.render_stateful_widget(scrollbar, host_area, &mut scrollbar_state);
        }

        if let Some(host) = self.hosts.get(self.focused_host) {
            frame.render_widget(HostDetails::new(host), main_layout[1]);
        }
    }
}
