//! Handler modules for feature-specific update logic.
//!
//! Each handler module manages messages for a specific feature domain:
//! - wallpaper: Loading, applying, and thumbnail generation
//! - lists: Collection management (Fase 3)
//! - slideshow: Automatic cycling (Fase 4)

pub mod wallpaper;
pub mod lists;
pub mod slideshow;
