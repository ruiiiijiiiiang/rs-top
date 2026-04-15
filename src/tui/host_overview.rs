use crate::{app::HostState, tui::metric_gauge::MetricGauge, util::format_bytes};
use ratatui::{prelude::*, widgets::*};

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

        let border_type = if self.focused {
            BorderType::Double
        } else {
            BorderType::Rounded
        };

        let block = Block::default()
            .title(format!(" {} ({}) ", host.name, host.status))
            .borders(Borders::ALL)
            .border_type(border_type)
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
                "IP: {} | Uptime: {} | Load: {:.2} {:.2} {:.2}",
                stats.ip_address,
                stats.uptime,
                stats.load_avg.0,
                stats.load_avg.1,
                stats.load_avg.2
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
                let (rx_val, rx_unit) =
                    format_bytes(host.net_rx_rate.last().copied().unwrap_or(0.0));
                let (tx_val, tx_unit) =
                    format_bytes(host.net_tx_rate.last().copied().unwrap_or(0.0));

                let net_load_info = format!(
                    "Net RX: {:.1} {}/s | Net TX: {:.1} {}/s",
                    rx_val, rx_unit, tx_val, tx_unit,
                );
                Paragraph::new(net_load_info).render(inner_layout[4], buf);

                let failed_style = if stats.failed_units.is_empty() {
                    Style::default().fg(Color::Reset)
                } else {
                    Style::default().fg(Color::Red).bold()
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
