use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

use crate::model::{DirectoryTree, NodeId, SortOrder};
use crate::ui::icons::get_file_icon;
use crate::ui::theme::Theme;

#[allow(clippy::too_many_arguments)]
pub fn render_file_table(
    f: &mut Frame,
    area: Rect,
    tree: &DirectoryTree,
    current_dir_id: NodeId,
    sorted_children: &[NodeId],
    selected_index: usize,
    sort_order: SortOrder,
    filter_query: &str,
    theme: &Theme,
    table_state: &mut TableState,
) {
    let current_dir = tree.get_node(current_dir_id);
    let total_dir_size = current_dir.map(|d| d.disk_size).unwrap_or(1).max(1);

    let sort_label = match sort_order {
        SortOrder::SizeDesc => "Size ▼",
        SortOrder::SizeAsc => "Size ▲",
        SortOrder::NameAsc => "Name ▲",
        SortOrder::NameDesc => "Name ▼",
        SortOrder::CountDesc => "Items ▼",
        SortOrder::ModifiedDesc => "Time ▼",
    };

    let title = if filter_query.is_empty() {
        format!(" 󰉋 Explorer [{}] ", sort_label)
    } else {
        format!(" 󰉋 Explorer [{}] (Filter: '{}') ", sort_label, filter_query)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(theme.border_style())
        .title(title)
        .title_style(theme.title_style());

    let header_cells = [
        "",
        " Name",
        "Usage Bar",
        "%",
        "Disk Size",
        "Items",
        "Modified",
    ]
    .iter()
    .map(|&h| {
        Cell::from(h).style(
            Style::default()
                .fg(theme.mauve)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells)
        .style(Style::default().bg(theme.bg_mantle))
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = sorted_children
        .iter()
        .enumerate()
        .map(|(idx, &child_id)| {
            let node = match tree.get_node(child_id) {
                Some(n) => n,
                None => return Row::new(vec![Cell::from("")]),
            };

            let is_selected = idx == selected_index;
            let prefix = if is_selected { " ❯ " } else { "   " };

            let file_icon = get_file_icon(&node.path, node.is_dir(), node.is_encrypted);

            let name_style = if node.is_encrypted {
                theme.encrypted_style()
            } else if node.is_dir() {
                theme.directory_style()
            } else {
                theme.file_style()
            };

            let icon_span = file_icon.glyph;
            let display_name = format!("{}{}{}", prefix, icon_span, node.name);

            // Usage bar calculation relative to current directory total
            let ratio = (node.disk_size as f64) / (total_dir_size as f64);
            let bar_width = 14;
            let (bar_str, bar_style) = theme.format_usage_bar(ratio, bar_width);

            let pct_str = format!("{:>5.1}%", ratio * 100.0);
            let size_str = format_bytes(node.disk_size);
            let size_style = Style::default().fg(theme.size_color(node.disk_size));

            let items_str = if node.is_dir() {
                format!("{}", node.item_count)
            } else {
                "-".to_string()
            };

            let mod_str = format_timestamp(node.modified);

            let row_cells = vec![
                Cell::from(""),
                Cell::from(display_name).style(name_style),
                Cell::from(bar_str).style(bar_style),
                Cell::from(pct_str).style(Style::default().fg(theme.subtext)),
                Cell::from(size_str).style(size_style),
                Cell::from(items_str).style(Style::default().fg(theme.subtext)),
                Cell::from(mod_str)
                    .style(Style::default().fg(theme.dim_style().fg.unwrap_or(Color::DarkGray))),
            ];

            let mut row = Row::new(row_cells).height(1);
            if is_selected {
                row = row.style(theme.selected_row_style());
            }
            row
        })
        .collect();

    let widths = [
        Constraint::Length(1),
        Constraint::Percentage(36),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Length(11),
        Constraint::Length(8),
        Constraint::Min(11),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(1)
        .row_highlight_style(Style::default()); // We handle highlight manually per-row

    if !sorted_children.is_empty() {
        table_state.select(Some(selected_index));
    } else {
        table_state.select(None);
    }

    f.render_stateful_widget(table, area, table_state);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:>6.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:>6.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:>6.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:>6.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:>6} B ", bytes)
    }
}

fn format_timestamp(secs: u64) -> String {
    if secs == 0 {
        return "-".to_string();
    }
    use chrono::{Local, TimeZone};
    if let chrono::LocalResult::Single(dt) = Local.timestamp_opt(secs as i64, 0) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        "-".to_string()
    }
}
