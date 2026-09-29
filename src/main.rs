mod ddc;
mod monitor;
mod preset;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

use monitor::Monitor;
use preset::Preset;

/// Application state.
struct App {
    monitor: Monitor,
    brightness: u8,
    pending_brightness: u8,
    selected_preset: usize,
    active_preset: usize,
    error_msg: Option<String>,
}

impl App {
    fn new(mon: Monitor, brightness: u8, active_preset: usize) -> Self {
        Self {
            monitor: mon,
            brightness,
            pending_brightness: brightness,
            selected_preset: active_preset,
            active_preset,
            error_msg: None,
        }
    }
}

fn main() -> Result<()> {
    // Detect monitor
    let mon = match Monitor::detect() {
        Ok(m) => m,
        Err(e) => {
            // Show error TUI
            run_error_ui(&format!("{}", e))?;
            return Ok(());
        }
    };

    // Read initial preset
    let active_preset = match monitor::get_preset(mon.display) {
        Ok(p) => Preset::ALL
            .iter()
            .position(|x| *x == p)
            .unwrap_or(0),
        Err(_) => 0,
    };

    // Read brightness — if in a non-Standard mode, the monitor may report
    // a locked value (e.g. 100%). Temporarily switch to Standard to read
    // the real brightness, then switch back.
    let brightness = if active_preset != 0 {
        let current_preset = Preset::ALL[active_preset];
        // Switch to Standard to read real brightness
        let _ = monitor::set_preset(mon.display, Preset::Standard);
        std::thread::sleep(Duration::from_millis(300));
        let b = monitor::get_brightness(mon.display).unwrap_or(50);
        // Switch back to the original preset
        let _ = monitor::set_preset(mon.display, current_preset);
        b
    } else {
        monitor::get_brightness(mon.display).unwrap_or(50)
    };

    let mut app = App::new(mon, brightness, active_preset);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Event loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Brightness editing only allowed in Standard mode (index 0)
        let brightness_locked = app.active_preset != 0;

        terminal.draw(|frame| {
            let state = ui::UiState {
                monitor_name: &app.monitor.name,
                connection: &app.monitor.connection,
                brightness: app.brightness,
                pending_brightness: app.pending_brightness,
                brightness_locked,
                presets: &Preset::ALL,
                selected_preset: app.selected_preset,
                active_preset: app.active_preset,
                error_msg: app.error_msg.as_deref(),
            };
            ui::draw(frame, &state);
        })?;

        // Poll for events (250ms timeout to keep responsive)
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release/repeat)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Clear previous error on any keypress
                app.error_msg = None;

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),

                    KeyCode::Up => {
                        if app.selected_preset > 0 {
                            app.selected_preset -= 1;
                        }
                    }

                    KeyCode::Down => {
                        if app.selected_preset < Preset::ALL.len() - 1 {
                            app.selected_preset += 1;
                        }
                    }

                    KeyCode::Left => {
                        if brightness_locked {
                            app.error_msg = Some("Brightness can only be changed in Standard mode".to_string());
                        } else {
                            app.pending_brightness = app.pending_brightness.saturating_sub(5);
                        }
                    }

                    KeyCode::Right => {
                        if brightness_locked {
                            app.error_msg = Some("Brightness can only be changed in Standard mode".to_string());
                        } else {
                            app.pending_brightness = (app.pending_brightness + 5).min(100);
                        }
                    }

                    KeyCode::Enter => {
                        // Apply pending brightness if changed (only in Standard mode)
                        if !brightness_locked && app.pending_brightness != app.brightness {
                            match monitor::set_brightness(app.monitor.display, app.pending_brightness) {
                                Ok(()) => app.brightness = app.pending_brightness,
                                Err(e) => app.error_msg = Some(format!("Failed to change brightness: {}", e)),
                            }
                        }
                        // Apply preset if changed
                        if app.selected_preset != app.active_preset {
                            let preset = Preset::ALL[app.selected_preset];
                            match monitor::set_preset(app.monitor.display, preset) {
                                Ok(()) => {
                                    app.active_preset = app.selected_preset;
                                    // Re-read brightness after preset change since it may differ
                                    if let Ok(b) = monitor::get_brightness(app.monitor.display) {
                                        app.brightness = b;
                                        app.pending_brightness = b;
                                    }
                                }
                                Err(e) => app.error_msg = Some(format!("Failed to set preset: {}", e)),
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}

/// Minimal error-only UI when no monitor is detected.
fn run_error_ui(message: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| {
            ui::draw_error(frame, message);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
