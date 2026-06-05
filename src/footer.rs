use unicode_width::UnicodeWidthStr;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
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

    pub fn render(&self, frame: &mut Frame, area: Rect, exit_hint: Option<&str>) {
        let block = Block::new().padding(Padding::new(2, 2, 0, 1));
        let inner = block.inner(area);

        let [path_area, hint_area, version_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let dim_style = Style::default().fg(self.theme.text_dim);
        let accent_style = Style::default().fg(self.theme.accent);
        let highlight_style = Style::default().fg(self.theme.text);

        let hint = Line::from(vec![
            Span::styled("● 提示", accent_style),
            Span::styled(" 使用 ", dim_style),
            Span::styled("←/→", highlight_style),
            Span::styled(" 切换标签", dim_style),
        ]);

        frame.render_widget(
            Paragraph::new(Line::from(self.current_dir.as_str()).style(dim_style)),
            path_area,
        );
        frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), hint_area);

        if let Some(text) = exit_hint {
            let exit_width = text.width() as u16 + 2;
            let [exit_hint_area, version_text_area] =
                Layout::horizontal([Constraint::Length(exit_width), Constraint::Fill(1)])
                    .areas(version_area);

            frame.render_widget(
                Paragraph::new(Line::from(text)).yellow().bold(),
                exit_hint_area,
            );
            frame.render_widget(
                Paragraph::new(Line::from(self.version.as_str()).style(dim_style))
                    .alignment(Alignment::Right),
                version_text_area,
            );
        } else {
            frame.render_widget(
                Paragraph::new(Line::from(self.version.as_str()).style(dim_style))
                    .alignment(Alignment::Right),
                version_area,
            );
        }
    }
}
