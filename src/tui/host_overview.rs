use ratatui::{
    prelude::*,
    style::palette::tailwind,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    app::{ConnectionStatus, DisplayMode, HostState},
    tui::metric_gauge::MetricGauge,
    util::{format_load_avg, format_rate, get_palette},
};

pub struct HostOverview<'a> {
    pub host: &'a HostState,
    pub focused: bool,
    pub display_mode: DisplayMode,
}

pub struct HostOverviewList<'a> {
    pub hosts: &'a [HostState],
    pub focused_host: usize,
    pub host_scroll: usize,
    pub display_mode: DisplayMode,
}

impl<'a> HostOverview<'a> {
    pub fn new(host: &'a HostState, focused: bool, display_mode: DisplayMode) -> Self {
        Self {
            host,
            focused,
            display_mode,
        }
    }
}

impl<'a> HostOverviewList<'a> {
    pub fn new(
        hosts: &'a [HostState],
        focused_host: usize,
        host_scroll: usize,
        display_mode: DisplayMode,
    ) -> Self {
        Self {
            hosts,
            focused_host,
            host_scroll,
            display_mode,
        }
    }

    pub fn item_height(&self) -> u16 {
        match self.display_mode {
            DisplayMode::Standard => 8,
            DisplayMode::Compact => 3,
        }
    }
}

impl<'a> Widget for HostOverview<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let host = self.host;
        let (status_label, color) = match host.connection_status {
            ConnectionStatus::Connecting => ("Connecting", tailwind::YELLOW),
            ConnectionStatus::Connected => ("Connected", tailwind::GREEN),
            ConnectionStatus::Failed => ("Failed", tailwind::RED),
        };

        let title = match self.display_mode {
            DisplayMode::Standard => {
                if self.focused {
                    format!(
                        " {} ({}) | Tab: ▼ | Shift+Tab: ▲ | Enter: Launch SSH ",
                        host.name, status_label
                    )
                } else {
                    format!(" {} ({}) ", host.name, status_label)
                }
            }
            DisplayMode::Compact => format!(" {} ", host.name),
        };

        let block = Block::bordered()
            .title(title)
            .border_type(BorderType::Rounded)
            .border_style(if self.focused {
                Style::default().fg(color.c800).on_white().bold().italic()
            } else {
                Style::default().fg(color.c500)
            });

        let inner_rect = block.inner(area);
        block.render(area, buf);

        match self.display_mode {
            DisplayMode::Standard => {
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

                        let failed_count = stats.failed_units.len();
                        let failed_style = if failed_count == 0 {
                            Style::default().fg(Color::Reset)
                        } else {
                            Style::default().fg(Color::Red)
                        };

                        let failed_units_str = stats.failed_units.join(", ");
                        let mut failed_text =
                            format!("Failed Units ({}): {}", failed_count, failed_units_str);

                        let width = inner_layout[5].width as usize;
                        if failed_text.chars().count() > width && width > 3 {
                            failed_text = failed_text.chars().take(width - 3).collect();
                            failed_text.push_str("...");
                        }

                        Paragraph::new(Span::styled(failed_text, failed_style))
                            .render(inner_layout[5], buf);
                    }
                }
            }
            DisplayMode::Compact => {
                let cpu_usage = host.cpu_usage.last().copied().unwrap_or(0.0);
                let mem_used = host.mem_used.last().copied().unwrap_or(0);
                let mem_usage = if host.mem_total > 0 {
                    100.0 * (mem_used as f64 / host.mem_total as f64)
                } else {
                    0.0
                };
                let disk_usage = if host.disk_total > 0 {
                    100.0 * (host.disk_used as f64 / host.disk_total as f64)
                } else {
                    0.0
                };

                let cpu_color = get_palette(cpu_usage).c500;
                let mem_color = get_palette(mem_usage).c500;
                let disk_color = get_palette(disk_usage).c500;

                let stats_line = Line::from(vec![
                    Span::raw("C:"),
                    Span::styled(format!("{:.0}%", cpu_usage), Style::default().fg(cpu_color)),
                    Span::raw(" R:"),
                    Span::styled(format!("{:.0}%", mem_usage), Style::default().fg(mem_color)),
                    Span::raw(" D:"),
                    Span::styled(
                        format!("{:.0}%", disk_usage),
                        Style::default().fg(disk_color),
                    ),
                ]);
                Paragraph::new(stats_line).render(inner_rect, buf);
            }
        }
    }
}

impl<'a> Widget for HostOverviewList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let item_height = self.item_height();
        let visible_count = (area.height as usize / item_height as usize).max(1);
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
                .map(|_| Constraint::Length(item_height))
                .collect::<Vec<_>>(),
        )
        .split(list_area);

        for (i, chunk) in chunks.iter().enumerate() {
            let host_idx = scroll + i;
            if let Some(host) = self.hosts.get(host_idx) {
                let focused = host_idx == self.focused_host;
                HostOverview::new(host, focused, self.display_mode).render(*chunk, buf);
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

#[cfg(test)]
mod tests {
    use super::HostOverviewList;
    use crate::app::{DisplayMode, HostState};

    #[test]
    fn item_height_matches_display_mode() {
        let hosts = vec![HostState::default()];

        let standard = HostOverviewList::new(&hosts, 0, 0, DisplayMode::Standard);
        let compact = HostOverviewList::new(&hosts, 0, 0, DisplayMode::Compact);

        assert_eq!(standard.item_height(), 8);
        assert_eq!(compact.item_height(), 3);
    }
}
