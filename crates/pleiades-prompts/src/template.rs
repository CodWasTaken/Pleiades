use std::collections::HashMap;

use serde_json::Value;

use crate::error::PromptError;

/// A compiled prompt template that supports `{{variable}}` substitution
/// and optional defaults via `{{variable|default value}}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptTemplate {
    name: String,
    description: String,
    raw: String,
    variables: Vec<Variable>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Variable {
    name: String,
    default: Option<String>,
}

impl PromptTemplate {
    /// Create a new template from raw text.
    pub fn new(name: impl Into<String>, description: impl Into<String>, raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let variables = extract_variables(&raw);
        Self {
            name: name.into(),
            description: description.into(),
            raw,
            variables,
        }
    }

    /// The template name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The template description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The raw (unrendered) template text.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Names of variables referenced by this template.
    pub fn variable_names(&self) -> Vec<String> {
        self.variables.iter().map(|v| v.name.clone()).collect()
    }

    /// Render the template using a map of variables.
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String, PromptError> {
        let mut out = String::with_capacity(self.raw.len());
        let mut rest = self.raw.as_str();

        while let Some(open) = rest.find("{{") {
            out.push_str(&rest[..open]);
            let after_open = &rest[open + 2..];
            let close = after_open
                .find("}}")
                .ok_or_else(|| PromptError::Unterminated { template: self.name.clone() })?;
            let token = after_open[..close].trim();
            let (name, default) = match token.split_once('|') {
                Some((name, default)) => (name.trim(), Some(default.trim().to_string())),
                None => (token.trim(), None),
            };

            let value = vars
                .get(name)
                .cloned()
                .or_else(|| default.clone())
                .ok_or_else(|| PromptError::MissingVariable {
                    name: name.to_string(),
                    template: self.name.clone(),
                })?;
            out.push_str(&value);
            rest = &after_open[close + 2..];
        }

        out.push_str(rest);
        Ok(out)
    }

    /// Render the template from a JSON object.
    pub fn render_json(&self, vars: &Value) -> Result<String, PromptError> {
        let mut map = HashMap::new();
        if let Some(obj) = vars.as_object() {
            for (k, v) in obj {
                let rendered = match v {
                    Value::String(s) => s.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                };
                map.insert(k.clone(), rendered);
            }
        }
        self.render(&map)
    }
}

fn extract_variables(raw: &str) -> Vec<Variable> {
    let mut vars = Vec::new();
    let mut rest = raw;
    while let Some(open) = rest.find("{{") {
        if let Some(after_open) = rest.get(open + 2..) {
            if let Some(close) = after_open.find("}}") {
                let token = after_open[..close].trim();
                let (name, default) = match token.split_once('|') {
                    Some((name, default)) => (name.trim(), Some(default.trim().to_string())),
                    None => (token.trim(), None),
                };
                if !name.is_empty() {
                    vars.push(Variable {
                        name: name.to_string(),
                        default,
                    });
                }
                rest = &after_open[close + 2..];
                continue;
            }
        }
        break;
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_simple_variable() {
        let tpl = PromptTemplate::new("greet", "greeting", "Hello {{name}}!");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Ada".to_string());
        assert_eq!(tpl.render(&vars).unwrap(), "Hello Ada!");
    }

    #[test]
    fn uses_default_when_missing() {
        let tpl = PromptTemplate::new("greet", "greeting", "Hello {{name|World}}!");
        let vars = HashMap::new();
        assert_eq!(tpl.render(&vars).unwrap(), "Hello World!");
    }

    #[test]
    fn errors_when_variable_missing_and_no_default() {
        let tpl = PromptTemplate::new("greet", "greeting", "Hello {{name}}!");
        let vars = HashMap::new();
        assert!(tpl.render(&vars).is_err());
    }

    #[test]
    fn renders_multiple_variables() {
        let tpl = PromptTemplate::new(
            "chat",
            "chat system",
            "You are {{role}} named {{name|Assistant}}.",
        );
        let mut vars = HashMap::new();
        vars.insert("role".to_string(), "helper".to_string());
        assert_eq!(tpl.render(&vars).unwrap(), "You are helper named Assistant.");
    }

    #[test]
    fn leaves_literal_braces_alone_when_not_double() {
        let tpl = PromptTemplate::new("code", "code", "fn main() { let x = 1; }");
        let vars = HashMap::new();
        assert_eq!(tpl.render(&vars).unwrap(), "fn main() { let x = 1; }");
    }

    #[test]
    fn renders_from_json() {
        let tpl = PromptTemplate::new("g", "g", "Hi {{name}}");
        let json = serde_json::json!({ "name": "Boo" });
        assert_eq!(tpl.render_json(&json).unwrap(), "Hi Boo");
    }
}
