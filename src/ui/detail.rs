use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::{App, Pane};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focused_pane == Pane::Detail;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let selected_name = app.selected_schema_name().unwrap_or("—");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Rounded
        } else {
            BorderType::Plain
        })
        .border_style(border_style)
        .title(format!(" {} ", selected_name));

    let content = vec![
        Line::from(vec![
            Span::styled("  Schema  ", Style::default().fg(Color::DarkGray)),
            Span::styled(selected_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Select a schema on the left to inspect it.", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}
