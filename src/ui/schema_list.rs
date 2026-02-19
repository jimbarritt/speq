use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, Pane};

pub fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " speq  ·  {}  ·  {} ",
        app.spec.title,
        app.spec.version.label()
    );
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " speq ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  {}  ·  {}",
                app.spec.title,
                app.spec.version.label()
            ),
            Style::default().fg(Color::Gray),
        ),
    ]));
    let _ = title; // suppress unused warning
    frame.render_widget(header, area);
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focused_pane == Pane::SchemaList;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(" Schemas ({}) ", app.spec.schema_names.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Rounded
        } else {
            BorderType::Plain
        })
        .border_style(border_style)
        .title(title);

    let items: Vec<ListItem> = app
        .spec
        .schema_names
        .iter()
        .map(|name| ListItem::new(format!(" ▶ {}", name)))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.selected));

    frame.render_stateful_widget(list, area, &mut state);
}
