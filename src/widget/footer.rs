use unicode_width::UnicodeWidthStr;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Padding, Paragraph},
};

use crate::theme::Theme;

pub struct Footer {
    current_dir: String,
    version: String,
    theme: Theme,
}

impl Footer {
    pub fn new(current_dir: String, version: String, theme: Theme) -> Self {
        Self {
            current_dir,
            version,
            theme,
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        status_hint: Option<Line<'_>>,
        tip_hint: Option<Line<'_>>,
        exit_hint: Option<&str>,
    ) {
        let block = Block::new().padding(Padding::new(2, 2, 0, 1));
        let inner = block.inner(area);
        let status_width = status_hint
            .as_ref()
            .map_or(0, |line| line.width() as u16 + 2);
        let [path_area, status_area, hint_area, version_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(status_width),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let dim_style = Style::default().fg(self.theme.text_dim);

        // CWD
        frame.render_widget(
            Paragraph::new(Line::from(self.current_dir.as_str()).style(dim_style)),
            path_area,
        );

        // Tip hint
        if let Some(hint) = tip_hint {
            frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), hint_area);
        }

        // Current screen status hint
        if let Some(status) = status_hint {
            frame.render_widget(
                Paragraph::new(status).alignment(Alignment::Center),
                status_area,
            );
        }

        // Exit hint
        let exit_width = exit_hint.map_or(0, |t| t.width() as u16 + 2);
        let [exit_hint_area, version_text_area] =
            Layout::horizontal([Constraint::Length(exit_width), Constraint::Fill(1)])
                .areas(version_area);
        if let Some(text) = exit_hint {
            frame.render_widget(
                Paragraph::new(Line::from(text)).yellow().bold(),
                exit_hint_area,
            );
        }

        // Version
        frame.render_widget(
            Paragraph::new(Line::from(self.version.as_str()).style(dim_style))
                .alignment(Alignment::Right),
            version_text_area,
        );
    }
}
