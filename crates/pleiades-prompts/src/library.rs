use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::builtin::BuiltinPrompts;
use crate::error::PromptError;
use crate::template::PromptTemplate;

/// A persisted, user-defined prompt stored as a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPrompt {
    pub name: String,
    pub description: String,
    pub template: String,
}

/// Library of prompt templates: built-in plus user-registered prompts.
#[derive(Debug, Clone, Default)]
pub struct PromptLibrary {
    builtins: HashMap<String, PromptTemplate>,
    custom: HashMap<String, PromptTemplate>,
}

impl PromptLibrary {
    /// Build a library with all built-in prompts registered.
    pub fn with_builtins() -> Self {
        let mut builtins = HashMap::new();
        for tpl in BuiltinPrompts::all() {
            builtins.insert(tpl.name().to_string(), tpl);
        }
        Self {
            builtins,
            custom: HashMap::new(),
        }
    }

    /// Register a custom template.
    pub fn register(&mut self, name: &str, description: &str, raw: &str) {
        let tpl = PromptTemplate::new(name, description, raw);
        self.custom.insert(name.to_string(), tpl);
    }

    /// Get a template by name (custom overrides builtin).
    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.custom
            .get(name)
            .or_else(|| self.builtins.get(name))
    }

    /// Render a template by name with variables.
    pub fn render(
        &self,
        name: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let tpl = self
            .get(name)
            .ok_or_else(|| PromptError::NotFound(name.to_string()))?;
        tpl.render(vars)
    }

    /// Remove a custom template.
    pub fn remove(&mut self, name: &str) -> bool {
        self.custom.remove(name).is_some()
    }

    /// List all template names with a marker for which are built-in.
    pub fn list(&self) -> Vec<PromptSummary> {
        let mut out = Vec::new();
        for tpl in self.builtins.values() {
            out.push(PromptSummary {
                name: tpl.name().to_string(),
                description: tpl.description().to_string(),
                source: "builtin".to_string(),
                variables: tpl.variable_names(),
            });
        }
        for tpl in self.custom.values() {
            out.push(PromptSummary {
                name: tpl.name().to_string(),
                description: tpl.description().to_string(),
                source: "custom".to_string(),
                variables: tpl.variable_names(),
            });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    /// Directory where custom prompts are persisted.
    pub fn store_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("pleiades")
            .join("prompts")
    }

    /// Persist a custom prompt to disk.
    pub fn save_custom(&self, prompt: &StoredPrompt) -> Result<(), PromptError> {
        let dir = Self::store_dir();
        std::fs::create_dir_all(&dir).map_err(|e| PromptError::Other(e.to_string()))?;
        let path = dir.join(format!("{}.json", prompt.name));
        let data = serde_json::to_string_pretty(prompt).map_err(|e| PromptError::Other(e.to_string()))?;
        std::fs::write(&path, data).map_err(|e| PromptError::Other(e.to_string()))?;
        Ok(())
    }

    /// Load all custom prompts from disk into this library.
    pub fn load_custom(&mut self) -> Result<(), PromptError> {
        let dir = Self::store_dir();
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&dir).map_err(|e| PromptError::Other(e.to_string()))? {
            let entry = entry.map_err(|e| PromptError::Other(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content =
                std::fs::read_to_string(&path).map_err(|e| PromptError::Other(e.to_string()))?;
            let stored: StoredPrompt =
                serde_json::from_str(&content).map_err(|e| PromptError::Other(e.to_string()))?;
            let tpl = PromptTemplate::new(&stored.name, &stored.description, &stored.template);
            self.custom.insert(stored.name.clone(), tpl);
        }
        Ok(())
    }
}

/// Summary info for a prompt template.
#[derive(Debug, Clone)]
pub struct PromptSummary {
    pub name: String,
    pub description: String,
    pub source: String,
    pub variables: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_renders_custom_prompt() {
        let mut lib = PromptLibrary::with_builtins();
        lib.register("welcome", "welcome msg", "Hi {{user}}");
        let mut vars = HashMap::new();
        vars.insert("user".to_string(), "Sam".to_string());
        assert_eq!(lib.render("welcome", &vars).unwrap(), "Hi Sam");
    }

    #[test]
    fn builtin_prompts_available() {
        let lib = PromptLibrary::with_builtins();
        assert!(lib.get("default-assistant").is_some());
        assert!(lib.get("summarizer").is_some());
        assert!(lib.get("code-reviewer").is_some());
    }

    #[test]
    fn list_includes_builtins_and_custom() {
        let mut lib = PromptLibrary::with_builtins();
        lib.register("x", "x", "{{y}}");
        let summaries = lib.list();
        assert!(summaries.iter().any(|s| s.name == "default-assistant"));
        assert!(summaries.iter().any(|s| s.name == "x"));
    }

    #[test]
    fn custom_overrides_builtin() {
        let mut lib = PromptLibrary::with_builtins();
        lib.register("default-assistant", "override", "Overridden {{os}}");
        let mut vars = HashMap::new();
        vars.insert("os".to_string(), "macos".to_string());
        assert_eq!(lib.render("default-assistant", &vars).unwrap(), "Overridden macos");
    }
}
