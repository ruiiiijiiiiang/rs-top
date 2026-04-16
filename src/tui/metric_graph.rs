use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Block, BorderType, Chart, Dataset, GraphType},
};

pub struct MetricGraph<'a> {
    title: String,
    data: &'a [(f64, f64)],
    color: Color,
}

impl<'a> MetricGraph<'a> {
    pub fn new(title: &str, data: &'a [(f64, f64)], color: Color) -> Self {
        Self {
            title: title.to_string(),
            data,
            color,
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

        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(self.color))
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
            );

        chart.render(area, buf);
    }
}
