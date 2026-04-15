use crate::{
    app::HostState,
    tui::{divergent_graph::DivergentGraph, metric_graph::MetricGraph},
    util::format_bytes,
};
use ratatui::{prelude::*, widgets::*};

pub struct HostDetail<'a> {
    pub host: &'a HostState,
}

impl<'a> HostDetail<'a> {
    pub fn new(host: &'a HostState) -> Self {
        Self { host }
    }
}

impl<'a> Widget for HostDetail<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks =
            Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

        let top_chunks =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(chunks[0]);

        let right_top_chunks =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(top_chunks[1]);

        let cpu_data: Vec<(f64, f64)> = self
            .host
            .cpu_usage
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect();
        let cpu_graph = MetricGraph::new("CPU Usage", &cpu_data, Color::Cyan);
        cpu_graph.render(top_chunks[0], buf);

        let mem_total_gb = self.host.mem_total as f64 / (1024.0 * 1024.0);
        let mem_used_latest_gb =
            self.host.mem_used.last().copied().unwrap_or(0) as f64 / (1024.0 * 1024.0);
        let mem_title = format!("RAM {:.1}G/{:.1}G", mem_used_latest_gb, mem_total_gb);

        let mem_data: Vec<(f64, f64)> = self
            .host
            .mem_used
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let percentage = if self.host.mem_total > 0 {
                    100.0 * (v as f64 / self.host.mem_total as f64)
                } else {
                    0.0
                };
                (i as f64, percentage)
            })
            .collect();
        let mem_graph = MetricGraph::new(&mem_title, &mem_data, Color::Magenta);
        mem_graph.render(right_top_chunks[0], buf);

        let rx_data: Vec<(f64, f64)> = self
            .host
            .net_rx_rate
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect();
        let tx_data: Vec<(f64, f64)> = self
            .host
            .net_tx_rate
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect();

        let mut max_net: f64 = 1.0;
        for &(_, y) in &rx_data {
            max_net = max_net.max(y);
        }
        for &(_, y) in &tx_data {
            max_net = max_net.max(y);
        }

        let (rx_latest, rx_unit) =
            format_bytes(self.host.net_rx_rate.last().copied().unwrap_or(0.0));
        let (tx_latest, tx_unit) =
            format_bytes(self.host.net_tx_rate.last().copied().unwrap_or(0.0));
        let net_title = format!(
            "Net RX: {:.1}{} TX: {:.1}{}",
            rx_latest, rx_unit, tx_latest, tx_unit
        );

        let (max_val, max_unit) = format_bytes(max_net);

        let net_graph =
            DivergentGraph::new(&net_title, &rx_data, &tx_data, Color::Green, Color::Yellow)
                .with_y_labels(vec![
                    format!("{:.1}{}", max_val, max_unit),
                    "0".to_string(),
                    format!("{:.1}{}", max_val, max_unit),
                ]);
        net_graph.render(right_top_chunks[1], buf);

        let (header, items) = if let Some(stats) = &self.host.stats {
            let mut lines = stats.processes.iter();
            let header = lines.next().map(|s| s.as_str()).unwrap_or("");
            let items: Vec<ListItem> = lines.map(|p| ListItem::new(p.as_str())).collect();
            (header, items)
        } else {
            ("", vec![])
        };

        let process_block = Block::default()
            .title(" Top Processes ")
            .borders(Borders::ALL);
        let process_inner = process_block.inner(chunks[1]);
        process_block.render(chunks[1], buf);

        let process_chunks =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(process_inner);

        Paragraph::new(header)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .render(process_chunks[0], buf);

        let state = ListState::default().with_offset(self.host.process_scroll);
        let process_list = List::new(items);

        StatefulWidget::render(process_list, process_chunks[1], buf, &mut state.clone());

        if let Some(stats) = &self.host.stats {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state = ScrollbarState::new(stats.processes.len().saturating_sub(1))
                .position(self.host.process_scroll);
            scrollbar.render(chunks[1], buf, &mut scrollbar_state);
        }
    }
}
