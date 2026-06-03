//! App 状态管理与事件循环。

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{layout::Rect, style::Stylize, widgets::Paragraph, DefaultTerminal, Frame};
use unicode_width::UnicodeWidthStr;

use crate::menu::Menu;

const EXIT_CONFIRM_TIMEOUT: Duration = Duration::from_secs(1);
const LOGO_MENU_GAP: u16 = 2;

#[derive(Debug)]
enum ExitState {
    Idle,
    ConfirmOnce(Instant),
    Confirmed,
}

pub struct App {
    menu: Menu,
    exit_state: ExitState,
}

impl App {
    pub fn new() -> Self {
        Self {
            menu: Menu::new(),
            exit_state: ExitState::Idle,
        }
    }

    /// 运行主事件循环。
    ///
    /// 每次迭代渲染界面并监听键盘事件。`←/→` 切换菜单选中项，1 秒内连按两次 `Ctrl+C` 退出程序。
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            // 连续按两次 => 退出程序
            if matches!(self.exit_state, ExitState::Confirmed) {
                return Ok(());
            }

            // 按一次 && 超时 => 退出状态改为空闲
            if let ExitState::ConfirmOnce(first) = self.exit_state
                && first.elapsed() >= EXIT_CONFIRM_TIMEOUT
            {
                self.exit_state = ExitState::Idle;
            }

            // 绘制界面
            terminal.draw(|frame| self.render(frame))?;

            // 按一次
            let timeout = match self.exit_state {
                ExitState::Idle | ExitState::Confirmed => Duration::from_millis(100),
                ExitState::ConfirmOnce(first) => {
                    EXIT_CONFIRM_TIMEOUT.saturating_sub(first.elapsed())
                }
            };

            // 按一次 && 超时 => 重新绘制界面
            if !event::poll(timeout)? {
                continue;
            }

            // 捕捉键盘事件
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key);
            }
        }
    }

    // 处理键盘事件
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Left => {
                self.menu.move_left();
                self.exit_state = ExitState::Idle;
            }
            KeyCode::Right => {
                self.menu.move_right();
                self.exit_state = ExitState::Idle;
            }
            KeyCode::Enter => {
                todo!();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let now = Instant::now();
                match self.exit_state {
                    ExitState::Idle => {
                        self.exit_state = ExitState::ConfirmOnce(now);
                    }
                    ExitState::ConfirmOnce(first)
                        if now.duration_since(first) < EXIT_CONFIRM_TIMEOUT =>
                    {
                        self.exit_state = ExitState::Confirmed;
                    }
                    ExitState::ConfirmOnce(_) => {
                        self.exit_state = ExitState::ConfirmOnce(now);
                    }
                    ExitState::Confirmed => {}
                }
            }
            KeyCode::Char(_) => {
                self.exit_state = ExitState::Idle;
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let art_area = crate::logo::render(frame, area);

        let menu_y = art_area.y + art_area.height + LOGO_MENU_GAP;
        let menu_origin = Rect::new(area.x, menu_y, area.width, 1);
        let menu_area = self.menu.render(frame, menu_origin);

        if let ExitState::ConfirmOnce(_) = self.exit_state {
            let hint = "再次按下 Ctrl+C 以退出";
            let hint_w = UnicodeWidthStr::width(hint) as u16;
            let hint_x = area.x + (area.width.saturating_sub(hint_w)) / 2;
            let hint_y = menu_area.y + menu_area.height + 1;
            let hint_rect = Rect::new(hint_x, hint_y, hint_w, 1);
            frame.render_widget(Paragraph::new(hint).yellow().bold(), hint_rect);
        }
    }
}
