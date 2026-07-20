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

        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Area)
                .style(Style::default().fg(Color::Reset))
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

        let plot_top = area.top() + 1;
        let plot_bottom = area.bottom().saturating_sub(1);
        if plot_bottom > plot_top {
            let height = (plot_bottom - plot_top) as f64;
            for y in plot_top..plot_bottom {
                let row_from_bottom = plot_bottom - 1 - y;
                let pct = (row_from_bottom as f64 + 0.5) / height * 100.0;
                let fg_color = util::get_palette(pct).c500;

                for x in area.left()..area.right() {
                    let cell = &mut buf[(x, y)];
                    if is_braille_symbol(cell.symbol()) {
                        cell.set_fg(fg_color);
                    }
                }
            }
        }
    }
}

fn is_braille_symbol(sym: &str) -> bool {
    if sym.is_empty() || sym == " " {
        return false;
    }
    if let Some(ch) = sym.chars().next() {
        // Braille patterns Unicode block: U+2800 to U+28FF
        // U+2800 is the blank braille pattern '⠀'
        ('\u{2801}'..='\u{28FF}').contains(&ch)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, prelude::Rect, widgets::Widget};

    use super::MetricGraph;
    use crate::util;

    #[test]
    fn graph_renders_without_panic() {
        let data = vec![(0.0, 10.0), (1.0, 50.0), (2.0, 90.0)];
        let graph = MetricGraph::new("Test CPU", &data);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));

        graph.render(Rect::new(0, 0, 30, 10), &mut buf);
    }

    #[test]
    fn graph_applies_tiered_colors_by_row() {
        let data = vec![(0.0, 100.0), (100.0, 100.0)];
        let graph = MetricGraph::new("CPU", &data);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));

        graph.render(Rect::new(0, 0, 30, 10), &mut buf);

        let bottom_plot_cell_fg = buf[(15, 8)].fg;
        assert_eq!(bottom_plot_cell_fg, util::PALETTES.first().unwrap().c500);

        let top_plot_cell_fg = buf[(15, 1)].fg;
        assert_eq!(top_plot_cell_fg, util::PALETTES.last().unwrap().c500);

        let plot_top = 1;
        let plot_bottom = 9;
        let height = (plot_bottom - plot_top) as f64;
        for y in plot_top..plot_bottom {
            let row_from_bottom = plot_bottom - 1 - y;
            let pct = (row_from_bottom as f64 + 0.5) / height * 100.0;
            let expected_color = util::get_palette(pct).c500;
            assert_eq!(buf[(15, y)].fg, expected_color);
        }
    }
}
