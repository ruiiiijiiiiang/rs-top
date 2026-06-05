use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Block, BorderType, Chart, Dataset, GraphType},
};

use crate::util;

pub struct MetricGraph<'a> {
    title: String,
    data: &'a [(f64, f64)],
}

impl<'a> MetricGraph<'a> {
    pub fn new(title: &str, data: &'a [(f64, f64)]) -> Self {
        Self {
            title: title.to_string(),
            data,
        }
    }
}

impl<'a> Widget for MetricGraph<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_x = if self.data.is_empty() {
            100.0
        } else {
            self.data.last().unwrap().0.max(100.0)
        };
        let min_x = (max_x - 100.0).max(0.0);

        let last_val = self.data.last().map(|&(_, v)| v).unwrap_or(0.0);
        let color = util::get_palette(last_val).c500;

        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Area)
                .style(Style::default().fg(color))
                .data(self.data),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::bordered()
                    .title(format!(" {} ", self.title))
                    .border_type(BorderType::Rounded),
            )
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Reset))
                    .bounds([min_x, max_x]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 100.0])
                    .labels(["0%", "50%", "100%"]),
            )
            .legend_position(None);

        chart.render(area, buf);
    }
}
