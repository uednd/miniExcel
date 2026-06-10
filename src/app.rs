use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
};

use crate::{
    exit::ExitHandler,
    screen::{EventResult, FrameState, Screen, ScreenCommand, home::MenuScreen},
    theme::Theme,
    widget::footer::Footer,
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct BlinkState {
    visible: bool,
    last_toggle: Instant,
}

impl BlinkState {
    pub fn new() -> Self {
        Self {
            visible: true,
            last_toggle: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        if self.last_toggle.elapsed() > Duration::from_millis(500) {
            self.visible = !self.visible;
            self.last_toggle = Instant::now();
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}

pub struct App {
    theme: Theme,
    cwd: String,
    active_screen: Box<dyn Screen>,
    exit_handler: ExitHandler,
    blink: BlinkState,
    footer: Footer,
}

impl App {
    pub fn new() -> Self {
        let full_cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("."));
        let display_cwd = replace_homedir::replace_homedir(&full_cwd, "~");
        let theme = Theme::dark();

        Self {
            theme,
            cwd: full_cwd.clone(),
            active_screen: Box::new(MenuScreen::new(theme, full_cwd)),
            exit_handler: ExitHandler::new(Duration::from_secs(1)),
            blink: BlinkState::new(),
            footer: Footer::new(display_cwd, APP_VERSION.to_string(), theme),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        // 注意：不要改动任何主循环顺序，已经调好了，否则可能造成程序阻塞
        loop {
            if self.exit_handler.should_exit() {
                return Ok(());
            }

            self.exit_handler.tick();
            self.blink.tick();

            let frame_state = FrameState {
                blink_visible: self.blink.visible(),
            };
            self.active_screen.pre_render(frame_state);

            let exit_hint = self.exit_handler.hint_text();
            let tip_hint = self.active_screen.footer_hint();
            let status_hint = self.active_screen.footer_status();

            terminal.draw(|frame| {
                let area = frame.area();
                // Background
                frame.render_widget(Block::new().style(Style::default().bg(self.theme.bg)), area);
                // Main Layout
                let [body, footer_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(area);
                // Active screen
                self.active_screen.render(frame, body);
                // Footer
                self.footer
                    .render(frame, footer_area, status_hint, tip_hint, exit_hint);
            })?;

            // 处理主循环中的输入事件
            if !event::poll(self.exit_handler.poll_timeout())? {
                continue;
            }
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    self.dispatch_key(key);
                }
                Event::Mouse(mouse)
                    if matches!(
                        mouse.kind,
                        MouseEventKind::ScrollUp
                            | MouseEventKind::ScrollDown
                            | MouseEventKind::ScrollLeft
                            | MouseEventKind::ScrollRight
                    ) =>
                {
                    match self.active_screen.handle_scroll(mouse) {
                        EventResult::Ignored => {}
                        EventResult::Handled => self.exit_handler.reset(),
                        EventResult::Command(cmd) => {
                            self.exit_handler.reset();
                            self.process_cmd(cmd);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn dispatch_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit_handler.press_ctrl_q();
            return;
        }
        match self.active_screen.handle_key(key) {
            EventResult::Ignored => {}
            EventResult::Handled => self.exit_handler.reset(),
            EventResult::Command(cmd) => {
                self.exit_handler.reset();
                self.process_cmd(cmd);
            }
        }
    }

    fn process_cmd(&mut self, cmd: ScreenCommand) {
        match cmd {
            ScreenCommand::OpenEditor { path } => {
                self.active_screen =
                    Box::new(super::screen::editor::TableScreen::new(self.theme, path));
            }
            ScreenCommand::GoHome => {
                self.active_screen = Box::new(MenuScreen::new(self.theme, self.cwd.clone()));
            }
        }
    }
}
