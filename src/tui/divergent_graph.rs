use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Block, BorderType, Chart, Dataset, GraphType},
};

pub struct DivergentGraph<'a> {
    title: String,
    top_data: &'a [(f64, f64)],
    bottom_data: &'a [(f64, f64)],
    top_color: Color,
    bottom_color: Color,
    y_labels: Vec<String>,
}

impl<'a> DivergentGraph<'a> {
    pub fn new(
        title: &str,
        top_data: &'a [(f64, f64)],
        bottom_data: &'a [(f64, f64)],
        top_color: Color,
        bottom_color: Color,
    ) -> Self {
        Self {
            title: title.to_string(),
            top_data,
            bottom_data,
            top_color,
            bottom_color,
            y_labels: vec![],
        }
    }

    pub fn with_y_labels(mut self, labels: Vec<String>) -> Self {
        self.y_labels = labels;
        self
    }
}

impl<'a> Widget for DivergentGraph<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut max_x: f64 = 100.0;
        let mut max_y: f64 = 1.0;

        for &v in self.top_data {
            max_x = max_x.max(v.0);
            max_y = max_y.max(v.1);
        }
        for &v in self.bottom_data {
            max_x = max_x.max(v.0);
            max_y = max_y.max(v.1);
        }

        let min_x = (max_x - 100.0).max(0.0);
        let bounds_y = [-max_y * 1.1, max_y * 1.1];

        let negated_bottom: Vec<(f64, f64)> =
            self.bottom_data.iter().map(|&(x, y)| (x, -y)).collect();

        let datasets = vec![
            Dataset::default()
                .name("Top")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(self.top_color))
                .data(self.top_data),
            Dataset::default()
                .name("Bottom")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(self.bottom_color))
                .data(&negated_bottom),
        ];

        let mut y_axis = Axis::default()
            .style(Style::default().fg(Color::Gray))
            .bounds(bounds_y);

        if !self.y_labels.is_empty() {
            let labels: Vec<Span> = self
                .y_labels
                .iter()
                .map(|l| Span::styled(l, Style::default().fg(Color::Gray)))
                .collect();
            y_axis = y_axis.labels(labels);
        }

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
            .y_axis(y_axis)
            .legend_position(None);

        chart.render(area, buf);
    }
}
