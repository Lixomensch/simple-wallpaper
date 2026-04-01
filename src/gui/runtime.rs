use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct GuiRuntime {
    runtime: Arc<Runtime>,
}

impl GuiRuntime {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        Ok(GuiRuntime {
            runtime: Arc::new(runtime),
        })
    }

    pub fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }

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

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
