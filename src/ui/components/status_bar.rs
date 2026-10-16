use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Scanning,
    Jump,
    Diff,
    Crypto,
    Inode,
    TopHogs,
    Delete,
    Filter,
    Breakdown,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[allow(clippy::too_many_arguments)]
pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    mode: AppMode,
    current_path: &str,
    total_size: u64,
    item_count: usize,
    status_message: &str,
    frame_tick: usize,
    preview_active: bool,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let spinner = SPINNER_FRAMES[frame_tick % SPINNER_FRAMES.len()];

    let scanning_text;
    let (mode_text, mode_bg, mode_fg): (&str, _, _) = match mode {
        AppMode::Normal => (" NORMAL ", theme.mauve, theme.bg_crust),
        AppMode::Scanning => {
            scanning_text = format!(" {} SCANNING ", spinner);
            (scanning_text.as_str(), theme.yellow, theme.bg_crust)
        }
        AppMode::Jump => (" QUICK-JUMP ", theme.teal, theme.bg_crust),
        AppMode::Diff => (" SNAPSHOT-DIFF ", theme.sapphire, theme.bg_crust),
        AppMode::Crypto => (" VAULT-CRYPT ", theme.peach, theme.bg_crust),
        AppMode::Inode => (" INODE-STAT ", theme.green, theme.bg_crust),
        AppMode::TopHogs => (" TOP-HOGS ", theme.red, theme.bg_crust),
        AppMode::Delete => (" RECLAIM ", theme.red, theme.bg_crust),
        AppMode::Filter => (" LIVE-FILTER ", theme.yellow, theme.bg_crust),
        AppMode::Breakdown => (" CATEGORY-STAT ", theme.peach, theme.bg_crust),
    };

    let mode_pill = Span::styled(
        mode_text,
        Style::default()
            .bg(mode_bg)
            .fg(mode_fg)
            .add_modifier(Modifier::BOLD),
    );

    let path_span = Span::styled(
        format!("  {} ", truncate_path(current_path, 34)),
        Style::default().bg(theme.surface0).fg(theme.text),
    );

    let gauge_span = Span::styled(
        format!(
            " 󰋊 {} ({} items) ",
            format_bytes_short(total_size),
            item_count
        ),
        Style::default()
            .bg(theme.surface1)
            .fg(theme.mauve)
            .add_modifier(Modifier::BOLD),
    );

    let preview_tag = if preview_active {
        Span::styled(
            " [PREVIEW] ",
            Style::default()
                .bg(theme.teal)
                .fg(theme.bg_crust)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let msg_span = if !status_message.is_empty() {
        Span::styled(
            format!(" │ {} ", status_message),
            Style::default().fg(theme.yellow),
        )
    } else {
        Span::raw("")
    };

    let line1 = Line::from(vec![
        mode_pill,
        path_span,
        gauge_span,
        preview_tag,
        msg_span,
    ]);
    f.render_widget(
        Paragraph::new(line1).block(Block::default().style(Style::default().bg(theme.bg_mantle))),
        chunks[0],
    );

    // Line 2: Linux-rice clean keybindings bar
    let keys = vec![
        Span::styled(
            " [j/k] ",
            Style::default()
                .fg(theme.mauve)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Nav ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [Enter/h] ",
            Style::default()
                .fg(theme.mauve)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("In/Out ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [Tab] ",
            Style::default().fg(theme.teal).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Preview ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [b] ",
            Style::default()
                .fg(theme.peach)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Breakdown ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [i] ",
            Style::default()
                .fg(theme.green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Inode ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [t] ",
            Style::default()
                .fg(theme.peach)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Hogs ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [/] ",
            Style::default()
                .fg(theme.yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Filter ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [s] ",
            Style::default()
                .fg(theme.mauve)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Sort ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [f] ",
            Style::default().fg(theme.teal).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Jump ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [p/d] ",
            Style::default()
                .fg(theme.sapphire)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Snap/Diff ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [e/u] ",
            Style::default()
                .fg(theme.peach)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Crypt ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [T] ",
            Style::default()
                .fg(theme.sapphire)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Theme ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [x] ",
            Style::default().fg(theme.red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Del ", Style::default().fg(theme.subtext)),
        Span::styled(
            " [q] ",
            Style::default().fg(theme.red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Quit", Style::default().fg(theme.subtext)),
    ];

    let line2 = Line::from(keys);
    f.render_widget(
        Paragraph::new(line2)
            .alignment(Alignment::Left)
            .style(Style::default().bg(theme.bg_crust)),
        chunks[1],
    );
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("…{}", &path[path.len().saturating_sub(max_len)..])
    }
}

fn format_bytes_short(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
