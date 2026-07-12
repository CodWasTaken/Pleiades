use std::sync::Arc;

use pleiades_core::error::Error;
use pleiades_core::event::Event;
use pleiades_core::model::ModelRegistry;

use pleiades_config::types::Config;
use pleiades_config::ConfigLoader;

use pleiades_engine::Engine;

/// Main application orchestrating all Pleiades subsystems.
pub struct App {
    pub config: Config,
    pub engine: Engine,
    pub model_registry: ModelRegistry,
    pub running: bool,
    event_receiver: Option<tokio::sync::mpsc::Receiver<Event>>,
}

impl App {
    /// Initialize a new application with default configuration.
    pub fn new() -> Result<Self, Error> {
        let loader = ConfigLoader::new();
        let config = loader.load().map_err(|e| Error::config(e))?;

        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(256);

        let mut engine = Engine::new(config.clone());
        engine.set_event_sender(event_sender);

        Ok(Self {
            config,
            engine,
            model_registry: ModelRegistry::new(),
            running: false,
            event_receiver: Some(event_receiver),
        })
    }

    /// Run the application event loop.
    pub async fn run(&mut self) -> Result<(), Error> {
        self.running = true;
        // Application event loop will be implemented in future milestones
        Ok(())
    }

    /// Stop the application.
    pub fn stop(&mut self) {
        self.running = false;
    }
}
