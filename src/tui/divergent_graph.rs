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
        let mut top_max_y: f64 = 0.0;
        let mut bottom_max_y: f64 = 0.0;

        for &v in self.top_data {
            max_x = max_x.max(v.0);
            top_max_y = top_max_y.max(v.1);
        }
        for &v in self.bottom_data {
            max_x = max_x.max(v.0);
            bottom_max_y = bottom_max_y.max(v.1);
        }

        let max_y = top_max_y.max(bottom_max_y).max(1.0);
        let min_x = (max_x - 100.0).max(0.0);
        let graph_height = area.height.saturating_sub(2) as f64;
        let bounds_y = if graph_height > 0.0 {
            let top_rows = if top_max_y >= bottom_max_y {
                (graph_height / 2.0).ceil()
            } else {
                (graph_height / 2.0).floor()
            };
            let resolution_y = graph_height * 4.0;
            let target_zero_idx = top_rows * 4.0;
            let ratio = target_zero_idx / (resolution_y - 1.0);

            let scale = max_y * 1.1;
            let top_bound = scale * ratio * 2.0;
            let bottom_bound = -scale * (1.0 - ratio) * 2.0;
            [bottom_bound, top_bound]
        } else {
            [-max_y * 1.1, max_y * 1.1]
        };

        let negated_bottom: Vec<(f64, f64)> =
            self.bottom_data.iter().map(|&(x, y)| (x, -y)).collect();

        let datasets = vec![
            Dataset::default()
                .name("Top")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Area)
                .style(Style::default().fg(self.top_color))
                .data(self.top_data),
            Dataset::default()
                .name("Bottom")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Area)
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
