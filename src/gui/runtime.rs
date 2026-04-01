//! Tokio runtime management for the GUI.
//!
//! Provides a wrapper around the tokio runtime to manage its lifecycle
//! and provide a simple interface for spawning async tasks.

use std::sync::Arc;
use tokio::runtime::Runtime;

/// Wrapper around tokio runtime for managing async task execution.
pub struct GuiRuntime {
    runtime: Arc<Runtime>,
}

impl GuiRuntime {
    /// Create a new GUI runtime with a single-threaded executor.
    ///
    /// Single-threaded is sufficient for GUI applications since the UI itself
    /// is single-threaded, and we use spawn_blocking for CPU-intensive tasks.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(GuiRuntime {
            runtime: Arc::new(runtime),
        })
    }

    /// Get a reference to the shared runtime.
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }

    /// Block on a future until completion (use sparingly - can block UI).
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.runtime.block_on(future)
    }
}

impl Default for GuiRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create tokio runtime")
    }
}

impl Clone for GuiRuntime {
    fn clone(&self) -> Self {
        GuiRuntime {
            runtime: Arc::clone(&self.runtime),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let rt = GuiRuntime::new().expect("Failed to create runtime");
        let result = rt.block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_spawn_simple_task() {
        let rt = GuiRuntime::new().expect("Failed to create runtime");
        let handle = rt.handle();

        handle.spawn(async {
            assert_eq!(1 + 1, 2);
        });

        // Give time for task to complete
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
