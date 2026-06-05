use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::theme::Theme;

const TAB_LABELS: [&str; 3] = ["最近打开", "新建表格", "设置"];

pub struct Tabs {
    selected: usize,
    theme: Theme,
}

impl Tabs {
    pub fn new(theme: Theme) -> Self {
        Self {
            selected: 0,
            theme,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn next(&mut self) {
        if self.selected + 1 < TAB_LABELS.len() {
            self.selected += 1;
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks: [Rect; 3] =
            Layout::horizontal([Constraint::Fill(1); 3]).areas(area);

        let selected_style = Style::default()
            .fg(self.theme.accent_text)
            .bg(self.theme.accent)
            .add_modifier(Modifier::BOLD);
        let unselected_style = Style::default()
            .fg(self.theme.text)
            .bg(self.theme.surface_alt);

        for (i, label) in TAB_LABELS.iter().enumerate() {
            let style = if i == self.selected {
                selected_style
            } else {
                unselected_style
            };
            frame.render_widget(
                Paragraph::new(Line::from(*label)).centered().style(style),
                chunks[i],
            );
        }
    }
}
