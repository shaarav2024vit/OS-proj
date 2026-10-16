use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use super::jump_modal::centered_rect;
use crate::ui::theme::Theme;

pub fn render_inode_modal(
    f: &mut Frame,
    area: Rect,
    target_path: &Path,
    theme: &Theme,
) {
    let popup_area = centered_rect(65, 60, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(theme.border_active_style())
        .title(" 󰋊 POSIX Inode & Filesystem Metadata Inspector ")
        .title_style(theme.title_style())
        .style(Style::default().bg(theme.bg_base));

    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(popup_area);

    let metadata = match fs::symlink_metadata(target_path) {
        Ok(m) => m,
        Err(err) => {
            let err_p = Paragraph::new(format!("Failed to read metadata: {}", err))
                .style(Style::default().fg(theme.red));
            f.render_widget(err_p, inner[0]);
            return;
        }
    };

    let mode = metadata.mode();
    let permissions_sym = format_permissions(mode);
    let permissions_oct = format!("{:04o}", mode & 0o7777);

    let ino = metadata.ino();
    let dev = metadata.dev();
    let nlink = metadata.nlink();
    let uid = metadata.uid();
    let gid = metadata.gid();
    let blksize = metadata.blksize();
    let blocks = metadata.blocks();
    let physical_bytes = blocks * 512;
    let logical_bytes = metadata.len();

    let atime = format_time(metadata.atime() as u64);
    let mtime = format_time(metadata.mtime() as u64);
    let ctime = format_time(metadata.ctime() as u64);

    let is_dir = metadata.is_dir();
    let is_symlink = metadata.file_type().is_symlink();
    let kind_str = if is_dir { "Directory" } else if is_symlink { "Symbolic Link" } else { "Regular File" };

    let mut lines = Vec::new();

    // Section 1: File Identity
    lines.push(Line::from(vec![
        Span::styled("Target Path:     ", Style::default().fg(theme.subtext)),
        Span::styled(target_path.display().to_string(), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Node Type:       ", Style::default().fg(theme.subtext)),
        Span::styled(kind_str, Style::default().fg(theme.blue).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Section 2: Inode & POSIX Properties
    lines.push(Line::from(vec![
        Span::styled("Inode Number:    ", Style::default().fg(theme.mauve)),
        Span::styled(format!("{:<15}", ino), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        Span::styled("Device ID:      ", Style::default().fg(theme.mauve)),
        Span::styled(format!("{}", dev), Style::default().fg(theme.text)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("POSIX Perms:     ", Style::default().fg(theme.teal)),
        Span::styled(format!("{} (octal: {})  ", permissions_sym, permissions_oct), Style::default().fg(theme.green).add_modifier(Modifier::BOLD)),
        Span::styled("Hard Links:     ", Style::default().fg(theme.teal)),
        Span::styled(format!("{}", nlink), Style::default().fg(theme.text)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Owner / Group:   ", Style::default().fg(theme.subtext)),
        Span::styled(format!("UID {} : GID {}", uid, gid), Style::default().fg(theme.text)),
    ]));
    lines.push(Line::from(""));

    // Section 3: Disk Space & Block Allocation
    lines.push(Line::from(vec![
        Span::styled("Logical Size:    ", Style::default().fg(theme.sapphire)),
        Span::styled(format!("{} ({} bytes)", format_bytes(logical_bytes), logical_bytes), Style::default().fg(theme.text)),
    ]));

    let eff_str = if physical_bytes > 0 {
        let ratio = (logical_bytes as f64) / (physical_bytes as f64) * 100.0;
        format!(" (Allocated: {:.1}%)", ratio)
    } else {
        "".to_string()
    };

    lines.push(Line::from(vec![
        Span::styled("Disk Allocated:  ", Style::default().fg(theme.sapphire)),
        Span::styled(format!("{} ({} 512-byte blocks){}", format_bytes(physical_bytes), blocks, eff_str), Style::default().fg(theme.peach).add_modifier(Modifier::BOLD)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("I/O Block Size:  ", Style::default().fg(theme.subtext)),
        Span::styled(format!("{} bytes (optimum filesystem cluster)", blksize), Style::default().fg(theme.text)),
    ]));
    lines.push(Line::from(""));

    // Section 4: Kernel Timestamps
    lines.push(Line::from(vec![
        Span::styled("Access (atime):  ", Style::default().fg(theme.overlay)),
        Span::styled(atime, Style::default().fg(theme.subtext)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Modify (mtime):  ", Style::default().fg(theme.overlay)),
        Span::styled(mtime, Style::default().fg(theme.subtext)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Change (ctime):  ", Style::default().fg(theme.overlay)),
        Span::styled(ctime, Style::default().fg(theme.subtext)),
    ]));

    f.render_widget(Paragraph::new(lines), inner[0]);

    // Footer
    let footer = Line::from(vec![
        Span::styled(" [Esc/i/q] ", Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD)),
        Span::styled("Close Inode Inspector", Style::default().fg(theme.subtext)),
    ]);
    f.render_widget(Paragraph::new(footer), inner[1]);
}

fn format_permissions(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    s.push(if (mode & 0o170000) == 0o040000 { 'd' } else if (mode & 0o170000) == 0o120000 { 'l' } else { '-' });
    s.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    s.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    s.push(if (mode & 0o100) != 0 { 'x' } else { '-' });
    s.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    s.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    s.push(if (mode & 0o010) != 0 { 'x' } else { '-' });
    s.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    s.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    s.push(if (mode & 0o001) != 0 { 'x' } else { '-' });
    s
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
