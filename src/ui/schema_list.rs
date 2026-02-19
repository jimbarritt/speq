use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, Pane};
use crate::tree::NodeKind;

pub fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
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
    frame.render_widget(header, area);
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focused_pane == Pane::SchemaList;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let root_count = app.tree.roots.len();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Rounded
        } else {
            BorderType::Plain
        })
        .border_style(border_style)
        .title(format!(" Schemas ({root_count}) "));

    let flat = app.tree.flatten();

    let items: Vec<ListItem> = flat
        .iter()
        .map(|fnode| {
            let node = fnode.node;
            let indent = "  ".repeat(fnode.depth);

            let icon = if node.is_expandable() {
                if node.expanded { "▼" } else { "▶" }
            } else {
                "·"
            };

            let icon_style = if node.is_expandable() {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let req_star = if node.info.required { "*" } else { "" };
            let req_style = Style::default().fg(Color::Yellow);

            // Type label: show format in parens when present
            let type_label = match &node.info.kind {
                NodeKind::Integer | NodeKind::Number | NodeKind::Str => {
                    if let Some(fmt) = &node.info.format {
                        format!("{} ({})", node.type_label(), fmt)
                    } else {
                        node.type_label()
                    }
                }
                _ => node.type_label(),
            };

            let line = Line::from(vec![
                Span::raw(indent),
                Span::styled(icon, icon_style),
                Span::raw(" "),
                Span::raw(node.name.clone()),
                Span::styled(req_star, req_style),
                Span::raw("  "),
                Span::styled(type_label, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.tree.cursor));

    frame.render_stateful_widget(list, area, &mut state);
}
