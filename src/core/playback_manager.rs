use std::env;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::core::lists_play;
use crate::core::playback_state::{
    PlaybackMode, build_state, clear_state, is_pid_alive, load_state, save_state,
};
use crate::error::{PlaybackError, SwpError};

fn spawn_worker(mode: &PlaybackMode, interval: &str) -> Result<u32, SwpError> {
    let exe = env::current_exe().map_err(|e| PlaybackError::SpawnError {
        message: format!("failed to resolve current executable: {e}"),
    })?;

    let mut command = Command::new(exe);
    command
        .arg("lists")
        .arg("play")
        .arg("--foreground")
        .arg("--interval")
        .arg(interval);

    if let PlaybackMode::List { name } = mode {
        command.arg(name);
    }

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let child = command.spawn().map_err(|e| PlaybackError::SpawnError {
        message: format!("failed to start playback worker: {e}"),
    })?;

    Ok(child.id())
}

fn kill_pid(pid: u32) -> Result<(), SwpError> {
    #[cfg(unix)]
    {
        let raw_pid = pid as i32;
        let term_result = unsafe { libc::kill(raw_pid, libc::SIGTERM) };

        if term_result == -1 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::NotFound {
                return Ok(());
            }

            return Err(PlaybackError::KillError {
                pid,
                message: err.to_string(),
            }
            .into());
        }

        for _ in 0..20 {
            if !is_pid_alive(pid) {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let kill_result = unsafe { libc::kill(raw_pid, libc::SIGKILL) };
        if kill_result == -1 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::NotFound {
                return Ok(());
            }

            return Err(PlaybackError::KillError {
                pid,
                message: err.to_string(),
            }
            .into());
        }

        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(PlaybackError::KillError {
        pid,
        message: "process control is only supported on Unix-like systems".to_string(),
    }
    .into())
}

fn stop_existing_if_any() -> Result<(), SwpError> {
    let state = match load_state() {
        Ok(Some(state)) => state,
        Ok(None) => {
            clear_state()?;
            return Ok(());
        }
        Err(SwpError::Playback(PlaybackError::InvalidState { .. })) => {
            clear_state()?;
            return Ok(());
        }
        Err(err) => return Err(err),
    };

    if is_pid_alive(state.pid) {
        kill_pid(state.pid)?;
    }

    clear_state()?;
    Ok(())
}

pub fn start_playback(name: Option<String>, interval: &str) -> Result<u32, SwpError> {
    stop_existing_if_any()?;

    let mode = match name {
        Some(name) => PlaybackMode::List { name },
        None => PlaybackMode::Filesystem,
    };

    let duration = lists_play::parse_interval(interval)?;
    let pid = spawn_worker(&mode, interval)?;
    let state = build_state(pid, mode, duration.as_secs());
    save_state(&state)?;

    Ok(pid)
}

pub fn stop_playback() -> Result<(), SwpError> {
    let state = match load_state() {
        Ok(Some(state)) => state,
        Ok(None) => {
            clear_state()?;
            return Err(PlaybackError::NoActivePlayback.into());
        }
        Err(SwpError::Playback(PlaybackError::InvalidState { .. })) => {
            clear_state()?;
            return Err(PlaybackError::NoActivePlayback.into());
        }
        Err(err) => return Err(err),
    };

    if is_pid_alive(state.pid) {
        kill_pid(state.pid)?;
    }

    clear_state()?;
    Ok(())
}

#[allow(clippy::manual_is_multiple_of)]
fn format_interval(interval_seconds: u64) -> String {
    if interval_seconds % 3600 == 0 {
        format!("{}h", interval_seconds / 3600)
    } else if interval_seconds % 60 == 0 {
        format!("{}m", interval_seconds / 60)
    } else {
        format!("{}s", interval_seconds)
    }
}

pub fn resume_playback() -> Result<(), SwpError> {
    let state = match load_state() {
        Ok(Some(state)) => state,
        Ok(None) => return Ok(()),
        Err(SwpError::Playback(PlaybackError::InvalidState { .. })) => {
            clear_state()?;
            return Ok(());
        }
        Err(err) => return Err(err),
    };

    if is_pid_alive(state.pid) {
        return Ok(());
    }

    let mode = state.mode.clone();
    let interval = format_interval(state.interval_seconds);
    let pid = spawn_worker(&mode, &interval)?;

    let mut resumed = build_state(pid, mode, state.interval_seconds);
    resumed.last_rotation = state.last_rotation;
    resumed.last_wallpaper = state.last_wallpaper;
    save_state(&resumed)?;

    Ok(())
}
