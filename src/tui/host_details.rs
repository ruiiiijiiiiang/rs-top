use ratatui::prelude::*;

use crate::{
    app::HostState,
    tui::{
        divergent_graph::DivergentGraph, metric_graph::MetricGraph, top_processes::TopProcesses,
    },
    util::{format_load_avg, format_mem_title, prepare_network_data},
};

pub struct HostDetails<'a> {
    pub host: &'a HostState,
}

impl<'a> HostDetails<'a> {
    pub fn new(host: &'a HostState) -> Self {
        Self { host }
    }
}

impl<'a> Widget for HostDetails<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks =
            Layout::vertical([Constraint::Percentage(45), Constraint::Percentage(55)]).split(area);

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
        let load_avg = self
            .host
            .stats
            .as_ref()
            .map(|s| format!(" ({})", format_load_avg(s.load_avg)))
            .unwrap_or_default();
        MetricGraph::new(&format!("CPU Usage{}", load_avg), &cpu_data).render(top_chunks[0], buf);

        let mem_title = format_mem_title(
            self.host.mem_used.last().copied().unwrap_or(0),
            self.host.mem_total,
        );
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
        MetricGraph::new(&mem_title, &mem_data).render(right_top_chunks[0], buf);

        let net_data = prepare_network_data(&self.host.net_rx_rate, &self.host.net_tx_rate);
        DivergentGraph::new(
            &net_data.title,
            &net_data.rx_data,
            &net_data.tx_data,
            Color::Green,
            Color::Yellow,
        )
        .with_y_labels(net_data.y_labels())
        .render(right_top_chunks[1], buf);

        let processes = self
            .host
            .stats
            .as_ref()
            .map(|stats| stats.processes.as_slice())
            .unwrap_or(&[]);
        TopProcesses::new(processes, self.host.process_scroll).render(chunks[1], buf);
    }
}
