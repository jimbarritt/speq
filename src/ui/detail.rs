use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::{App, Pane};
use crate::tree::{NodeKind, TreeNode};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focused_pane == Pane::Detail;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Rounded
        } else {
            BorderType::Plain
        })
        .border_style(border_style)
        .title(" Detail ");

    match app.tree.selected_node() {
        None => {
            let paragraph = Paragraph::new("  No schema selected.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            frame.render_widget(paragraph, area);
        }
        Some(node) => {
            let content = build_detail_lines(node);
            let paragraph = Paragraph::new(content)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((app.detail_scroll, 0));
            frame.render_widget(paragraph, area);
        }
    }
}

// ── detail content builder ────────────────────────────────────────────────────

const KEY_WIDTH: usize = 14; // left column width for key labels

fn kv_line(key: &str, value_spans: Vec<Span<'static>>) -> Line<'static> {
    let mut spans = vec![
        Span::raw("  "),
        Span::styled(
            format!("{:<KEY_WIDTH$}", key),
            Style::default().fg(Color::DarkGray),
        ),
    ];
    spans.extend(value_spans);
    Line::from(spans)
}

fn kv_str(key: &str, value: impl Into<String>) -> Line<'static> {
    kv_line(
        key,
        vec![Span::styled(
            value.into(),
            Style::default().fg(Color::White),
        )],
    )
}

fn separator() -> Line<'static> {
    Line::from(Span::styled(
        format!("  {}", "─".repeat(60)),
        Style::default().fg(Color::DarkGray),
    ))
}

fn build_detail_lines(node: &TreeNode) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // ── Header: name + type ──────────────────────────────────────────────────
    let type_label = node.type_label();
    let format_suffix = match &node.info.format {
        Some(f) => format!(" ({})", f),
        None => String::new(),
    };

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            node.name.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}{}", type_label, format_suffix),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    lines.push(separator());

    // ── Required ─────────────────────────────────────────────────────────────
    if node.info.required {
        lines.push(kv_line(
            "required",
            vec![Span::styled("yes", Style::default().fg(Color::Yellow))],
        ));
    }

    // ── Description ──────────────────────────────────────────────────────────
    if let Some(desc) = &node.info.description {
        lines.push(kv_str("description", desc.clone()));
    }

    // ── Kind-specific fields ─────────────────────────────────────────────────
    match &node.info.kind {
        NodeKind::Object => {
            let prop_count = node.children.len();
            if prop_count > 0 {
                lines.push(kv_str("properties", prop_count.to_string()));
            }
            let required_props: Vec<String> = node
                .children
                .iter()
                .filter(|c| c.info.required)
                .map(|c| c.name.clone())
                .collect();
            if !required_props.is_empty() {
                lines.push(kv_str("required", required_props.join(" · ")));
            }
        }

        NodeKind::Array => {
            if let Some(items) = node.children.first() {
                lines.push(kv_line(
                    "items",
                    vec![Span::styled(
                        items.type_label(),
                        Style::default().fg(Color::Cyan),
                    )],
                ));
            }
        }

        NodeKind::Ref(target) => {
            lines.push(kv_line(
                "reference",
                vec![Span::styled(
                    format!("→{}", target),
                    Style::default().fg(Color::Cyan),
                )],
            ));
        }

        NodeKind::AllOf => {
            lines.push(kv_str("combiner", format!("allOf ({} schemas)", node.children.len())));
        }
        NodeKind::OneOf => {
            lines.push(kv_str("combiner", format!("oneOf ({} schemas)", node.children.len())));
        }
        NodeKind::AnyOf => {
            lines.push(kv_str("combiner", format!("anyOf ({} schemas)", node.children.len())));
        }

        _ => {}
    }

    // ── Constraints ───────────────────────────────────────────────────────────
    for constraint in &node.info.constraints {
        lines.push(kv_str("constraint", constraint.clone()));
    }

    // ── Enum values ───────────────────────────────────────────────────────────
    if !node.info.enum_values.is_empty() {
        lines.push(kv_str("enum", node.info.enum_values.join(" · ")));
    }

    // ── Default / Example ─────────────────────────────────────────────────────
    if let Some(default) = &node.info.default_val {
        lines.push(kv_str("default", default.clone()));
    }
    if let Some(example) = &node.info.example {
        lines.push(kv_str("example", example.clone()));
    }

    // Trailing blank line for breathing room
    lines.push(Line::from(""));

    lines
}
