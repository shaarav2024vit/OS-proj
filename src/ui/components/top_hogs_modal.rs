use std::path::PathBuf;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use super::jump_modal::centered_rect;
use crate::model::DirectoryTree;
use crate::ui::icons::get_file_icon;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct SpaceHogEntry {
    pub path: PathBuf,
    pub size: u64,
    pub parent_id: Option<usize>,
}

pub fn render_top_hogs_modal(
    f: &mut Frame,
    area: Rect,
    hogs: &[SpaceHogEntry],
    selected_index: usize,
    theme: &Theme,
) {
    let popup_area = centered_rect(75, 70, area);
    f.render_widget(Clear, popup_area);

    let max_size = hogs.first().map(|h| h.size).unwrap_or(1).max(1);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(theme.border_active_style())
        .title(" 󰆓 Top 20 Space Hogs (Largest Files Across Project) ")
        .title_style(theme.title_style())
        .style(Style::default().bg(theme.bg_base));

    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let header_text = format!(
        " Identified {} massive files across scanned storage: ",
        hogs.len()
    );
    let header_p = Paragraph::new(header_text).style(
        Style::default()
            .fg(theme.mauve)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header_p, inner[0]);

    let list_items: Vec<ListItem> = hogs
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_sel = idx == selected_index;
            let rank = format!("#{:02} ", idx + 1);

            let icon = get_file_icon(&entry.path, false, false);
            let path_str = entry.path.to_string_lossy();

            let ratio = (entry.size as f64) / (max_size as f64);
            let (bar_str, bar_style) = theme.format_usage_bar(ratio, 14);

            let prefix = if is_sel { " ❯ " } else { "   " };

            let spans = vec![
                Span::styled(
                    prefix,
                    if is_sel {
                        Style::default().fg(theme.mauve)
                    } else {
                        theme.dim_style()
                    },
                ),
                Span::styled(rank, Style::default().fg(theme.overlay)),
                Span::styled(format!("{} ", icon.glyph), Style::default().fg(icon.color)),
                Span::styled(
                    format!("{:<38} ", truncate(&path_str, 36)),
                    if is_sel {
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.subtext)
                    },
                ),
                Span::styled(bar_str, bar_style),
                Span::styled(
                    format!(" {:>10}", format_bytes(entry.size)),
                    Style::default()
                        .fg(theme.size_color(entry.size))
                        .add_modifier(Modifier::BOLD),
                ),
            ];

            let mut item = ListItem::new(Line::from(spans));
            if is_sel {
                item = item.style(theme.selected_row_style());
            }
            item
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(Style::default().fg(theme.surface1));

    let list = List::new(list_items).block(list_block);
    f.render_widget(list, inner[1]);

    let footer = Line::from(vec![
        Span::styled(
            " [Enter] ",
            Style::default()
                .fg(theme.green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Jump to Folder   ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [Esc/q] ",
            Style::default().fg(theme.red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Close", Style::default().fg(theme.subtext)),
    ]);
    f.render_widget(Paragraph::new(footer), inner[2]);
}

pub fn collect_space_hogs(tree: &DirectoryTree, limit: usize) -> Vec<SpaceHogEntry> {
    let mut files: Vec<SpaceHogEntry> = tree
        .nodes
        .iter()
        .filter(|n| !n.is_dir())
        .map(|n| SpaceHogEntry {
            path: n.path.clone(),
            size: n.disk_size,
            parent_id: n.parent,
        })
        .collect();

    files.sort_by_key(|a| std::cmp::Reverse(a.size));
    files.truncate(limit);
    files
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

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("…{}", &s[s.len().saturating_sub(max_len)..])
    }
}
