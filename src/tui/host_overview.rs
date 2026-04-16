use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    app::{ConnectionStatus, HostState},
    tui::metric_gauge::MetricGauge,
    util::{format_load_avg, format_rate},
};

pub struct HostOverview<'a> {
    pub host: &'a HostState,
    pub focused: bool,
}

pub struct HostOverviewList<'a> {
    pub hosts: &'a [HostState],
    pub focused_host: usize,
    pub host_scroll: usize,
}

impl<'a> HostOverview<'a> {
    pub fn new(host: &'a HostState, focused: bool) -> Self {
        Self { host, focused }
    }
}

impl<'a> HostOverviewList<'a> {
    pub const ITEM_HEIGHT: u16 = 8;

    pub fn new(hosts: &'a [HostState], focused_host: usize, host_scroll: usize) -> Self {
        Self {
            hosts,
            focused_host,
            host_scroll,
        }
    }
}

impl<'a> Widget for HostOverview<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let host = self.host;
        let (status_label, color) = match host.connection_status {
            ConnectionStatus::Connecting => ("Connecting", Color::Yellow),
            ConnectionStatus::Connected => ("Connected", Color::Green),
            ConnectionStatus::Failed => ("Failed", Color::Red),
        };

        let style = if self.focused {
            Style::new().on_dark_gray().bold().italic()
        } else {
            Style::new()
        };

        let block = Block::bordered()
            .title(format!(" {} ({}) ", host.name, status_label))
            .style(style)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color));

        let inner_rect = block.inner(area);
        block.render(area, buf);

        let inner_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner_rect);

        let host_info = if let Some(stats) = &host.stats {
            format!(
                "IP: {} | Uptime: {} | Load: {}",
                stats.ip_address,
                stats.uptime,
                format_load_avg(stats.load_avg)
            )
        } else {
            "".to_string()
        };

        Paragraph::new(host_info).render(inner_layout[0], buf);

        if host.session.is_some() {
            let cpu_usage = host.cpu_usage.last().copied().unwrap_or(0.0);
            MetricGauge::new("CPU", cpu_usage).render(inner_layout[1], buf);

            let mem_used = host.mem_used.last().copied().unwrap_or(0);
            let mem_usage = if host.mem_total > 0 {
                100.0 * (mem_used as f64 / host.mem_total as f64)
            } else {
                0.0
            };
            MetricGauge::new("RAM", mem_usage).render(inner_layout[2], buf);

            let disk_usage = if host.disk_total > 0 {
                100.0 * (host.disk_used as f64 / host.disk_total as f64)
            } else {
                0.0
            };
            MetricGauge::new("Disk", disk_usage).render(inner_layout[3], buf);

            if let Some(stats) = &host.stats {
                let rx_rate = host.net_rx_rate.last().copied().unwrap_or(0.0);
                let tx_rate = host.net_tx_rate.last().copied().unwrap_or(0.0);

                let net_load_info = format!(
                    "Net RX: {} | Net TX: {}",
                    format_rate(rx_rate),
                    format_rate(tx_rate),
                );
                Paragraph::new(net_load_info).render(inner_layout[4], buf);

                let failed_style = if stats.failed_units.is_empty() {
                    Style::default().fg(Color::Reset)
                } else {
                    Style::default().fg(Color::Red)
                };
                Paragraph::new(Span::styled(
                    format!("Failed Units: {}", stats.failed_units.len()),
                    failed_style,
                ))
                .render(inner_layout[5], buf);
            }
        }
    }
}

impl<'a> Widget for HostOverviewList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let visible_count = (area.height as usize / Self::ITEM_HEIGHT as usize).max(1);
        let mut scroll = self.host_scroll;
        if self.focused_host < scroll {
            scroll = self.focused_host;
        } else if self.focused_host >= scroll + visible_count {
            scroll = self.focused_host - visible_count + 1;
        }

        let has_scrollbar = self.hosts.len() > visible_count;
        let list_area = if has_scrollbar {
            Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).split(area)[0]
        } else {
            area
        };

        let chunks = Layout::vertical(
            (0..visible_count)
                .map(|_| Constraint::Length(Self::ITEM_HEIGHT))
                .collect::<Vec<_>>(),
        )
        .split(list_area);

        for (i, chunk) in chunks.iter().enumerate() {
            let host_idx = scroll + i;
            if let Some(host) = self.hosts.get(host_idx) {
                let focused = host_idx == self.focused_host;
                HostOverview::new(host, focused).render(*chunk, buf);
            }
        }

        if has_scrollbar {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));
            let mut scrollbar_state = ScrollbarState::new(self.hosts.len()).position(scroll);
            scrollbar.render(area, buf, &mut scrollbar_state);
        }
    }
}
