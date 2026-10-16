use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use super::jump_modal::centered_rect;
use crate::model::DirectoryTree;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct CategoryStat {
    pub name: &'static str,
    pub icon: &'static str,
    pub color: Color,
    pub total_size: u64,
    pub file_count: usize,
}

pub fn render_breakdown_modal(
    f: &mut Frame,
    area: Rect,
    categories: &[CategoryStat],
    total_scanned_bytes: u64,
    selected_index: usize,
    theme: &Theme,
) {
    let popup_area = centered_rect(72, 65, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(theme.border_active_style())
        .title(" 󰄫 Storage Breakdown by File Category ")
        .title_style(theme.title_style())
        .style(Style::default().bg(theme.bg_base));

    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(2), Constraint::Min(8), Constraint::Length(1)])
        .split(popup_area);

    let header_text = format!(
        " Overall Project Footprint: {} across {} categories ",
        format_bytes(total_scanned_bytes),
        categories.len()
    );
    let header_p = Paragraph::new(header_text).style(Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD));
    f.render_widget(header_p, inner[0]);

    let max_cat_size = categories.first().map(|c| c.total_size).unwrap_or(1).max(1);

    let list_items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(idx, cat)| {
            let is_sel = idx == selected_index;
            let ratio_overall = if total_scanned_bytes > 0 {
                (cat.total_size as f64) / (total_scanned_bytes as f64)
            } else {
                0.0
            };

            let ratio_max = (cat.total_size as f64) / (max_cat_size as f64);
            let bar_width = 16;
            let (bar_str, _) = theme.format_usage_bar(ratio_max, bar_width);

            let prefix = if is_sel { " ❯ " } else { "   " };
            let pct_str = format!("{:>5.1}%", ratio_overall * 100.0);

            let spans = vec![
                Span::styled(prefix, if is_sel { Style::default().fg(theme.mauve) } else { theme.dim_style() }),
                Span::styled(format!("{} ", cat.icon), Style::default().fg(cat.color)),
                Span::styled(
                    format!("{:<22} ", cat.name),
                    if is_sel { Style::default().fg(theme.text).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.subtext) },
                ),
                Span::styled(bar_str, Style::default().fg(cat.color)),
                Span::styled(format!(" {} ", pct_str), Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(" {:>10}", format_bytes(cat.total_size)),
                    Style::default().fg(theme.size_color(cat.total_size)).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  ({} files)", cat.file_count),
                    Style::default().fg(theme.overlay),
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
        Span::styled(" [j/k] ", Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD)),
        Span::styled("Select Category   ", Style::default().fg(theme.subtext)),
        Span::styled(" [Esc/b/q] ", Style::default().fg(theme.red).add_modifier(Modifier::BOLD)),
        Span::styled("Close Breakdown", Style::default().fg(theme.subtext)),
    ]);
    f.render_widget(Paragraph::new(footer), inner[2]);
}

/// Compute category aggregates across all files in the tree
pub fn calculate_category_stats(tree: &DirectoryTree) -> (Vec<CategoryStat>, u64) {
    let mut code_size = 0u64;
    let mut code_count = 0usize;

    let mut build_size = 0u64;
    let mut build_count = 0usize;

    let mut media_size = 0u64;
    let mut media_count = 0usize;

    let mut archive_size = 0u64;
    let mut archive_count = 0usize;

    let mut data_size = 0u64;
    let mut data_count = 0usize;

    let mut doc_size = 0u64;
    let mut doc_count = 0usize;

    let mut other_size = 0u64;
    let mut other_count = 0usize;

    let mut total_bytes = 0u64;

    for node in &tree.nodes {
        if node.is_dir() {
            continue;
        }

        total_bytes += node.disk_size;
        let ext = node
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_build_path = node
            .path
            .components()
            .any(|c| c.as_os_str() == "target" || c.as_os_str() == "build" || c.as_os_str() == "dist" || c.as_os_str() == "node_modules");

        if is_build_path || matches!(ext.as_str(), "o" | "so" | "a" | "dll" | "exe" | "bin" | "dylib" | "class") {
            build_size += node.disk_size;
            build_count += 1;
        } else if node.is_encrypted || matches!(ext.as_str(), "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "zst") {
            archive_size += node.disk_size;
            archive_count += 1;
        } else if matches!(ext.as_str(), "rs" | "py" | "c" | "cpp" | "h" | "hpp" | "go" | "java" | "js" | "ts" | "jsx" | "tsx" | "sh" | "html" | "css" | "lua" | "sql") {
            code_size += node.disk_size;
            code_count += 1;
        } else if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "mp4" | "mkv" | "avi" | "mp3" | "wav" | "flac") {
            media_size += node.disk_size;
            media_count += 1;
        } else if matches!(ext.as_str(), "json" | "yaml" | "yml" | "toml" | "xml" | "csv" | "ini" | "conf" | "db" | "sqlite") {
            data_size += node.disk_size;
            data_count += 1;
        } else if matches!(ext.as_str(), "md" | "txt" | "pdf" | "docx" | "odt") {
            doc_size += node.disk_size;
            doc_count += 1;
        } else {
            other_size += node.disk_size;
            other_count += 1;
        }
    }

    let mut list = vec![
        CategoryStat {
            name: "Build Artifacts & Cache",
            icon: "󰏗",
            color: Color::Rgb(249, 226, 175), // Yellow
            total_size: build_size,
            file_count: build_count,
        },
        CategoryStat {
            name: "Source Code & Scripts",
            icon: "",
            color: Color::Rgb(250, 179, 135), // Peach
            total_size: code_size,
            file_count: code_count,
        },
        CategoryStat {
            name: "Archives & Vaults",
            icon: "",
            color: Color::Rgb(203, 166, 247), // Mauve
            total_size: archive_size,
            file_count: archive_count,
        },
        CategoryStat {
            name: "Data & Configurations",
            icon: "",
            color: Color::Rgb(148, 226, 213), // Teal
            total_size: data_size,
            file_count: data_count,
        },
        CategoryStat {
            name: "Media, Audio & Video",
            icon: "󰋩",
            color: Color::Rgb(137, 180, 250), // Blue
            total_size: media_size,
            file_count: media_count,
        },
        CategoryStat {
            name: "Documents & Notes",
            icon: "",
            color: Color::Rgb(166, 227, 161), // Green
            total_size: doc_size,
            file_count: doc_count,
        },
        CategoryStat {
            name: "Other / Unclassified",
            icon: "󰈔",
            color: Color::Rgb(166, 173, 200), // Subtext
            total_size: other_size,
            file_count: other_count,
        },
    ];

    list.retain(|c| c.file_count > 0 || c.total_size > 0);
    list.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    (list, total_bytes)
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
