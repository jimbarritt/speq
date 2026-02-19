use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, _app: &App, area: Rect) {
    let hints = " j/k navigate  ·  Tab switch pane  ·  q quit";
    let bar = Paragraph::new(hints).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(bar, area);
}
