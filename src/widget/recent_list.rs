use std::path::PathBuf;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

use crate::{model::recent::RecentFile, theme::Theme};

pub struct RecentList {
    files: Vec<RecentFile>,
    selected: usize,
}

impl RecentList {
    pub fn new(files: Vec<RecentFile>) -> Self {
        Self { files, selected: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.files.get(self.selected).map(|file| file.path.clone())
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.files.len() {
            self.selected += 1;
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        if self.files.is_empty() {
            let [_, empty_area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(area);

            frame.render_widget(
                Paragraph::new("暂无最近打开的文件")
                    .centered()
                    .style(Style::default().fg(theme.text_dim)),
                empty_area,
            );
            return;
        }

        let items: Vec<ListItem> = self
            .files
            .iter()
            .map(|file| recent_item(file, theme))
            .collect();

        let list = List::new(items)
            .highlight_symbol(Span::styled("▎", Style::default().fg(theme.accent)))
            .highlight_style(Style::default().fg(theme.text).bg(theme.surface));

        let mut state = ListState::default();
        state.select(Some(self.selected.min(self.files.len().saturating_sub(1))));
        frame.render_stateful_widget(list, area, &mut state);
    }
}

fn recent_item(file: &RecentFile, theme: Theme) -> ListItem<'static> {
    let exists = file.path.exists();
    let name_style = if exists {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_dim)
    };
    let path_style = Style::default().fg(theme.text_dim);
    let display_path = replace_homedir::replace_homedir(&file.path.display().to_string(), "~");

    ListItem::new(vec![
        Line::from(Span::styled(file.name.clone(), name_style)),
        Line::from(Span::styled(format!("  {}", display_path), path_style)),
    ])
}
