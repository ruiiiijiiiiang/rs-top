use ratatui::{
    prelude::*,
    style::palette::tailwind::{self, Palette},
    widgets::*,
};

const PALETTES: [Palette; 7] = [
    tailwind::EMERALD,
    tailwind::GREEN,
    tailwind::LIME,
    tailwind::YELLOW,
    tailwind::ORANGE,
    tailwind::RED,
    tailwind::PINK,
];

pub struct MetricGauge<'a> {
    label: String,
    percentage: f64,
    gauge: LineGauge<'a>,
    color: Color,
}

impl<'a> MetricGauge<'a> {
    fn get_colors(percentage: f64) -> (Color, Color) {
        let num_segments = PALETTES.len();
        let segment_index = (percentage / 100.0 * num_segments as f64)
            .floor()
            .min(num_segments as f64 - 1.0) as usize;

        let palette = &PALETTES[segment_index];
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

        let label_span = Span::styled(format!("{}:", self.label), Style::default().bold());
        label_span.render(label_area, buf);

        self.gauge.render(gauge_area, buf);

        let span = Span::styled(
            format!("{:.1}%", self.percentage),
            Style::default().fg(self.color).bold(),
        );
        span.render(percentage_area, buf);
    }
}
