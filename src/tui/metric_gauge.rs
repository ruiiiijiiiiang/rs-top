use ratatui::prelude::*;

use crate::util;

pub struct MetricGauge<'a> {
    label: String,
    percentage: f64,
    color: Color,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> MetricGauge<'a> {
    pub fn new(label: &str, percentage: f64) -> Self {
        let color = util::get_palette(percentage).c500;
        Self {
            label: label.to_string(),
            percentage,
            color,
            _phantom: std::marker::PhantomData,
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

        if gauge_area.width > 0 && gauge_area.height > 0 {
            let width = gauge_area.width as usize;
            let ratio = (self.percentage.clamp(0.0, 100.0) / 100.0).min(1.0);
            let filled_cols = (ratio * width as f64).round() as usize;

            for i in 0..width {
                let col_pct = (i as f64) / (width as f64) * 100.0;
                let palette = util::get_palette(col_pct);

                let (symbol, fg_color) = if i < filled_cols {
                    ("⣿", palette.c500)
                } else {
                    ("⡇", palette.c900)
                };

                let cell_x = gauge_area.left() + i as u16;
                let cell_y = gauge_area.top();
                buf[(cell_x, cell_y)]
                    .set_symbol(symbol)
                    .set_fg(fg_color);
            }
        }

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

