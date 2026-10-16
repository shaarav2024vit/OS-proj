use std::path::Path;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use super::jump_modal::centered_rect;
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoAction {
    Encrypt,
    Decrypt,
}

#[allow(clippy::too_many_arguments)]
pub fn render_crypto_modal(
    f: &mut Frame,
    area: Rect,
    action: CryptoAction,
    target_path: &Path,
    password_len: usize,
    confirm_len: Option<usize>,
    is_confirm_field: bool,
    status_msg: Option<&str>,
    theme: &Theme,
) {
    let popup_area = centered_rect(58, 42, area);
    f.render_widget(Clear, popup_area);

    let (title, border_color) = match action {
        CryptoAction::Encrypt => (
            " 󰌆 Secure Vault: Encrypt (AES-256-GCM + Argon2id) ",
            theme.peach,
        ),
        CryptoAction::Decrypt => (" 󰌆 Secure Vault: Decrypt (AES-256-GCM) ", theme.teal),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg_base));

    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(popup_area);

    let target_name = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("selected item");

    let action_desc = match action {
        CryptoAction::Encrypt => {
            format!("Encrypting '{}' with Argon2id key derivation", target_name)
        }
        CryptoAction::Decrypt => format!("Decrypting '{}' using master passphrase", target_name),
    };

    let info_p = Paragraph::new(action_desc).style(Style::default().fg(theme.text));
    f.render_widget(info_p, inner[0]);

    // Password input box
    let masked_pw = "•".repeat(password_len);
    let pw_border_color = if !is_confirm_field {
        theme.mauve
    } else {
        theme.surface1
    };
    let pw_block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::BORDER_TYPE)
        .border_style(Style::default().fg(pw_border_color))
        .title(" Master Passphrase ");

    let cursor1 = if !is_confirm_field { "█" } else { "" };
    let pw_text = format!(" ❯ {}{}", masked_pw, cursor1);
    let pw_p = Paragraph::new(pw_text)
        .style(
            Style::default()
                .fg(theme.mauve)
                .add_modifier(Modifier::BOLD),
        )
        .block(pw_block);
    f.render_widget(pw_p, inner[1]);

    // Confirmation input box or Strength indicator
    if let Some(c_len) = confirm_len {
        let masked_confirm = "•".repeat(c_len);
        let confirm_border_color = if is_confirm_field {
            theme.mauve
        } else {
            theme.surface1
        };
        let confirm_block = Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::BORDER_TYPE)
            .border_style(Style::default().fg(confirm_border_color))
            .title(" Confirm Passphrase [Tab to switch] ");

        let cursor2 = if is_confirm_field { "█" } else { "" };
        let confirm_text = format!(" ❯ {}{}", masked_confirm, cursor2);
        let confirm_p = Paragraph::new(confirm_text)
            .style(
                Style::default()
                    .fg(theme.mauve)
                    .add_modifier(Modifier::BOLD),
            )
            .block(confirm_block);
        f.render_widget(confirm_p, inner[2]);
    } else {
        // Render simple strength indication
        let strength_text = if password_len == 0 {
            Span::styled(" Entropy: None ", theme.dim_style())
        } else if password_len < 6 {
            Span::styled(" Entropy: Weak ▰▱▱▱ ", Style::default().fg(theme.red))
        } else if password_len < 10 {
            Span::styled(" Entropy: Fair ▰▰▱▱ ", Style::default().fg(theme.yellow))
        } else {
            Span::styled(" Entropy: Strong ▰▰▰▰ ", Style::default().fg(theme.green))
        };
        let strength_p = Paragraph::new(Line::from(vec![strength_text]));
        f.render_widget(strength_p, inner[2]);
    }

    // Footer / status
    let footer_text = if let Some(msg) = status_msg {
        Line::from(vec![Span::styled(
            msg,
            Style::default().fg(theme.red).add_modifier(Modifier::BOLD),
        )])
    } else {
        Line::from(vec![
            Span::styled(
                " [Enter] ",
                Style::default()
                    .fg(theme.green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Execute  ", Style::default().fg(theme.subtext)),
            Span::styled(
                " [Tab] ",
                Style::default().fg(theme.teal).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Next field  ", Style::default().fg(theme.subtext)),
            Span::styled(
                " [Esc] ",
                Style::default().fg(theme.red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Cancel", Style::default().fg(theme.subtext)),
        ])
    };

    let footer_p = Paragraph::new(footer_text);
    f.render_widget(footer_p, inner[3]);
}
