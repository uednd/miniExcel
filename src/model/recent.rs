use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const RECENT_LIMIT: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentFile {
    pub path: PathBuf,
    pub name: String,
    pub opened_at: u64,
}

/// 最近打开文件的数据源。
///
/// App 持有该类型；UI 只读取 `items()` 并通过命令请求修改。
pub struct RecentFiles {
    storage_path: PathBuf,
    items: Vec<RecentFile>,
}

impl RecentFiles {
    pub fn load() -> Self {
        let storage_path = recent_json_path();
        let items = fs::read_to_string(&storage_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Vec<RecentFile>>(&raw).ok())
            .map(normalize_items)
            .unwrap_or_default();

        Self {
            storage_path,
            items,
        }
    }

    pub fn items(&self) -> &[RecentFile] {
        &self.items
    }

    pub fn add(&mut self, path: impl AsRef<Path>) {
        let path = stable_path(path.as_ref());
        self.items
            .retain(|file| !same_stable_path(&file.path, &path));
        self.items.insert(
            0,
            RecentFile {
                name: display_name(&path),
                path,
                opened_at: now_secs(),
            },
        );
        self.items.truncate(RECENT_LIMIT);
        self.save();
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) {
        let path = stable_path(path.as_ref());
        self.items
            .retain(|file| !same_stable_path(&file.path, &path));
        self.save();
    }

    fn save(&self) {
        if let Some(parent) = self.storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.items) {
            let _ = fs::write(&self.storage_path, json);
        }
    }
}

pub fn recent_json_path() -> PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mini-excel")
        .join("recent.json")
}

fn normalize_items(mut items: Vec<RecentFile>) -> Vec<RecentFile> {
    items.sort_by(|a, b| b.opened_at.cmp(&a.opened_at));

    let mut unique = Vec::new();
    for item in items {
        let path = stable_path(&item.path);
        if unique.iter().any(|file: &RecentFile| file.path == path) {
            continue;
        }
        unique.push(RecentFile {
            name: display_name(&path),
            path,
            opened_at: item.opened_at,
        });
        if unique.len() == RECENT_LIMIT {
            break;
        }
    }
    unique
}

fn stable_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn same_stable_path(candidate: &Path, stable: &Path) -> bool {
    stable_path(candidate) == stable
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled.mxlsx")
        .to_string()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
