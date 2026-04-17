use ratatui::prelude::*;

use crate::{
    app::HostState,
    tui::{
        divergent_graph::DivergentGraph, failed_units::FailedUnits, metric_graph::MetricGraph,
        top_processes::TopProcesses,
    },
    util::{NetworkData, format_bytes, format_load_avg, format_rate, prepare_network_data},
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

        let failed_units_count = self
            .host
            .stats
            .as_ref()
            .map(|s| s.failed_units.len())
            .unwrap_or(0);

        let (cpu_area, failed_area) = if failed_units_count > 0 {
            let chunks =
                Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(top_chunks[0]);
            (chunks[0], Some(chunks[1]))
        } else {
            (top_chunks[0], None)
        };

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
        let cpu_graph = MetricGraph::new(&format!("CPU Usage{}", load_avg), &cpu_data, Color::Cyan);
        cpu_graph.render(cpu_area, buf);

        if let Some(failed_area) = failed_area {
            let failed_units = self
                .host
                .stats
                .as_ref()
                .map(|s| s.failed_units.as_slice())
                .unwrap_or(&[]);
            FailedUnits::new(failed_units, self.host.failed_units_scroll).render(failed_area, buf);
        }

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

        let NetworkData(rx_data, tx_data, max_net) =
            prepare_network_data(&self.host.net_rx_rate, &self.host.net_tx_rate);

        let rx_latest = self.host.net_rx_rate.last().copied().unwrap_or(0.0);
        let tx_latest = self.host.net_tx_rate.last().copied().unwrap_or(0.0);
        let net_title = format!(
            "Net RX: {} TX: {}",
            format_rate(rx_latest),
            format_rate(tx_latest)
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

        let processes = self
            .host
            .stats
            .as_ref()
            .map(|stats| stats.processes.as_slice())
            .unwrap_or(&[]);
        TopProcesses::new(processes, self.host.process_scroll).render(chunks[1], buf);
    }
}
