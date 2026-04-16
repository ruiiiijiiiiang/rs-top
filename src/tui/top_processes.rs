use ratatui::{
    prelude::*,
    widgets::{
        Block, BorderType, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};

pub struct TopProcesses<'a> {
    pub processes: &'a [String],
    pub scroll: usize,
}

impl<'a> TopProcesses<'a> {
    pub fn new(processes: &'a [String], scroll: usize) -> Self {
        Self { processes, scroll }
    }
}

impl<'a> Widget for TopProcesses<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = self.processes.iter();
        let header = lines.next().map(|s| s.as_str()).unwrap_or("");
        let items: Vec<ListItem> = lines.map(|p| ListItem::new(p.as_str())).collect();

        let process_block = Block::bordered()
            .title(" Top Processes ")
            .border_type(BorderType::Rounded);
        let process_inner = process_block.inner(area);
        process_block.render(area, buf);

        let process_chunks =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(process_inner);

        Paragraph::new(format!("    {}", header)).render(process_chunks[0], buf);

        let state = ListState::default().with_offset(self.scroll);
        let process_list = List::new(items);
        StatefulWidget::render(process_list, process_chunks[1], buf, &mut state.clone());

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        let mut scrollbar_state =
            ScrollbarState::new(self.processes.len().saturating_sub(1)).position(self.scroll);
        scrollbar.render(area, buf, &mut scrollbar_state);
    }
}
