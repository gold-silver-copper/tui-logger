//! Line-by-line scrolling of the log view.
//!
//! Run with: `cargo run --example scroll_line --features crossterm`
//!
//! | KEY          | ACTION
//! |--------------|-------------------------------------------------------
//! | UP / k       | Scroll one line up
//! | DOWN / j     | Scroll one line down, follow the log again at the bottom
//! | PAGEUP       | Scroll approx. half a page up
//! | PAGEDOWN     | Scroll approx. half a page down
//! | ESCAPE       | Stop scrolling and follow the log again
//! | q            | Quit
use std::{thread, time::Duration};

use log::*;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Line,
    widgets::Block,
    DefaultTerminal, Frame,
};
use tui_logger::*;

fn main() -> std::io::Result<()> {
    init_logger(LevelFilter::Trace).unwrap();
    set_default_level(LevelFilter::Trace);

    // Background task producing log messages of different lengths, so that
    // scrolling through wrapped messages can be tried out as well.
    thread::spawn(|| {
        let mut i = 0u64;
        loop {
            match i % 7 {
                0 => warn!(target: "worker", "Message {i}: {}", "long message ".repeat(12)),
                3 => debug!(target: "worker", "Message {i}: multi\nline\nmessage"),
                _ => info!(target: "worker", "Message {i}"),
            }
            i += 1;
            thread::sleep(Duration::from_millis(300));
        }
    });

    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> std::io::Result<()> {
    let state = TuiWidgetState::new();
    loop {
        terminal.draw(|frame| draw(frame, &state))?;
        // Redraw periodically to show new log messages
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Up | KeyCode::Char('k') => state.transition(TuiWidgetEvent::PrevLineKey),
            KeyCode::Down | KeyCode::Char('j') => state.transition(TuiWidgetEvent::NextLineKey),
            KeyCode::PageUp => state.transition(TuiWidgetEvent::PrevPageKey),
            KeyCode::PageDown => state.transition(TuiWidgetEvent::NextPageKey),
            KeyCode::Esc => state.transition(TuiWidgetEvent::EscapeKey),
            _ => {}
        }
    }
}

fn draw(frame: &mut Frame, state: &TuiWidgetState) {
    let [log_area, help_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    let widget = TuiLoggerWidget::default()
        .block(Block::bordered().title("Log"))
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::Cyan))
        .style_debug(Style::default().fg(Color::Green))
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_target(true)
        .output_file(false)
        .output_line(false)
        .state(state);
    frame.render_widget(widget, log_area);
    frame.render_widget(
        Line::from("↑/k ↓/j: Scroll line | PageUp/Down: Scroll page | Esc: Follow log | q: Quit")
            .style(Color::Gray)
            .centered(),
        help_area,
    );
}
