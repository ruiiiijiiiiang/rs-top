use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};

pub struct MetricGraph<'a> {
    title: String,
    data: &'a [(f64, f64)],
    color: Color,
    y_labels: Vec<String>,
}

impl<'a> MetricGraph<'a> {
    pub fn new(title: &str, data: &'a [(f64, f64)], color: Color) -> Self {
        Self {
            title: title.to_string(),
            data,
            color,
            y_labels: vec!["0%".to_string(), "50%".to_string(), "100%".to_string()],
        }
    }

    pub fn with_y_labels(mut self, labels: Vec<String>) -> Self {
        self.y_labels = labels;
        self
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

        let labels: Vec<Span> = self
            .y_labels
            .iter()
            .map(|l| Span::styled(l, Style::default().fg(Color::Gray)))
            .collect();

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" {} ", self.title),
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
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
                    .labels(labels),
            );

        chart.render(area, buf);
    }
}
