use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::preset::Preset;

/// All the state the UI needs to render a frame.
pub struct UiState<'a> {
    pub monitor_name: &'a str,
    pub connection: &'a str,
    pub brightness: u8,
    pub pending_brightness: u8,
    pub presets: &'a [Preset],
    pub selected_preset: usize,
    pub active_preset: usize,
    pub error_msg: Option<&'a str>,
}

pub fn draw(frame: &mut Frame, state: &UiState) {
    let area = frame.area();

    // Outer block
    let outer = Block::default()
        .title("  MonitorCtl  ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Layout: info | brightness | presets | status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // monitor info
            Constraint::Length(3), // brightness
            Constraint::Min(3),   // presets
            Constraint::Length(3), // status bar
        ])
        .split(inner);

    draw_monitor_info(frame, chunks[0], state);
    draw_brightness(frame, chunks[1], state);
    draw_presets(frame, chunks[2], state);
    draw_status_bar(frame, chunks[3], state);
}

fn draw_monitor_info(frame: &mut Frame, area: Rect, state: &UiState) {
    let label = Style::default().fg(Color::DarkGray);
    let value = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);

    let lines = vec![
        Line::from(vec![
            Span::styled("  Monitor     ", label),
            Span::styled(state.monitor_name, value),
        ]),
        Line::from(vec![
            Span::styled("  Connection  ", label),
            Span::styled(state.connection, value),
        ]),
    ];

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn draw_brightness(frame: &mut Frame, area: Rect, state: &UiState) {
    let is_pending = state.pending_brightness != state.brightness;
    let display_val = state.pending_brightness as usize;
    let bar_width = (area.width as usize).saturating_sub(18); // room for label + percentage
    let filled = (display_val * bar_width) / 100;
    let empty = bar_width.saturating_sub(filled);

    let bar = format!(
        "{}{}",
        "━".repeat(filled),
        "─".repeat(empty),
    );

    let bar_color = if is_pending { Color::Yellow } else { Color::Cyan };
    let pct_label = if is_pending {
        format!("  {}% (pending)", display_val)
    } else {
        format!("  {}%", display_val)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Brightness  ", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&bar[..filled * 3], Style::default().fg(bar_color)), // ━ is 3 bytes
            Span::styled(&bar[filled * 3..], Style::default().fg(Color::DarkGray)),
            Span::styled(pct_label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn draw_presets(frame: &mut Frame, area: Rect, state: &UiState) {
    let mut lines = vec![
        Line::from(Span::styled(
            "  Preset",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    for (i, preset) in state.presets.iter().enumerate() {
        let is_selected = i == state.selected_preset;
        let is_active = i == state.active_preset;

        let marker = if is_selected { "▸ " } else { "  " };
        let label = format!("{}{}", marker, preset);

        let style = if is_active && is_selected {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(Span::styled(format!("  {}", label), style)));
    }

    // Show error if any
    if let Some(err) = state.error_msg {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  ✗ {}", err),
            Style::default().fg(Color::Red),
        )));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, _state: &UiState) {
    let style_key = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let style_desc = Style::default().fg(Color::DarkGray);

    let lines = vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled("↑↓", style_key),
            Span::styled(" Select   ", style_desc),
            Span::styled("←→", style_key),
            Span::styled(" Brightness   ", style_desc),
            Span::styled("Enter", style_key),
            Span::styled(" Apply", style_desc),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("q", style_key),
            Span::styled(" Quit", style_desc),
        ]),
    ];

    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(para, area);
}

/// Draw the "no monitor" error screen.
pub fn draw_error(frame: &mut Frame, message: &str) {
    let area = frame.area();

    let outer = Block::default()
        .title("  MonitorCtl  ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  ✗ {}", message),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Press q to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}
