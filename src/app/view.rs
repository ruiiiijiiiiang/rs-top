use ratatui::prelude::*;

use crate::{
    app::DisplayMode,
    tui::{host_details::HostDetails, host_overview::HostOverviewList},
};

use super::App;

impl App {
    pub(super) fn draw(&self, frame: &mut Frame) {
        let constraints = match self.display_mode {
            DisplayMode::Standard => [Constraint::Percentage(40), Constraint::Percentage(60)],
            DisplayMode::Compact => [Constraint::Length(22), Constraint::Min(0)],
        };

        let main_layout = Layout::horizontal(constraints).split(frame.area());

        frame.render_widget(
            HostOverviewList::new(
                &self.hosts,
                self.focused_host,
                self.host_scroll,
                self.display_mode,
            ),
            main_layout[0],
        );

        if let Some(host) = self.hosts.get(self.focused_host) {
            frame.render_widget(HostDetails::new(host), main_layout[1]);
        }
    }
}
