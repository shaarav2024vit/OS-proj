use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::model::VolumeInfo;
use crate::ui::theme::Theme;

pub fn render_volume_bar(
    f: &mut Frame,
    area: Rect,
    vol: Option<&VolumeInfo>,
    theme: &Theme,
) {
    let vol = match vol {
        Some(v) => v,
        None => {
            let p = Paragraph::new(" 󰋊 Storage Volume: Initializing mount metrics... ")
                .style(theme.dim_style())
                .block(Block::default().style(Style::default().bg(theme.bg_mantle)));
            f.render_widget(p, area);
            return;
        }
    };

    let ratio = vol.used_ratio();
    let bar_width = 14;
    let (bar_str, bar_style) = theme.format_usage_bar(ratio, bar_width);

    let dev_tag = format!(" 󰋊 {} [{} on {}] ", vol.device, vol.fs_type, vol.mount_point);
    let dev_span = Span::styled(
        dev_tag,
        Style::default()
            .bg(theme.surface0)
            .fg(theme.mauve)
            .add_modifier(Modifier::BOLD),
    );

    let used_str = format_bytes(vol.used_bytes);
    let total_str = format_bytes(vol.total_bytes);
    let free_str = format_bytes(vol.available_bytes);
    let pct_str = format!("{:.1}%", ratio * 100.0);

    let stats_span = Span::styled(
        format!(" {} / {} ({} used, {} free) ", used_str, total_str, pct_str, free_str),
        Style::default().fg(theme.text),
    );

    let inode_str = if vol.total_inodes > 0 {
        let inode_ratio = vol.inode_used_ratio();
        format!(" │ Inodes: {:.1}% ", inode_ratio * 100.0)
    } else {
        "".to_string()
    };
    let inode_span = Span::styled(inode_str, Style::default().fg(theme.overlay));

    let line = Line::from(vec![
        dev_span,
        Span::raw(" "),
        Span::styled(bar_str, bar_style),
        stats_span,
        inode_span,
    ]);

    let p = Paragraph::new(line).block(Block::default().style(Style::default().bg(theme.bg_mantle)));
    f.render_widget(p, area);
}

fn format_bytes(bytes: u64) -> String {
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
