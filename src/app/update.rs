use ratatui::crossterm::event::{Event, KeyCode};

use super::{App, AppAction, ConnectionStatus, DisplayMode, INTERVAL, MAX_HISTORY};

impl App {
    pub(super) fn update(&mut self, action: AppAction) {
        match action {
            AppAction::Input(event) => self.handle_input(event),
            AppAction::Connected(name, session) => {
                if let Some(host) = self.hosts.iter_mut().find(|h| h.name == name) {
                    host.connection_status = ConnectionStatus::Connected;
                    host.session = Some(session);
                }
            }
            AppAction::ConnectionFailed(name, _err) => {
                if let Some(host) = self.hosts.iter_mut().find(|h| h.name == name) {
                    host.connection_status = ConnectionStatus::Failed;
                }
            }
            AppAction::StatsFetched(name, stats) => {
                if let Some(host) = self.hosts.iter_mut().find(|h| h.name == name) {
                    if host.prev_cpu_total > 0 {
                        let total_diff = stats.cpu_total.saturating_sub(host.prev_cpu_total);
                        let idle_diff = stats.cpu_idle.saturating_sub(host.prev_cpu_idle);
                        if total_diff > 0 {
                            let usage = 100.0 * (1.0 - (idle_diff as f64 / total_diff as f64));
                            push_history(&mut host.cpu_usage, usage, MAX_HISTORY);
                        }
                    }
                    host.prev_cpu_total = stats.cpu_total;
                    host.prev_cpu_idle = stats.cpu_idle;

                    host.mem_total = stats.mem_total;
                    let used = stats.mem_total.saturating_sub(stats.mem_available);
                    push_history(&mut host.mem_used, used, MAX_HISTORY);

                    host.disk_total = stats.disk_total;
                    host.disk_used = stats.disk_used;

                    if host.prev_net_rx > 0 {
                        let rx_rate = (stats.net_rx.saturating_sub(host.prev_net_rx)) as f64
                            / INTERVAL as f64;
                        let tx_rate = (stats.net_tx.saturating_sub(host.prev_net_tx)) as f64
                            / INTERVAL as f64;
                        push_history(&mut host.net_rx_rate, rx_rate, MAX_HISTORY);
                        push_history(&mut host.net_tx_rate, tx_rate, MAX_HISTORY);
                    }
                    host.prev_net_rx = stats.net_rx;
                    host.prev_net_tx = stats.net_tx;

                    host.stats = Some(stats);
                }
            }
        }
    }

    fn handle_input(&mut self, event: Event) {
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
                KeyCode::Char('m') => {
                    self.display_mode = match self.display_mode {
                        DisplayMode::Standard => DisplayMode::Compact,
                        DisplayMode::Compact => DisplayMode::Standard,
                    };
                }
                _ => {}
            }
        }
    }
}

fn push_history<T>(values: &mut Vec<T>, value: T, max_len: usize) {
    values.push(value);
    if values.len() > max_len {
        values.remove(0);
    }
}
