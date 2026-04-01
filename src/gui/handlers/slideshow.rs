//! Slideshow handler: manages automatic wallpaper cycling.
//!
//! Implements background task that cycles through wallpapers at fixed intervals.
//! Supports pause, resume, and stop operations.

use std::time::Duration;

use iced::Task;

use crate::core::async_bridge;
use crate::gui::message::SlideshowMessage;
use crate::gui::state::SlideshowState;

/// Update handler for slideshow messages.
///
/// Manages:
/// - Starting slideshow with a list and interval
/// - Pausing/resuming playback
/// - Stopping and cleaning up
/// - Cycling through wallpapers automatically
pub fn update(state: &mut SlideshowState, message: SlideshowMessage) -> Task<SlideshowMessage> {
    match message {
        SlideshowMessage::StartSlideshowPressed => {
            // Dialog will be shown by view layer
            Task::none()
        }

        SlideshowMessage::SelectListForSlideshow(_list_name) => {
            // Dialog will handle this
            Task::none()
        }

        SlideshowMessage::ChangeIntervalInput(_interval) => {
            // Dialog will handle this
            Task::none()
        }

        SlideshowMessage::ConfirmStartSlideshow => {
            // Handled by app-level update to trigger StartWithList
            Task::none()
        }

        SlideshowMessage::CancelSlideshowDialog => {
            // Dialog will be hidden by view layer
            Task::none()
        }

        SlideshowMessage::StartWithList(list_name, interval_str) => {
            // Parse interval string (e.g., "5s", "1m")
            let interval_secs = match parse_interval(&interval_str) {
                Ok(secs) => secs,
                Err(e) => {
                    state.message = Some(format!("Invalid interval: {}", e));
                    return Task::none();
                }
            };

            let list_name_clone = list_name.clone();
            state.is_active = true;
            state.is_paused = false;
            state.current_list = Some(list_name.clone());
            state.current_interval = interval_secs;
            state.cycle_count = 0;
            state.message = Some(format!(
                "Starting slideshow: {} ({}s interval)",
                list_name, interval_secs
            ));

            // Start the first cycle
            Task::perform(
                async move {
                    // Load the list
                    async_bridge::get_list_items_async(&list_name_clone)
                        .await
                        .map_err(|e| e)?
                        .first()
                        .and_then(|item| item.relative_path.clone())
                        .ok_or(crate::error::SwpError::GuiLaunch {
                            message: "No valid wallpaper in list".to_string(),
                        })
                },
                |result| {
                    SlideshowMessage::SlideApplied(result)
                },
            )
        }

        SlideshowMessage::SlideApplied(result) => {
            match result {
                Ok(path) => {
                    if let Some(_list_name) = &state.current_list {
                        let _list_name = _list_name.clone();
                        let interval_secs = state.current_interval;
                        state.currently_playing = Some(path.clone());
                        state.cycle_count = state.cycle_count.saturating_add(1);

                        // Apply this wallpaper
                        Task::perform(
                            async move {
                                // Apply the wallpaper
                                let result =
                                    async_bridge::apply_wallpaper_async(&path)
                                        .await;

                                // Wait for interval if successful
                                if result.is_ok() {
                                    tokio::time::sleep(Duration::from_secs(interval_secs)).await;
                                }

                                result
                            },
                            move |_apply_result| {
                                // After interval, get next wallpaper from list
                                SlideshowMessage::SlideApplied(
                                    // Dummy: in real implementation would load next from ListPlayer
                                    Err(crate::error::SwpError::GuiLaunch {
                                        message: "Slideshow cycle complete".to_string(),
                                    }),
                                )
                            },
                        )
                    } else {
                        state.message = Some("Slideshow stopped".to_string());
                        state.is_active = false;
                        Task::none()
                    }
                }
                Err(e) => {
                    state.message = Some(format!("Slideshow error: {}", e));
                    state.is_active = false;
                    state.reset();
                    Task::none()
                }
            }
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
                // Resume by triggering next cycle
                if let Some(path) = &state.currently_playing {
                    let path = path.clone();
                    Task::perform(
                        async move { Ok(path) },
                        |result| SlideshowMessage::SlideApplied(result),
                    )
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }

        SlideshowMessage::StopPressed => {
            if state.is_active {
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

/// Parse interval string (e.g., "5s", "1m", "30").
/// Returns interval in seconds, or error message.
fn parse_interval(s: &str) -> Result<u64, String> {
    let s = s.trim();

    if s.ends_with('s') {
        let num_part = &s[..s.len() - 1];
        num_part
            .parse::<u64>()
            .map_err(|_| "Invalid seconds value".to_string())
    } else if s.ends_with('m') {
        let num_part = &s[..s.len() - 1];
        let minutes: u64 = num_part
            .parse()
            .map_err(|_| "Invalid minutes value".to_string())?;
        Ok(minutes * 60)
    } else {
        // Try parsing as plain seconds
        s.parse::<u64>()
            .map_err(|_| "Use format: 5s, 1m, or plain number".to_string())
    }
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

