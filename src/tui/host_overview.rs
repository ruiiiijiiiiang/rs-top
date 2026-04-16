use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    app::HostState,
    tui::metric_gauge::MetricGauge,
    util::{format_load_avg, format_rate},
};

pub struct HostOverview<'a> {
    pub host: &'a HostState,
    pub focused: bool,
}

impl<'a> HostOverview<'a> {
    pub fn new(host: &'a HostState, focused: bool) -> Self {
        Self { host, focused }
    }
}

impl<'a> Widget for HostOverview<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let host = self.host;
        let color = match host.status.as_str() {
            "Connected" => Color::Green,
            s if s.starts_with("Failed") => Color::Red,
            _ => Color::Yellow,
        };

        let style = if self.focused {
            Style::new().on_dark_gray().bold().italic()
        } else {
            Style::new()
        };

        let block = Block::bordered()
            .title(format!(" {} ({}) ", host.name, host.status))
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
