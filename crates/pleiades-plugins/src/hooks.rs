use pleiades_core::error::Error;

/// Hook execution context.
pub struct HookContext {
    pub plugin_name: String,
    pub event: String,
    pub data: serde_json::Value,
}

/// Result of a hook execution.
pub struct HookResult {
    pub modified_data: Option<serde_json::Value>,
    pub abort: bool,
}

type HookHandler = Box<dyn Fn(&HookContext) -> Result<HookResult, Error> + Send + Sync>;

/// Hook registration for plugin extension points.
pub struct HookRegistry {
    hooks: Vec<(String, String, HookHandler)>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
        }
    }

    pub fn register(
        &mut self,
        plugin_name: impl Into<String>,
        hook_name: impl Into<String>,
        handler: HookHandler,
    ) {
        self.hooks.push((plugin_name.into(), hook_name.into(), handler));
    }

    pub fn trigger(&self, hook_name: &str, ctx: &HookContext) -> Vec<Result<HookResult, Error>> {
        self.hooks
            .iter()
            .filter(|(_, name, _)| name == hook_name)
            .map(|(_, _, handler)| handler(ctx))
            .collect()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
