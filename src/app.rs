use std::{error::Error, sync::Arc, time::Duration};

use openssh::{KnownHosts, Session};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    style::palette::tailwind::{self, Palette},
    widgets::*,
};
use tokio::{sync::mpsc, time::interval};

use crate::util::{HostStats, fetch_stats};

const INTERVAL: u64 = 2;

const PALETTES: [Palette; 13] = [
    tailwind::GREEN,
    tailwind::EMERALD,
    tailwind::TEAL,
    tailwind::CYAN,
    tailwind::SKY,
    tailwind::BLUE,
    tailwind::INDIGO,
    tailwind::VIOLET,
    tailwind::PURPLE,
    tailwind::FUCHSIA,
    tailwind::PINK,
    tailwind::ROSE,
    tailwind::RED,
];

pub enum AppAction {
    Input(Event),
    Connected(String, Arc<Session>),
    ConnectionFailed(String, String),
    StatsFetched(String, HostStats),
}

pub struct HostState {
    pub name: String,
    pub status: String,
    pub session: Option<Arc<Session>>,
    pub stats: Option<HostStats>,
    pub prev_cpu_total: u64,
    pub prev_cpu_idle: u64,
    pub cpu_usage: f64,
    pub mem_usage: f64,
    pub disk_usage: f64,
}

pub struct App {
    pub running: bool,
    pub hosts: Vec<HostState>,
}

pub struct MetricGauge<'a> {
    percentage: f64,
    gauge: LineGauge<'a>,
    color: Color,
}

impl<'a> MetricGauge<'a> {
    fn get_colors(percentage: f64) -> (Color, Color) {
        let num_segments = PALETTES.len();
        let segment_index = (percentage / 100.0 * num_segments as f64)
            .floor()
            .min(num_segments as f64 - 1.0) as usize;

        let palette = &PALETTES[segment_index];
        (palette.c500, palette.c900)
    }

    pub fn new(label: &str, percentage: f64) -> Self {
        let (filled_color, unfilled_color) = Self::get_colors(percentage);
        let gauge = LineGauge::default()
            .filled_symbol("⣿")
            .unfilled_symbol("⣿")
            .filled_style(Style::default().fg(filled_color))
            .unfilled_style(Style::default().fg(unfilled_color))
            .ratio(percentage.clamp(0.0, 100.0) / 100.0)
            .label(format!("{}: ", label));
        Self {
            percentage,
            gauge,
            color: filled_color,
        }
    }
}

impl<'a> Widget for MetricGauge<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let gauge_layout = Layout::horizontal([Constraint::Min(0), Constraint::Length(6)]);
        let [gauge_area_1, gauge_area_2] = area.layout(&gauge_layout);
        self.gauge.render(gauge_area_1, buf);

        let percentage_text = format!("{:.1}%", self.percentage);
        let span = Span::styled(percentage_text, Style::default().fg(self.color).bold());
        span.render(gauge_area_2, buf);
    }
}

impl App {
    pub fn new(hosts: Vec<String>) -> Self {
        Self {
            running: true,
            hosts: hosts
                .into_iter()
                .map(|name| HostState {
                    name,
                    status: "Connecting...".to_string(),
                    session: None,
                    stats: None,
                    prev_cpu_total: 0,
                    prev_cpu_idle: 0,
                    cpu_usage: 0.0,
                    mem_usage: 0.0,
                    disk_usage: 0.0,
                })
                .collect(),
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        color_eyre::install()?;
        let terminal = ratatui::init();

        let (tx, rx) = mpsc::channel(100);

        // Spawn connection tasks for each host
        for host in &self.hosts {
            let host_name = host.name.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match Session::connect(&host_name, KnownHosts::Strict).await {
                    Ok(session) => {
                        let session = Arc::new(session);
                        let _ = tx
                            .send(AppAction::Connected(
                                host_name.clone(),
                                Arc::clone(&session),
                            ))
                            .await;

                        // Fetch initial stats immediately after connecting
                        if let Ok(stats) = fetch_stats(session).await {
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

        // Spawn event forwarding task
        let tx_event = tx.clone();
        tokio::spawn(async move {
            loop {
                // Poll for events to avoid blocking indefinitely
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
        // Skip the first tick since it happens immediately and we trigger initial stats in the connection task
        stats_interval.tick().await;

        loop {
            tokio::select! {
                // Receive and process messages from background tasks
                Some(action) = rx.recv() => {
                    self.update(action);
                }
                // Periodic stats fetching
                _ = stats_interval.tick() => {
                    for host in &self.hosts {
                        if let Some(session) = &host.session {
                            let session = Arc::clone(session);
                            let host_name = host.name.clone();
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                if let Ok(stats) = fetch_stats(session).await {
                                    let _ = tx.send(AppAction::StatsFetched(host_name, stats)).await;
                                }
                            });
                        }
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;

            if !self.running {
                break;
            }
        }

        Ok(())
    }

    fn update(&mut self, action: AppAction) {
        match action {
            AppAction::Input(event) => {
                if let Event::Key(key) = event
                    && let KeyCode::Char('q') = key.code
                {
                    self.running = false;
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
                    // CPU calculation
                    if host.prev_cpu_total > 0 {
                        let total_diff = stats.cpu_total.saturating_sub(host.prev_cpu_total);
                        let idle_diff = stats.cpu_idle.saturating_sub(host.prev_cpu_idle);
                        if total_diff > 0 {
                            host.cpu_usage = 100.0 * (1.0 - (idle_diff as f64 / total_diff as f64));
                        }
                    }
                    host.prev_cpu_total = stats.cpu_total;
                    host.prev_cpu_idle = stats.cpu_idle;

                    // Memory calculation
                    if stats.mem_total > 0 {
                        let used = stats.mem_total.saturating_sub(stats.mem_available);
                        host.mem_usage = 100.0 * (used as f64 / stats.mem_total as f64);
                    }

                    // Disk calculation
                    if stats.disk_total > 0 {
                        host.disk_usage =
                            100.0 * (stats.disk_used as f64 / stats.disk_total as f64);
                    }

                    host.stats = Some(stats);
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                self.hosts
                    .iter()
                    .map(|_| Constraint::Length(7))
                    .collect::<Vec<_>>(),
            )
            .split(frame.area());

        for (i, host) in self.hosts.iter().enumerate() {
            let color = match host.status.as_str() {
                "Connected" => Color::Green,
                s if s.starts_with("Failed") => Color::Red,
                _ => Color::Yellow,
            };

            let block = Block::default()
                .title(format!(" {} ", host.name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color));

            let inner_rect = block.inner(chunks[i]);
            frame.render_widget(block, chunks[i]);

            let inner_layout = Layout::vertical([
                Constraint::Length(1), // Info line
                Constraint::Length(1), // CPU Gauge
                Constraint::Length(1), // RAM Gauge
                Constraint::Length(1), // Disk Gauge
                Constraint::Min(0),
            ])
            .split(inner_rect);

            let uptime_info = if let Some(stats) = &host.stats {
                format!("Uptime: {}s", stats.uptime)
            } else {
                "".to_string()
            };

            let info_line = Paragraph::new(format!("Status: {} | {}", host.status, uptime_info));
            frame.render_widget(info_line, inner_layout[0]);

            if host.session.is_some() {
                let cpu_gauge = MetricGauge::new("CPU", host.cpu_usage);
                frame.render_widget(cpu_gauge, inner_layout[1]);

                let mem_gauge = MetricGauge::new("RAM", host.mem_usage);
                frame.render_widget(mem_gauge, inner_layout[2]);

                let disk_gauge = MetricGauge::new("Disk", host.disk_usage);
                frame.render_widget(disk_gauge, inner_layout[3]);
            }
        }
    }
}
