use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{PlaybackError, SwpError};

pub const PLAYBACK_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaybackMode {
    Filesystem,
    List { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub version: u32,
    pub pid: u32,
    pub active: bool,
    pub mode: PlaybackMode,
    pub interval_seconds: u64,
    pub started_at: u64,
    pub last_rotation: Option<u64>,
    pub last_wallpaper: Option<String>,
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub(crate) fn config_dir() -> Result<PathBuf, SwpError> {
    let home = std::env::var("HOME").map_err(|_| SwpError::HomeEnvMissing)?;
    Ok(PathBuf::from(home).join(".config/swp"))
}

pub(crate) fn state_path() -> Result<PathBuf, SwpError> {
    Ok(config_dir()?.join("playback.json"))
}

fn tmp_state_path(path: &Path) -> PathBuf {
    let pid = std::process::id();
    let stamp = now_unix_secs();
    path.with_extension(format!("tmp.{pid}.{stamp}"))
}

pub(crate) fn save_state(state: &PlaybackState) -> Result<(), SwpError> {
    let path = state_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PlaybackError::InvalidState {
            message: format!("failed to create {}: {e}", parent.display()),
        })?;
    }

    let serialized = serde_json::to_vec_pretty(state).map_err(|e| PlaybackError::InvalidState {
        message: format!("failed to serialize playback state: {e}"),
    })?;

    let tmp_path = tmp_state_path(&path);
    let mut file = fs::File::create(&tmp_path).map_err(|e| PlaybackError::InvalidState {
        message: format!("failed to create {}: {e}", tmp_path.display()),
    })?;
    file.write_all(&serialized)
        .map_err(|e| PlaybackError::InvalidState {
            message: format!("failed to write {}: {e}", tmp_path.display()),
        })?;
    file.sync_all().map_err(|e| PlaybackError::InvalidState {
        message: format!("failed to sync {}: {e}", tmp_path.display()),
    })?;

    fs::rename(&tmp_path, &path).map_err(|e| PlaybackError::InvalidState {
        message: format!("failed to replace {}: {e}", path.display()),
    })?;

    Ok(())
}

pub(crate) fn load_state() -> Result<Option<PlaybackState>, SwpError> {
    let path = state_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).map_err(|e| PlaybackError::InvalidState {
        message: format!("failed to read {}: {e}", path.display()),
    })?;

    let state: PlaybackState =
        serde_json::from_str(&content).map_err(|e| PlaybackError::InvalidState {
            message: format!("failed to parse {}: {e}", path.display()),
        })?;

    if state.version != PLAYBACK_SCHEMA_VERSION {
        return Err(PlaybackError::InvalidState {
            message: format!(
                "unsupported playback state version {} in {}",
                state.version,
                path.display()
            ),
        }
        .into());
    }

    if !state.active {
        return Ok(None);
    }

    Ok(Some(state))
}

pub(crate) fn clear_state() -> Result<(), SwpError> {
    let path = state_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(PlaybackError::InvalidState {
            message: format!("failed to remove {}: {err}", path.display()),
        }
        .into()),
    }
}

pub(crate) fn is_pid_alive(pid: u32) -> bool {
    let proc_dir = PathBuf::from("/proc").join(pid.to_string());
    proc_dir.exists()
}

pub(crate) fn build_state(pid: u32, mode: PlaybackMode, interval_seconds: u64) -> PlaybackState {
    PlaybackState {
        version: PLAYBACK_SCHEMA_VERSION,
        pid,
        active: true,
        mode,
        interval_seconds,
        started_at: now_unix_secs(),
        last_rotation: None,
        last_wallpaper: None,
    }
}
