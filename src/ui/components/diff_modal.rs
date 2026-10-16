use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use super::jump_modal::centered_rect;
use crate::snapshot::{DiffEntry, DiffKind, Snapshot};
use crate::ui::theme::Theme;

pub fn render_diff_modal(
    f: &mut Frame,
    area: Rect,
    snapshots: &[Snapshot],
    active_snap_idx: usize,
    diffs: &[DiffEntry],
    selected_diff_idx: usize,
    theme: &Theme,
) {
    let popup_area = centered_rect(80, 75, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(theme.border_active_style())
        .title(" 󰆓 Point-in-Time Snapshot Diff Engine ")
        .title_style(theme.title_style())
        .style(Style::default().bg(theme.bg_base));

    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(popup_area);

    // Snapshot selector header
    let snap_desc = if let Some(snap) = snapshots.get(active_snap_idx) {
        format!(
            " Comparing against: {} │ Captured: {} │ Total: {} ",
            snap.id,
            format_time(snap.created_at),
            format_bytes(snap.total_size)
        )
    } else {
        " No snapshots saved yet. Press [p] in Explorer to capture a snapshot! ".to_string()
    };

    let snap_block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(Style::default().fg(theme.sapphire));
    let snap_p = Paragraph::new(snap_desc)
        .style(Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD))
        .block(snap_block);
    f.render_widget(snap_p, inner[0]);

    // Diff entries list
    let list_items: Vec<ListItem> = diffs
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_sel = idx == selected_diff_idx;
            let (badge, badge_style) = match entry.kind {
                DiffKind::Added => (" [+] ADD  ", Style::default().bg(theme.green).fg(theme.bg_crust).add_modifier(Modifier::BOLD)),
                DiffKind::Removed => (" [-] DEL  ", Style::default().bg(theme.red).fg(theme.bg_crust).add_modifier(Modifier::BOLD)),
                DiffKind::Grown => (" [▲] GROW ", Style::default().bg(theme.peach).fg(theme.bg_crust).add_modifier(Modifier::BOLD)),
                DiffKind::Shrunk => (" [▼] SHRK ", Style::default().bg(theme.teal).fg(theme.bg_crust).add_modifier(Modifier::BOLD)),
                DiffKind::Unchanged => (" [=] SAME ", Style::default().fg(theme.overlay)),
            };

            let delta_str = if entry.size_delta > 0 {
                format!("+{}", format_bytes(entry.size_delta as u64))
            } else if entry.size_delta < 0 {
                format!("-{}", format_bytes(entry.size_delta.unsigned_abs()))
            } else {
                "0 B".to_string()
            };

            let prefix = if is_sel { " ❯ " } else { "   " };
            let spans = vec![
                Span::styled(prefix, if is_sel { Style::default().fg(theme.mauve) } else { theme.dim_style() }),
                Span::styled(badge, badge_style),
                Span::styled(
                    format!("  {:<35} ", entry.relative_path.to_string_lossy()),
                    if is_sel { Style::default().fg(theme.text).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.subtext) },
                ),
                Span::styled(
                    format!(" {:>12} (was {})", delta_str, format_bytes(entry.old_size)),
                    Style::default().fg(theme.size_color(entry.new_size)),
                ),
            ];

            let mut item = ListItem::new(Line::from(spans));
            if is_sel {
                item = item.style(theme.selected_row_style());
            }
            item
        })
        .collect();

    let diff_title = format!(" Changes Detected ({}) │ [Tab] Switch Snapshot │ [Esc] Close ", diffs.len());
    let diff_block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(Style::default().fg(theme.surface1))
        .title(diff_title)
        .title_style(Style::default().fg(theme.subtext));

    let list = List::new(list_items).block(diff_block);
    f.render_widget(list, inner[1]);
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

fn format_time(secs: u64) -> String {
    use chrono::{Local, TimeZone};
    if let chrono::LocalResult::Single(dt) = Local.timestamp_opt(secs as i64, 0) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "-".to_string()
    }
}
