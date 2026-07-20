use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};

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
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => self.running = false,
                KeyCode::Enter => {
                    self.pending_ssh_host = self
                        .hosts
                        .get(self.focused_host)
                        .and_then(|host| host.config.clone());
                    self.running = false;
                }
                KeyCode::Tab if !self.hosts.is_empty() => {
                    self.focused_host = (self.focused_host + 1) % self.hosts.len();
                }
                KeyCode::BackTab if !self.hosts.is_empty() => {
                    if self.focused_host == 0 {
                        self.focused_host = self.hosts.len() - 1;
                    } else {
                        self.focused_host -= 1;
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

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    };

    use super::*;
    use crate::{HostConfig, app::HostState, remote::host_stats::HostStats};

    fn make_host(address: &str) -> HostState {
        HostState {
            name: address.to_string(),
            config: Some(HostConfig {
                user: Some("alice".to_string()),
                port: Some(22),
                identity_file: Some("/tmp/id_ed25519".to_string()),
                address: address.to_string(),
            }),
            ..Default::default()
        }
    }

    fn key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn tab_and_backtab_wrap_focused_host() {
        let mut app = App {
            running: true,
            hosts: vec![
                make_host("host-a"),
                make_host("host-b"),
                make_host("host-c"),
            ],
            ..Default::default()
        };

        app.update(AppAction::Input(key_event(KeyCode::Tab)));
        assert_eq!(app.focused_host, 1);

        app.update(AppAction::Input(key_event(KeyCode::Tab)));
        assert_eq!(app.focused_host, 2);

        app.update(AppAction::Input(key_event(KeyCode::Tab)));
        assert_eq!(app.focused_host, 0);

        app.update(AppAction::Input(key_event(KeyCode::BackTab)));
        assert_eq!(app.focused_host, 2);
    }

    #[test]
    fn enter_sets_pending_ssh_host_and_stops_app() {
        let mut app = App {
            running: true,
            hosts: vec![make_host("host-a"), make_host("host-b")],
            focused_host: 1,
            ..Default::default()
        };

        app.update(AppAction::Input(key_event(KeyCode::Enter)));

        assert!(!app.running);
        assert_eq!(
            app.pending_ssh_host
                .as_ref()
                .map(|host| host.address.as_str()),
            Some("host-b")
        );
    }

    #[test]
    fn key_release_events_are_ignored() {
        let mut app = App {
            running: true,
            hosts: vec![make_host("host-a"), make_host("host-b")],
            ..Default::default()
        };
        let release = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        });

        app.update(AppAction::Input(release));

        assert!(app.running);
        assert!(app.pending_ssh_host.is_none());
    }

    #[test]
    fn display_mode_toggles_with_m_key() {
        let mut app = App {
            display_mode: DisplayMode::Standard,
            ..Default::default()
        };

        app.update(AppAction::Input(key_event(KeyCode::Char('m'))));
        assert_eq!(app.display_mode, DisplayMode::Compact);

        app.update(AppAction::Input(key_event(KeyCode::Char('m'))));
        assert_eq!(app.display_mode, DisplayMode::Standard);
    }

    #[test]
    fn stats_fetched_updates_history_and_latest_stats() {
        let mut app = App {
            hosts: vec![HostState {
                name: "host-a".to_string(),
                prev_cpu_total: 100,
                prev_cpu_idle: 40,
                prev_net_rx: 1_000,
                prev_net_tx: 2_000,
                ..make_host("host-a")
            }],
            ..Default::default()
        };
        let stats = HostStats {
            mem_total: 4_000,
            mem_available: 1_000,
            cpu_total: 220,
            cpu_idle: 70,
            disk_total: 10_000,
            disk_used: 2_500,
            net_rx: 1_600,
            net_tx: 2_800,
            processes: vec!["header".to_string(), "proc".to_string()],
            ..Default::default()
        };

        app.update(AppAction::StatsFetched("host-a".to_string(), stats.clone()));

        let host = &app.hosts[0];
        assert_eq!(host.mem_total, 4_000);
        assert_eq!(host.mem_used, vec![3_000]);
        assert_eq!(host.disk_total, 10_000);
        assert_eq!(host.disk_used, 2_500);
        assert_eq!(host.net_rx_rate, vec![300.0]);
        assert_eq!(host.net_tx_rate, vec![400.0]);
        assert_eq!(host.prev_cpu_total, 220);
        assert_eq!(host.prev_cpu_idle, 70);
        assert_eq!(host.prev_net_rx, 1_600);
        assert_eq!(host.prev_net_tx, 2_800);
        assert_eq!(host.stats.as_ref().unwrap().processes, stats.processes);
        assert_eq!(host.cpu_usage.len(), 1);
        assert!((host.cpu_usage[0] - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn process_scrolling_respects_process_list_bounds() {
        let mut app = App {
            hosts: vec![HostState {
                name: "host-a".to_string(),
                stats: Some(HostStats {
                    processes: vec![
                        "header".to_string(),
                        "proc-1".to_string(),
                        "proc-2".to_string(),
                    ],
                    ..Default::default()
                }),
                ..make_host("host-a")
            }],
            ..Default::default()
        };

        app.update(AppAction::Input(key_event(KeyCode::Down)));
        app.update(AppAction::Input(key_event(KeyCode::Char('j'))));
        app.update(AppAction::Input(key_event(KeyCode::Down)));
        assert_eq!(app.hosts[0].process_scroll, 2);

        app.update(AppAction::Input(key_event(KeyCode::Up)));
        app.update(AppAction::Input(key_event(KeyCode::Char('k'))));
        app.update(AppAction::Input(key_event(KeyCode::Up)));
        assert_eq!(app.hosts[0].process_scroll, 0);
    }
}
