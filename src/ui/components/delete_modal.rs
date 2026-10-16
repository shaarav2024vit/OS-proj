use std::path::Path;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use super::jump_modal::centered_rect;
use crate::ui::theme::Theme;

pub fn render_delete_modal(
    f: &mut Frame,
    area: Rect,
    target_path: &Path,
    is_dir: bool,
    reclaimed_bytes: u64,
    theme: &Theme,
) {
    let popup_area = centered_rect(55, 30, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(Style::default().fg(theme.red))
        .title(" 󰆴 Confirm Space Reclamation (Delete) ")
        .title_style(Style::default().fg(theme.red).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg_base));

    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(4), Constraint::Length(2)])
        .split(popup_area);

    let filename = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let item_type = if is_dir { "directory and all contents" } else { "file" };

    let text = vec![
        Line::from(vec![
            Span::styled("Are you sure you want to delete this ", Style::default().fg(theme.text)),
            Span::styled(item_type, Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD)),
            Span::styled("?", Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(theme.subtext)),
            Span::styled(filename, Style::default().fg(theme.red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Space to be freed: ", Style::default().fg(theme.subtext)),
            Span::styled(format_bytes(reclaimed_bytes), Style::default().fg(theme.green).add_modifier(Modifier::BOLD)),
        ]),
    ];

    f.render_widget(Paragraph::new(text), inner[0]);

    let footer = Line::from(vec![
        Span::styled(" [y] ", Style::default().fg(theme.red).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm Delete    ", Style::default().fg(theme.subtext)),
        Span::styled(" [n / Esc] ", Style::default().fg(theme.green).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", Style::default().fg(theme.subtext)),
    ]);

    f.render_widget(Paragraph::new(footer), inner[1]);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
