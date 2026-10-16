use std::path::PathBuf;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::ui::theme::Theme;

pub fn render_jump_modal(
    f: &mut Frame,
    area: Rect,
    query: &str,
    results: &[(PathBuf, i64, Vec<usize>)],
    selected_index: usize,
    theme: &Theme,
) {
    // Center popup Rect
    let popup_area = centered_rect(65, 55, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(theme.border_active_style())
        .title(" 󰀝 Frecency Quick-Jump (Zoxide) ")
        .title_style(theme.title_style())
        .style(Style::default().bg(theme.bg_base));

    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(popup_area);

    // Search query box
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(Style::default().fg(theme.mauve))
        .title(" 󰍉 Search visited directories ");

    let input_text = format!(" ❯ {}█", query);
    let input_p = Paragraph::new(input_text)
        .style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
        .block(input_block);
    f.render_widget(input_p, inner[0]);

    // Results list
    let list_items: Vec<ListItem> = results
        .iter()
        .enumerate()
        .map(|(idx, (path, score, indices))| {
            let is_sel = idx == selected_index;
            let path_str = path.to_string_lossy();

            let mut spans = Vec::new();
            let prefix = if is_sel { " ❯ " } else { "   " };
            spans.push(Span::styled(
                prefix,
                if is_sel {
                    Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD)
                } else {
                    theme.dim_style()
                },
            ));

            // Highlight matching character indices in Mauve
            for (char_idx, ch) in path_str.chars().enumerate() {
                if indices.contains(&char_idx) {
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default()
                            .fg(theme.mauve)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                    ));
                } else {
                    spans.push(Span::styled(
                        ch.to_string(),
                        if is_sel {
                            Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.subtext)
                        },
                    ));
                }
            }

            spans.push(Span::styled(
                format!("  [{}]", score),
                Style::default().fg(theme.overlay),
            ));

            let item = ListItem::new(Line::from(spans));
            if is_sel {
                item.style(theme.selected_row_style())
            } else {
                item
            }
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::NONE)
        .title(format!(" Matches ({}) ", results.len()))
        .title_style(Style::default().fg(theme.subtext));

    let list = List::new(list_items).block(list_block);
    f.render_widget(list, inner[1]);
}

/// Helper to create a centered Rect for modals
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
