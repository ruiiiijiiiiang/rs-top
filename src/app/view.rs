use ratatui::prelude::*;

use crate::tui::{host_details::HostDetails, host_overview::HostOverviewList};

use super::App;

impl App {
    pub(super) fn draw(&self, frame: &mut Frame) {
        let main_layout =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(frame.area());

        frame.render_widget(
            HostOverviewList::new(&self.hosts, self.focused_host, self.host_scroll),
            main_layout[0],
        );

        if let Some(host) = self.hosts.get(self.focused_host) {
            frame.render_widget(HostDetails::new(host), main_layout[1]);
        }
    }
}
