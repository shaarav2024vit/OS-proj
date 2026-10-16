use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::model::{DirectoryTree, NodeId};
use crate::ui::icons::get_file_icon;
use crate::ui::theme::Theme;

pub fn render_preview_pane(
    f: &mut Frame,
    area: Rect,
    tree: &DirectoryTree,
    selected_node_id: Option<NodeId>,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(theme.border_style())
        .title(" 󰈈 Preview ")
        .title_style(theme.title_style())
        .style(Style::default().bg(theme.bg_base));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let node = match selected_node_id.and_then(|id| tree.get_node(id)) {
        Some(n) => n,
        None => {
            let empty_p = Paragraph::new(" No item selected ").style(theme.dim_style());
            f.render_widget(empty_p, inner);
            return;
        }
    };

    let icon = get_file_icon(&node.path, node.is_dir(), node.is_encrypted);

    if node.is_encrypted {
        render_encrypted_preview(f, inner, node.name.as_str(), &node.path, theme);
    } else if node.is_dir() {
        render_directory_preview(f, inner, tree, node.id, theme);
    } else {
        render_file_preview(f, inner, &node.path, icon.glyph, icon.color, theme);
    }
}

fn render_file_preview(
    f: &mut Frame,
    area: Rect,
    path: &Path,
    icon: &str,
    icon_color: Color,
    theme: &Theme,
) {
    let mut lines = Vec::new();

    // Header line
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
        Span::styled(
            path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Attempt to read text content
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let max_lines = area.height.saturating_sub(3) as usize;
        let mut is_binary = false;

        for (idx, line_res) in reader.lines().take(max_lines).enumerate() {
            match line_res {
                Ok(content) => {
                    // Check if line contains null bytes (binary file)
                    if content.contains('\0') {
                        is_binary = true;
                        break;
                    }
                    let line_num = format!("{:>3} │ ", idx + 1);
                    lines.push(Line::from(vec![
                        Span::styled(line_num, Style::default().fg(theme.overlay)),
                        Span::styled(content, Style::default().fg(theme.text)),
                    ]));
                }
                Err(_) => {
                    is_binary = true;
                    break;
                }
            }
        }

        if is_binary {
            lines.clear();
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                Span::styled(
                    path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(" [Binary or Non-UTF8 Content] ", Style::default().fg(theme.yellow))));
            if let Ok(meta) = path.metadata() {
                lines.push(Line::from(vec![
                    Span::styled(" Size: ", Style::default().fg(theme.subtext)),
                    Span::styled(format_bytes(meta.len()), Style::default().fg(theme.sapphire)),
                ]));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(" Unable to read file ", Style::default().fg(theme.red))));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn render_directory_preview(
    f: &mut Frame,
    area: Rect,
    tree: &DirectoryTree,
    dir_id: NodeId,
    theme: &Theme,
) {
    let mut lines = Vec::new();
    let dir_node = match tree.get_node(dir_id) {
        Some(n) => n,
        None => return,
    };

    lines.push(Line::from(vec![
        Span::styled("󰉋 ", Style::default().fg(theme.blue)),
        Span::styled(&dir_node.name, Style::default().fg(theme.blue).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" Total Size:   ", Style::default().fg(theme.subtext)),
        Span::styled(format_bytes(dir_node.disk_size), Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" Total Items:  ", Style::default().fg(theme.subtext)),
        Span::styled(format!("{}", dir_node.item_count), Style::default().fg(theme.teal)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(" Largest Contents: ", Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD))));

    let sorted = tree.sorted_children(dir_id, crate::model::SortOrder::SizeDesc);
    let max_display = area.height.saturating_sub(6) as usize;

    for &child_id in sorted.iter().take(max_display) {
        if let Some(child) = tree.get_node(child_id) {
            let icon = get_file_icon(&child.path, child.is_dir(), child.is_encrypted);
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon.glyph), Style::default().fg(icon.color)),
                Span::styled(format!("{:<20} ", truncate(&child.name, 18)), Style::default().fg(theme.text)),
                Span::styled(format_bytes(child.disk_size), Style::default().fg(theme.size_color(child.disk_size))),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn render_encrypted_preview(
    f: &mut Frame,
    area: Rect,
    name: &str,
    _path: &Path,
    theme: &Theme,
) {
    let lines = vec![
        Line::from(vec![
            Span::styled("󰌆 ", Style::default().fg(theme.peach)),
            Span::styled(name, Style::default().fg(theme.peach).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(" 󰒃 Authenticated Strata Vault ", Style::default().bg(theme.peach).fg(theme.bg_crust).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Algorithm:    ", Style::default().fg(theme.subtext)),
            Span::styled("AES-256-GCM (NIST SP 800-38D)", Style::default().fg(theme.teal)),
        ]),
        Line::from(vec![
            Span::styled(" Key Deriv:    ", Style::default().fg(theme.subtext)),
            Span::styled("Argon2id (RFC 9106)", Style::default().fg(theme.teal)),
        ]),
        Line::from(vec![
            Span::styled(" Nonce & Salt: ", Style::default().fg(theme.subtext)),
            Span::styled("Cryptographically Random (28B)", Style::default().fg(theme.teal)),
        ]),
        Line::from(vec![
            Span::styled(" Integrity:    ", Style::default().fg(theme.subtext)),
            Span::styled("Poly1305 AEAD Tag (Tamper-Proof)", Style::default().fg(theme.green)),
        ]),
        Line::from(""),
        Line::from(Span::styled(" Press [u] to unlock & decrypt this item ", Style::default().fg(theme.yellow))),
    ];

    f.render_widget(Paragraph::new(lines), area);
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
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}
