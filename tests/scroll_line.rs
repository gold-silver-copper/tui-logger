use log::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_logger::*;

const WIDTH: u16 = 20;

fn draw(state: &TuiWidgetState, height: u16) -> Vec<String> {
    let area = Rect::new(0, 0, WIDTH, height);
    let mut buf = Buffer::empty(area);
    TuiLoggerWidget::default()
        .output_timestamp(None)
        .output_level(None)
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(state)
        .render(area, &mut buf);
    (0..height)
        .map(|y| {
            (0..WIDTH)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// Sends `event` `n` times, drawing in between like an application would, and
/// returns how many times the view moved by exactly one line.
fn press(state: &TuiWidgetState, height: u16, event: TuiWidgetEvent, n: usize) -> usize {
    let h = height as usize;
    let mut moved = 0;
    for _ in 0..n {
        let before = draw(state, height);
        state.transition(event);
        let after = draw(state, height);
        if after == before {
            continue;
        }
        let one_line = match event {
            TuiWidgetEvent::PrevLineKey => after[1..] == before[..h - 1],
            _ => after[..h - 1] == before[1..],
        };
        assert!(
            one_line,
            "{event:?}: not a one line step\n{before:?}\n{after:?}"
        );
        moved += 1;
    }
    moved
}

#[test]
fn test_scroll_line() {
    init_logger(LevelFilter::Trace).unwrap();
    set_default_level(LevelFilter::Trace);

    for i in 0..20 {
        info!("line {i:02}");
    }
    // wraps into several lines at width 20
    info!("long {}", "x".repeat(40));
    for i in 20..25 {
        info!("line {i:02}");
    }
    move_events();
    let total = draw(&TuiWidgetState::new(), 100)
        .iter()
        .filter(|l| !l.is_empty())
        .count();
    assert!(total > 26, "long message should wrap");

    for height in [3u16, 4, 5, 8, 11] {
        let state = TuiWidgetState::new();
        let span = total - height as usize;
        let bottom = draw(&state, height);

        // Up to the top, one line at a time, then stay there instead of wrapping
        assert_eq!(
            press(&state, height, TuiWidgetEvent::PrevLineKey, span + 5),
            span
        );
        assert_eq!(draw(&state, height)[0], "line 00");

        // Back down to the bottom, then return to following the log
        assert_eq!(
            press(&state, height, TuiWidgetEvent::NextLineKey, span + 5),
            span
        );
        assert_eq!(draw(&state, height), bottom);
        assert_eq!(press(&state, height, TuiWidgetEvent::PrevLineKey, 3), 3);
        assert_eq!(press(&state, height, TuiWidgetEvent::NextLineKey, 3), 3);
        state.transition(TuiWidgetEvent::NextLineKey);
        assert_eq!(draw(&state, height), bottom);
    }

    // The logger is global, so this runs in the same test to not interfere
    let state = TuiWidgetState::new();
    assert_eq!(press(&state, 4, TuiWidgetEvent::PrevLineKey, 2), 2);
    info!("line new");
    move_events();
    // Scrolled back: the new event does not move the view
    assert_ne!(draw(&state, 4)[3], "line new");
    press(&state, 4, TuiWidgetEvent::NextLineKey, 3);
    // Past the bottom the widget follows the log again
    assert_eq!(draw(&state, 4)[3], "line new");
}
