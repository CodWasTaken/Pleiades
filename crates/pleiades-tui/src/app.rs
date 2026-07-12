use pleiades_core::error::Error;

/// Terminal UI application state and management.
pub struct TuiApp {
    pub running: bool,
}

impl TuiApp {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            running: false,
        })
    }

    /// Run the TUI event loop.
    pub async fn run(&mut self) -> Result<(), Error> {
        self.running = true;
        // TUI implementation will be added in Milestone 8
        Err(Error::NotImplemented("TUI not yet implemented".to_string()))
    }

    /// Stop the TUI.
    pub fn stop(&mut self) {
        self.running = false;
    }
}
