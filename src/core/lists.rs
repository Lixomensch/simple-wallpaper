use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::RngExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::wallpaper;
use crate::error::{ListError, SwpError};

const LISTS_SCHEMA_VERSION: u32 = 1;
const INDEX_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct WallpaperList {
    pub name: String,
    pub wallpapers: Vec<Uuid>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub struct ListSummary {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub id: Uuid,
    pub relative_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ListPlayer {
    list_name: String,
    all_ids: Vec<Uuid>,
    cycle_queue: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WallpaperListFile {
    name: String,
    wallpapers: Vec<Uuid>,
    created_at: u64,
    updated_at: u64,
    version: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct WallpaperIndexFile {
    version: u32,
    by_id: BTreeMap<String, String>,
    by_path: BTreeMap<String, String>,
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn lists_dir() -> Result<PathBuf, SwpError> {
    let home = std::env::var("HOME").map_err(|_| SwpError::HomeEnvMissing)?;
    let dir = PathBuf::from(home).join(".config/swp/lists");
    fs::create_dir_all(&dir).map_err(|e| ListError::Storage {
        message: format!("failed to create lists directory {}: {e}", dir.display()),
    })?;
    Ok(dir)
}

fn index_path() -> Result<PathBuf, SwpError> {
    let base = wallpaper::wallpaper_dir()?;
    let app_dir = base.parent().ok_or_else(|| ListError::Index {
        message: format!("invalid wallpapers base path: {}", base.display()),
    })?;
    fs::create_dir_all(app_dir).map_err(|e| ListError::Index {
        message: format!("failed to create app data dir {}: {e}", app_dir.display()),
    })?;
    Ok(app_dir.join("wallpaper-index.json"))
}

fn normalize_list_name(name: &str) -> Result<String, SwpError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ListError::InvalidName {
            name: name.to_string(),
        }
        .into());
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ListError::InvalidName {
            name: name.to_string(),
        }
        .into());
    }

    Ok(trimmed.to_string())
}

fn list_file_path(name: &str) -> Result<PathBuf, SwpError> {
    let normalized = normalize_list_name(name)?;
    Ok(lists_dir()?.join(format!("{normalized}.json")))
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn to_relative_wallpaper_path(abs_path: &Path) -> Result<PathBuf, SwpError> {
    let base = wallpaper::wallpaper_dir()?;
    match abs_path.strip_prefix(&base) {
        Ok(relative) => Ok(relative.to_path_buf()),
        Err(_) => Err(ListError::WallpaperOutsideBase {
            path: abs_path.to_path_buf(),
            base,
        }
        .into()),
    }
}

fn to_absolute_wallpaper_path(relative_path: &Path) -> Result<PathBuf, SwpError> {
    Ok(wallpaper::wallpaper_dir()?.join(relative_path))
}

fn read_json_or_default<T>(path: &Path) -> Result<T, SwpError>
where
    T: for<'de> Deserialize<'de> + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }

    let content = fs::read_to_string(path).map_err(|e| ListError::Storage {
        message: format!("failed to read {}: {e}", path.display()),
    })?;

    serde_json::from_str(&content).map_err(|e| {
        ListError::Storage {
            message: format!("failed to parse {}: {e}", path.display()),
        }
        .into()
    })
}

fn write_json_atomic<T>(path: &Path, value: &T) -> Result<(), SwpError>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ListError::Storage {
            message: format!(
                "failed to create parent directory {}: {e}",
                parent.display()
            ),
        })?;
    }

    let pid = std::process::id();
    let stamp = now_unix_secs();
    let tmp_path = path.with_extension(format!("tmp.{pid}.{stamp}"));

    let serialized = serde_json::to_vec_pretty(value).map_err(|e| ListError::Storage {
        message: format!("failed to serialize JSON for {}: {e}", path.display()),
    })?;

    let mut file = fs::File::create(&tmp_path).map_err(|e| ListError::Storage {
        message: format!("failed to create temp file {}: {e}", tmp_path.display()),
    })?;
    file.write_all(&serialized)
        .map_err(|e| ListError::Storage {
            message: format!("failed writing temp file {}: {e}", tmp_path.display()),
        })?;
    file.sync_all().map_err(|e| ListError::Storage {
        message: format!("failed syncing temp file {}: {e}", tmp_path.display()),
    })?;

    fs::rename(&tmp_path, path).map_err(|e| ListError::Storage {
        message: format!(
            "failed to atomically replace {} with {}: {e}",
            path.display(),
            tmp_path.display()
        ),
    })?;

    Ok(())
}

fn load_index() -> Result<WallpaperIndexFile, SwpError> {
    let path = index_path()?;
    let mut index: WallpaperIndexFile = read_json_or_default(&path)?;
    if index.version == 0 {
        index.version = INDEX_SCHEMA_VERSION;
    }
    Ok(index)
}

fn save_index(index: &WallpaperIndexFile) -> Result<(), SwpError> {
    let path = index_path()?;
    write_json_atomic(&path, index)
}

fn get_or_create_uuid_with_index(
    abs_path: &Path,
    index: &mut WallpaperIndexFile,
) -> Result<Uuid, SwpError> {
    let relative = to_relative_wallpaper_path(abs_path)?;
    let relative_key = normalize_relative_path(&relative);

    if let Some(id_str) = index.by_path.get(&relative_key) {
        return Uuid::parse_str(id_str).map_err(|e| {
            ListError::Index {
                message: format!("invalid UUID in index for path '{relative_key}': {e}"),
            }
            .into()
        });
    }

    let id = Uuid::new_v4();
    let id_str = id.to_string();
    index.by_path.insert(relative_key.clone(), id_str.clone());
    index.by_id.insert(id_str, relative_key);
    index.version = INDEX_SCHEMA_VERSION;
    Ok(id)
}

fn resolve_uuid_to_path_with_index(
    id: &Uuid,
    index: &WallpaperIndexFile,
) -> Result<Option<PathBuf>, SwpError> {
    let id_key = id.to_string();
    let Some(relative_str) = index.by_id.get(&id_key) else {
        return Ok(None);
    };

    let abs = to_absolute_wallpaper_path(Path::new(relative_str))?;
    Ok(Some(abs))
}

fn resolve_uuid_to_relative_path_with_index(
    id: &Uuid,
    index: &WallpaperIndexFile,
) -> Result<Option<PathBuf>, SwpError> {
    let id_key = id.to_string();
    let Some(relative_str) = index.by_id.get(&id_key) else {
        return Ok(None);
    };

    Ok(Some(PathBuf::from(relative_str)))
}

fn load_list(name: &str) -> Result<WallpaperList, SwpError> {
    let path = list_file_path(name)?;
    if !path.exists() {
        return Err(ListError::NotFound {
            name: name.to_string(),
        }
        .into());
    }

    let content = fs::read_to_string(&path).map_err(|e| ListError::Storage {
        message: format!("failed to read {}: {e}", path.display()),
    })?;

    let parsed: WallpaperListFile =
        serde_json::from_str(&content).map_err(|e| ListError::Storage {
            message: format!("failed to parse {}: {e}", path.display()),
        })?;

    Ok(WallpaperList {
        name: parsed.name,
        wallpapers: parsed.wallpapers,
        created_at: parsed.created_at,
        updated_at: parsed.updated_at,
    })
}

fn save_list(list: &WallpaperList) -> Result<(), SwpError> {
    let path = list_file_path(&list.name)?;
    let file = WallpaperListFile {
        name: list.name.clone(),
        wallpapers: list.wallpapers.clone(),
        created_at: list.created_at,
        updated_at: list.updated_at,
        version: LISTS_SCHEMA_VERSION,
    };

    write_json_atomic(&path, &file)
}

pub fn create_list(name: &str) -> Result<(), SwpError> {
    let normalized = normalize_list_name(name)?;

    if load_list(&normalized).is_ok() {
        return Err(ListError::AlreadyExists { name: normalized }.into());
    }

    let ts = now_unix_secs();
    let list = WallpaperList {
        name: normalized,
        wallpapers: Vec::new(),
        created_at: ts,
        updated_at: ts,
    };

    save_list(&list)
}

pub fn delete_list(name: &str) -> Result<(), SwpError> {
    let normalized = normalize_list_name(name)?;
    let path = list_file_path(&normalized)?;

    if !path.exists() {
        return Err(ListError::NotFound { name: normalized }.into());
    }

    fs::remove_file(&path).map_err(|e| ListError::Storage {
        message: format!("failed to remove {}: {e}", path.display()),
    })?;

    Ok(())
}

pub fn list_lists() -> Result<Vec<ListSummary>, SwpError> {
    let dir = lists_dir()?;
    let mut summaries = Vec::new();

    let entries = fs::read_dir(&dir).map_err(|e| ListError::Storage {
        message: format!("failed to read {}: {e}", dir.display()),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ListError::Storage {
            message: format!("failed to iterate list files: {e}"),
        })?;

        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        let Ok(parsed) = serde_json::from_str::<WallpaperListFile>(&content) else {
            continue;
        };

        summaries.push(ListSummary {
            name: parsed.name,
            count: parsed.wallpapers.len(),
        });
    }

    summaries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(summaries)
}

pub fn get_list(name: &str) -> Result<WallpaperList, SwpError> {
    load_list(name)
}

pub fn get_list_items(name: &str) -> Result<Vec<ListItem>, SwpError> {
    let list = load_list(name)?;
    get_list_items_from_list(&list)
}

pub fn get_list_items_from_list(list: &WallpaperList) -> Result<Vec<ListItem>, SwpError> {
    let index = load_index()?;
    let mut items = Vec::with_capacity(list.wallpapers.len());

    for &id in &list.wallpapers {
        items.push(ListItem {
            id,
            relative_path: resolve_uuid_to_relative_path_with_index(&id, &index)?,
        });
    }

    Ok(items)
}

pub fn add_wallpapers_by_paths(name: &str, paths: &[PathBuf]) -> Result<usize, SwpError> {
    let mut list = load_list(name)?;
    let mut index = load_index()?;
    let existing: std::collections::HashSet<Uuid> = list.wallpapers.iter().copied().collect();
    let mut added = 0usize;

    for path in paths {
        let id = get_or_create_uuid_with_index(path, &mut index)?;
        if existing.contains(&id) {
            return Err(ListError::AlreadyInList {
                name: name.to_string(),
                wallpaper: path.display().to_string(),
            }
            .into());
        }

        list.wallpapers.push(id);
        added += 1;
    }

    save_index(&index)?;
    list.updated_at = now_unix_secs();
    save_list(&list)?;
    Ok(added)
}

pub fn remove_wallpapers(name: &str, paths: &[PathBuf], ids: &[Uuid]) -> Result<usize, SwpError> {
    let mut list = load_list(name)?;
    let original_len = list.wallpapers.len();
    let index = load_index()?;

    let mut to_remove: std::collections::HashSet<Uuid> = ids.iter().copied().collect();

    for path in paths {
        let relative = to_relative_wallpaper_path(path)?;
        let key = normalize_relative_path(&relative);
        if let Some(id_str) = index.by_path.get(&key)
            && let Ok(id) = Uuid::parse_str(id_str)
        {
            to_remove.insert(id);
        }
    }

    list.wallpapers.retain(|item| !to_remove.contains(item));

    let removed = original_len.saturating_sub(list.wallpapers.len());
    list.updated_at = now_unix_secs();
    save_list(&list)?;
    Ok(removed)
}

fn fisher_yates_shuffle(ids: &[Uuid]) -> Vec<Uuid> {
    let mut result = ids.to_vec();
    let mut rng = rand::rng();

    if result.len() < 2 {
        return result;
    }

    for i in (1..result.len()).rev() {
        let j = rng.random_range(0..=i);
        result.swap(i, j);
    }

    result
}

impl ListPlayer {
    pub fn new(list_name: &str) -> Result<Self, SwpError> {
        let list = load_list(list_name)?;

        if list.wallpapers.is_empty() {
            return Err(ListError::Empty {
                name: list.name.clone(),
            }
            .into());
        }

        let cycle_queue = fisher_yates_shuffle(&list.wallpapers);
        Ok(Self {
            list_name: list.name,
            all_ids: list.wallpapers,
            cycle_queue,
        })
    }

    fn persist_current_ids(&self) -> Result<(), SwpError> {
        let mut list = load_list(&self.list_name)?;
        list.wallpapers = self.all_ids.clone();
        list.updated_at = now_unix_secs();
        save_list(&list)
    }

    pub fn next_wallpaper(&mut self) -> Result<PathBuf, SwpError> {
        let index = load_index()?;

        loop {
            if self.all_ids.is_empty() {
                return Err(ListError::Empty {
                    name: self.list_name.clone(),
                }
                .into());
            }

            if self.cycle_queue.is_empty() {
                self.cycle_queue = fisher_yates_shuffle(&self.all_ids);
            }

            let id = self
                .cycle_queue
                .pop()
                .expect("cycle_queue should not be empty");

            match resolve_uuid_to_path_with_index(&id, &index)? {
                Some(path) if path.exists() => return Ok(path),
                _ => {
                    self.all_ids.retain(|x| x != &id);
                    self.cycle_queue.retain(|x| x != &id);
                    self.persist_current_ids()?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fisher_yates_shuffle;
    use uuid::Uuid;

    #[test]
    fn fisher_yates_preserves_length() {
        let items = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let shuffled = fisher_yates_shuffle(&items);
        assert_eq!(items.len(), shuffled.len());
    }
}
