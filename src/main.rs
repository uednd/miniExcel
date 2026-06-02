//! mini-excel —— 基于 Ratatui 的 TUI 迷你 Excel 应用。
//!
//! 按两次 Ctrl+C 退出。

mod logo;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::Rect, style::Stylize, widgets::Paragraph};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

/// 运行主事件循环。
///
/// 每次迭代渲染 logo 并监听键盘事件。
/// 需要连续按两次 Ctrl+C 才能退出，按其他任意键则重置计数。
fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut ctrl_c_count: u8 = 0;
    loop {
        terminal.draw(|frame| render(frame, ctrl_c_count))?;
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    ctrl_c_count += 1;
                    if ctrl_c_count >= 2 {
                        break Ok(());
                    }
                }
                _ => {
                    ctrl_c_count = 0;
                }
            },
            _ => {}
        }
    }
}

/// 在帧上绘制 logo。首次按下 Ctrl+C 后显示退出提示。
fn render(frame: &mut Frame, ctrl_c_count: u8) {
    let area = frame.area();
    let art_area = logo::render(frame, area);

    if ctrl_c_count >= 1 {
        let hint = "再次按下 Ctrl+C 以退出";
        let hint_w = hint.len() as u16;
        let hint_x = area.x + (area.width.saturating_sub(hint_w)) / 2;
        let hint_y = art_area.y + art_area.height;
        let hint_rect = Rect::new(hint_x, hint_y, hint_w, 1);
        let hint_paragraph = Paragraph::new(hint).yellow().bold();
        frame.render_widget(hint_paragraph, hint_rect);
    }
}
