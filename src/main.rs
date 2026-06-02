//! mini-excel —— 基于 Ratatui 的 TUI 迷你 Excel 应用。
//!
//! 按两次 Ctrl+C 退出。

mod logo;

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::Rect, style::Stylize, widgets::Paragraph};

const EXIT_CONFIRM_TIMEOUT: Duration = Duration::from_secs(1);

/// Ctrl+C 退出状态。
#[derive(Debug, Clone, Copy)]
enum ExitState {
    Idle,
    ConfirmOnce(Instant),
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(mini_excel)?;
    Ok(())
}

/// 运行主事件循环。
///
/// 每次迭代渲染 logo 并监听键盘事件。
/// 1 秒内连按两次 Ctrl+C 退出程序，按其他任意键则重置状态。
fn mini_excel(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut state = ExitState::Idle;

    loop {
        if let ExitState::ConfirmOnce(first_pressed) = state
            && first_pressed.elapsed() >= EXIT_CONFIRM_TIMEOUT
        {
            state = ExitState::Idle;
        }

        terminal.draw(|frame| render(frame, state))?;

        // 根据当前状态决定【最多等待多久的用户输入】
        let timeout = match state {
            ExitState::Idle => Duration::from_millis(250),
            ExitState::ConfirmOnce(first_pressed) => {
                EXIT_CONFIRM_TIMEOUT.saturating_sub(first_pressed.elapsed())
            }
        };

        if !event::poll(timeout)? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let now = Instant::now();

                    match state {
                        ExitState::Idle => {
                            state = ExitState::ConfirmOnce(now);
                        }

                        ExitState::ConfirmOnce(first_pressed)
                            if now.duration_since(first_pressed) < EXIT_CONFIRM_TIMEOUT =>
                        {
                            break Ok(());
                        }

                        ExitState::ConfirmOnce(_) => {
                            state = ExitState::ConfirmOnce(now);
                        }
                    }
                }

                _ => {
                    state = ExitState::Idle;
                }
            },

            _ => {}
        }
    }
}

/// 在帧上绘制 logo。处于 ConfirmOnce 状态时显示退出提示。
fn render(frame: &mut Frame, state: ExitState) {
    let area = frame.area();
    let art_area = logo::render(frame, area);

    if let ExitState::ConfirmOnce(_) = state {
        let hint = "再次按下 Ctrl+C 以退出";
        let hint_w = hint.len() as u16;
        let hint_x = area.x + (area.width.saturating_sub(hint_w)) / 2;
        let hint_y = art_area.y + art_area.height;
        let hint_rect = Rect::new(hint_x, hint_y, hint_w, 1);
        let hint_paragraph = Paragraph::new(hint).yellow().bold();

        frame.render_widget(hint_paragraph, hint_rect);
    }
}
