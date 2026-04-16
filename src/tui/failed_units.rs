use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

pub struct FailedUnits<'a> {
    pub failed_units: &'a [String],
    pub scroll: usize,
}

impl<'a> FailedUnits<'a> {
    pub fn new(failed_units: &'a [String], scroll: usize) -> Self {
        Self {
            failed_units,
            scroll,
        }
    }
}

impl<'a> Widget for FailedUnits<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let failed_units_count = self.failed_units.len();
        let failed_units_block = Block::bordered()
            .title(format!(" Failed Units ({}) ", failed_units_count))
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red));

        let failed_inner = failed_units_block.inner(area);
        failed_units_block.render(area, buf);

        let failed_units_str = self.failed_units.join(", ");
        let content_width = failed_units_str.len();
        let max_scroll = content_width.saturating_sub(failed_inner.width as usize);
        let scroll = self.scroll.min(max_scroll);

        let failed_units_paragraph = Paragraph::new(failed_units_str)
            .style(Style::default().fg(Color::Red))
            .scroll((0, scroll as u16));
        failed_units_paragraph.render(failed_inner, buf);

        if content_width > failed_inner.width as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(Some("◄"))
                .end_symbol(Some("►"));
            let mut scrollbar_state = ScrollbarState::new(content_width)
                .content_length(content_width)
                .viewport_content_length(failed_inner.width as usize)
                .position(scroll);
            scrollbar.render(area, buf, &mut scrollbar_state);
        }
    }
}
