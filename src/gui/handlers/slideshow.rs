use crate::core::async_bridge;
use crate::core::wallpaper;
use crate::gui::message::SlideshowMessage;
use crate::gui::state::SlideshowState;
use iced::Task;
use std::time::Duration;

pub fn update(state: &mut SlideshowState, message: SlideshowMessage) -> Task<SlideshowMessage> {
    match message {
        SlideshowMessage::StartSlideshowPressed => Task::none(),
        SlideshowMessage::SelectListForSlideshow(_list_name) => Task::none(),
        SlideshowMessage::ChangeIntervalInput(_interval) => Task::none(),
        SlideshowMessage::ConfirmStartSlideshow => Task::none(),
        SlideshowMessage::CancelSlideshowDialog => Task::none(),
        SlideshowMessage::StartWithList(list_name, interval_str) => {
            let interval_secs = match parse_interval(&interval_str) {
                Ok(secs) => secs,
                Err(e) => {
                    state.message = Some(format!("Invalid interval: {}", e));

                    return Task::none();
                }
            };

            let list_name_clone = list_name.clone();

            state.run_id = state.run_id.wrapping_add(1);

            let run_id = state.run_id;

            state.is_active = true;

            state.is_paused = false;

            state.current_list = Some(list_name.clone());

            state.current_interval = interval_secs;

            state.cycle_count = 0;

            state.playlist_paths.clear();

            state.current_index = 0;

            state.currently_playing = None;

            state.message = Some(format!(
                "Starting slideshow: {} ({}s interval)",
                list_name, interval_secs
            ));

            Task::perform(
                async move {
                    let wallpaper_dir = wallpaper::wallpaper_dir()?;

                    let items = async_bridge::get_list_items_async(&list_name_clone)
                        .await
                        .map_err(|e| e)?;

                    let paths: Vec<std::path::PathBuf> = items
                        .into_iter()
                        .filter_map(|item| item.relative_path)
                        .map(|relative_path| {
                            if relative_path.is_absolute() {
                                relative_path
                            } else {
                                wallpaper_dir.join(relative_path)
                            }
                        })
                        .filter(|path| path.exists())
                        .collect();

                    if paths.is_empty() {
                        return Err(crate::error::SwpError::GuiLaunch {
                            message: "No valid wallpaper in list".to_string(),
                        });
                    }

                    Ok(paths)
                },
                move |result| SlideshowMessage::PlaylistLoaded(run_id, result),
            )
        }
        SlideshowMessage::PlaylistLoaded(run_id, result) => {
            if run_id != state.run_id || !state.is_active {
                return Task::none();
            }

            match result {
                Ok(paths) => {
                    state.playlist_paths = paths;

                    state.current_index = 0;

                    Task::done(SlideshowMessage::ApplyCurrent(run_id))
                }
                Err(e) => {
                    let msg = format!("Slideshow error: {}", e);

                    state.reset();

                    state.message = Some(msg);

                    Task::none()
                }
            }
        }
        SlideshowMessage::ApplyCurrent(run_id) => {
            if run_id != state.run_id || !state.is_active || state.is_paused {
                return Task::none();
            }

            if state.playlist_paths.is_empty() {
                let msg = "Slideshow has no playable items.".to_string();

                state.reset();

                state.message = Some(msg);

                return Task::none();
            }

            if state.current_index >= state.playlist_paths.len() {
                state.current_index = 0;
            }

            let path = state.playlist_paths[state.current_index].clone();

            Task::perform(
                async move { async_bridge::apply_wallpaper_async(&path).await },
                move |result| SlideshowMessage::AppliedCurrent(run_id, result),
            )
        }
        SlideshowMessage::AppliedCurrent(run_id, result) => {
            if run_id != state.run_id || !state.is_active {
                return Task::none();
            }

            match result {
                Ok(path) => {
                    state.currently_playing = Some(path.clone());

                    state.cycle_count = state.cycle_count.saturating_add(1);

                    state.message = Some(format!("Applied: {}", path.display()));

                    if state.is_paused {
                        return Task::none();
                    }

                    let interval_secs = state.current_interval;

                    Task::perform(
                        async move {
                            let _ =
                                async_bridge::sleep_async(Duration::from_secs(interval_secs)).await;
                        },
                        move |_| SlideshowMessage::Tick(run_id),
                    )
                }
                Err(e) => {
                    let msg = format!("Slideshow error: {}", e);

                    state.reset();

                    state.message = Some(msg);

                    Task::none()
                }
            }
        }
        SlideshowMessage::Tick(run_id) => {
            if run_id != state.run_id || !state.is_active || state.is_paused {
                return Task::none();
            }

            if state.playlist_paths.is_empty() {
                let msg = "Slideshow has no playable items.".to_string();

                state.reset();

                state.message = Some(msg);

                return Task::none();
            }

            state.current_index = (state.current_index + 1) % state.playlist_paths.len();

            Task::done(SlideshowMessage::ApplyCurrent(run_id))
        }
        SlideshowMessage::PausePressed => {
            if state.is_active && !state.is_paused {
                state.is_paused = true;

                state.message = Some("Slideshow paused".to_string());

                Task::none()
            } else {
                Task::none()
            }
        }
        SlideshowMessage::ResumePressed => {
            if state.is_active && state.is_paused {
                state.is_paused = false;

                state.message = Some("Slideshow resumed".to_string());

                if !state.playlist_paths.is_empty() {
                    if state.currently_playing.is_some() {
                        state.current_index =
                            (state.current_index + 1) % state.playlist_paths.len();
                    }

                    Task::done(SlideshowMessage::ApplyCurrent(state.run_id))
                } else {
                    state.message = Some("Cannot resume: empty playlist.".to_string());

                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        SlideshowMessage::StopPressed => {
            if state.is_active {
                state.run_id = state.run_id.wrapping_add(1);

                state.reset();

                state.message = Some("Slideshow stopped".to_string());
            }

            Task::none()
        }
        SlideshowMessage::IntervalChanged(interval_str) => {
            match parse_interval(&interval_str) {
                Ok(secs) => {
                    state.current_interval = secs;

                    state.message = Some(format!("Interval set to {}s", secs));
                }
                Err(e) => {
                    state.message = Some(format!("Invalid interval: {}", e));
                }
            }

            Task::none()
        }
    }
}

fn parse_interval(s: &str) -> Result<u64, String> {
    let s = s.trim();

    if s.ends_with('s') {
        let num_part = &s[..s.len() - 1];

        let secs = num_part
            .parse::<u64>()
            .map_err(|_| "Invalid seconds value".to_string())?;

        if secs == 0 {
            return Err("Interval must be greater than zero".to_string());
        }

        Ok(secs)
    } else if s.ends_with('m') {
        let num_part = &s[..s.len() - 1];

        let minutes: u64 = num_part
            .parse()
            .map_err(|_| "Invalid minutes value".to_string())?;

        if minutes == 0 {
            return Err("Interval must be greater than zero".to_string());
        }

        Ok(minutes * 60)
    } else {
        let secs = s
            .parse::<u64>()
            .map_err(|_| "Use format: 5s, 1m, or plain number".to_string())?;

        if secs == 0 {
            return Err("Interval must be greater than zero".to_string());
        }

        Ok(secs)
    }
}

pub fn validate_interval_input(interval: &str) -> Option<String> {
    parse_interval(interval).err().map(|err| err)
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]

    fn test_parse_interval_seconds() {
        assert_eq!(parse_interval("5s"), Ok(5));

        assert_eq!(parse_interval("30s"), Ok(30));
    }

    #[test]

    fn test_parse_interval_minutes() {
        assert_eq!(parse_interval("1m"), Ok(60));

        assert_eq!(parse_interval("5m"), Ok(300));
    }

    #[test]

    fn test_parse_interval_plain() {
        assert_eq!(parse_interval("10"), Ok(10));
    }

    #[test]

    fn test_parse_interval_invalid() {
        assert!(parse_interval("abc").is_err());

        assert!(parse_interval("5x").is_err());
    }
}
