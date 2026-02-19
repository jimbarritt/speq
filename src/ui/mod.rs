use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::App;

mod detail;
mod schema_list;
mod statusbar;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Vertical split: header (1) / body (fill) / footer (1)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    crate::ui::schema_list::draw_header(frame, app, outer[0]);

    // Body: left pane (35%) / right pane (65%)
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[1]);

    schema_list::draw(frame, app, body[0]);
    detail::draw(frame, app, body[1]);

    // Status bar
    statusbar::draw(frame, app, outer[2]);
}
