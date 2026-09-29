use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::preset::Preset;

/// Custom Dark Palette (Matched to terminal prompt)
struct Palette;
impl Palette {
    // Purple/Lavender from the time section
    const BLUE: Color = Color::Rgb(186, 180, 250); 
    const TEXT: Color = Color::Rgb(240, 240, 240);
    const SUBTEXT: Color = Color::Rgb(120, 125, 140);
    // Green from the version section
    const GREEN: Color = Color::Rgb(152, 222, 142);
    // Yellow/Peach from the git branch section
    const YELLOW: Color = Color::Rgb(246, 211, 143);
    // Pink/Red from the user/host section
    const RED: Color = Color::Rgb(238, 121, 149);
}

/// All the state the UI needs to render a frame.
pub struct UiState<'a> {
    pub monitor_name: &'a str,
    pub connection: &'a str,
    pub brightness: u8,
    pub pending_brightness: u8,
    pub brightness_locked: bool,
    pub presets: &'a [Preset],
    pub selected_preset: usize,
    pub active_preset: usize,
    pub error_msg: Option<&'a str>,
}

pub fn draw(frame: &mut Frame, state: &UiState) {
    let area = frame.area();

    // Outer block
    let outer = Block::default()
        .title("  MadCtl  ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Palette::BLUE));

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
    let label = Style::default().fg(Palette::SUBTEXT);
    let value = Style::default().fg(Palette::TEXT).add_modifier(Modifier::BOLD);

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

    let (bar_color, pct_label) = if state.brightness_locked {
        (Palette::SUBTEXT, format!("  {}% · locked", display_val))
    } else if is_pending {
        (Palette::YELLOW, format!("  {}% (pending)", display_val))
    } else {
        (Palette::BLUE, format!("  {}%", display_val))
    };

    let label_style = if state.brightness_locked {
        Style::default().fg(Palette::SUBTEXT)
    } else {
        Style::default().fg(Palette::TEXT).add_modifier(Modifier::BOLD)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Brightness  ", Style::default().fg(Palette::SUBTEXT)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&bar[..filled * 3], Style::default().fg(bar_color)), // ━ is 3 bytes
            Span::styled(&bar[filled * 3..], Style::default().fg(Palette::SUBTEXT)),
            Span::styled(pct_label, label_style),
        ]),
    ];

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn draw_presets(frame: &mut Frame, area: Rect, state: &UiState) {
    let mut lines = vec![
        Line::from(Span::styled(
            "  Preset",
            Style::default().fg(Palette::SUBTEXT),
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
                .fg(Palette::GREEN)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(Palette::BLUE)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Palette::GREEN)
        } else {
            Style::default().fg(Palette::TEXT)
        };

        lines.push(Line::from(Span::styled(format!("  {}", label), style)));
    }

    // Show error if any
    if let Some(err) = state.error_msg {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  ✗ {}", err),
            Style::default().fg(Palette::RED),
        )));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, _state: &UiState) {
    let style_key = Style::default()
        .fg(Palette::BLUE)
        .add_modifier(Modifier::BOLD);
    let style_desc = Style::default().fg(Palette::SUBTEXT);

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
            .border_style(Style::default().fg(Palette::SUBTEXT)),
    );
    frame.render_widget(para, area);
}

/// Draw the "no monitor" error screen.
pub fn draw_error(frame: &mut Frame, message: &str) {
    let area = frame.area();

    let outer = Block::default()
        .title("  MadCtl  ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Palette::RED));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  ✗ {}", message),
            Style::default().fg(Palette::RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Press q to quit",
            Style::default().fg(Palette::SUBTEXT),
        )),
    ];

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}
