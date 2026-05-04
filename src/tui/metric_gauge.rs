use ratatui::prelude::*;
use ratatui::widgets::LineGauge;

use crate::util;

pub struct MetricGauge<'a> {
    label: String,
    percentage: f64,
    gauge: LineGauge<'a>,
    color: Color,
}

impl<'a> MetricGauge<'a> {
    fn get_colors(percentage: f64) -> (Color, Color) {
        let palette = util::get_palette(percentage);
        (palette.c500, palette.c900)
    }

    pub fn new(label: &str, percentage: f64) -> Self {
        let (filled_color, unfilled_color) = Self::get_colors(percentage);
        let gauge = LineGauge::default()
            .filled_symbol("⣿")
            .unfilled_symbol("⡇")
            .filled_style(Style::default().fg(filled_color))
            .unfilled_style(Style::default().fg(unfilled_color))
            .ratio(percentage.clamp(0.0, 100.0) / 100.0)
            .label("");
        Self {
            label: label.to_string(),
            percentage,
            gauge,
            color: filled_color,
        }
    }
}

impl<'a> Widget for MetricGauge<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let gauge_layout = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(6),
        ]);
        let [label_area, gauge_area, percentage_area] = area.layout(&gauge_layout);

        let label_span = format!("{}:", self.label);
        label_span.render(label_area, buf);

        self.gauge.render(gauge_area, buf);

        let span = Span::styled(
            format!("{:.1}%", self.percentage),
            Style::default().fg(self.color),
        );
        span.render(percentage_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, prelude::Rect, widgets::Widget};

    use super::MetricGauge;

    #[test]
    fn gauge_renders_label_and_percentage_text() {
        let gauge = MetricGauge::new("CPU", 42.5);
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 1));

        gauge.render(Rect::new(0, 0, 24, 1), &mut buf);

        let rendered: String = (0..24).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(rendered.contains("CPU:"));
        assert!(rendered.contains("42.5%"));
    }
}
