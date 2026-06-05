use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
};

use crate::{
    exit_handler::ExitHandler,
    footer::Footer,
    menu_screen::MenuScreen,
    screen::{Screen, ScreenCommand},
    theme::Theme,
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct App {
    theme: Theme,
    active_screen: Box<dyn Screen>,
    exit_handler: ExitHandler,
    footer: Footer,
}

impl App {
    pub fn new() -> Self {
        let full_cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("."));
        let cwd = replace_homedir::replace_homedir(&full_cwd, "~");
        let theme = Theme::dark();

        Self {
            theme,
            active_screen: Box::new(MenuScreen::new(theme)),
            exit_handler: ExitHandler::new(Duration::from_secs(1)),
            footer: Footer::new(cwd, APP_VERSION.to_string(), theme),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            if self.exit_handler.should_exit() {
                return Ok(());
            }

            self.exit_handler.tick();

            let hint = self.exit_handler.hint_text();

            terminal.draw(|frame| {
                let area = frame.area();

                frame.render_widget(
                    Block::new().style(Style::default().bg(self.theme.bg)),
                    area,
                );

                let [body, footer_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(area);

                self.active_screen.render(frame, body);
                self.footer.render(frame, footer_area, hint);
            })?;

            if !event::poll(self.exit_handler.poll_timeout())? {
                continue;
            }
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.dispatch_key(key);
            }
        }
    }

    fn dispatch_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit_handler.press_ctrl_c();
            return;
        }

        if let Some(cmd) = self.active_screen.handle_key(key) {
            self.exit_handler.reset();
            match cmd {
                ScreenCommand::Stay => {}
                ScreenCommand::Navigate(screen) => self.active_screen = screen,
            }
        }
    }
}
