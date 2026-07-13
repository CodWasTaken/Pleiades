use std::collections::HashMap;
use std::path::Path;

use pleiades_agent_core::conversation::Message;
use pleiades_agent_core::error::Error;
use pleiades_agent_core::provider::{ChatRequest, Provider};
use pleiades_agent_prompts::PromptTemplate;
use tokio::process::Command;

pub(crate) async fn git_diff(repository: &Path, arguments: &[&str]) -> Result<String, Error> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repository)
        .output()
        .await?;
    if !output.status.success() {
        return Err(Error::tool(format!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let diff = String::from_utf8_lossy(&output.stdout).into_owned();
    if diff.trim().is_empty() {
        return Err(Error::invalid_input("git diff is empty"));
    }
    Ok(diff)
}

pub(crate) async fn git_output(repository: &Path, arguments: &[&str]) -> Result<String, Error> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repository)
        .output()
        .await?;
    if !output.status.success() {
        return Err(Error::tool(String::from_utf8_lossy(&output.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(crate) async fn generate_from_diff(
    provider: &dyn Provider,
    model: &str,
    template: PromptTemplate,
    diff: &str,
    title: Option<&str>,
) -> Result<String, Error> {
    let mut variables = HashMap::from([("diff".to_string(), diff.to_string())]);
    if let Some(title) = title {
        variables.insert("title".to_string(), title.to_string());
    }
    let prompt = template
        .render(&variables)
        .map_err(|error| Error::invalid_input(error.to_string()))?;
    let response = provider
        .chat(ChatRequest {
            model: model.to_string(),
            messages: vec![Message::user(prompt)],
            system_prompt: None,
            temperature: Some(0.2),
            top_p: None,
            max_tokens: Some(2048),
            stop: None,
            tools: None,
        })
        .await?;
    let text = response.message.text_content().trim().to_string();
    if text.is_empty() {
        return Err(Error::provider("provider returned an empty response"));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use pleiades_agent_core::conversation::Message;
    use pleiades_agent_core::model::ModelInfo;
    use pleiades_agent_core::provider::{
        ChatResponse, EmbeddingResponse, ProviderCapabilities, StreamEvent,
    };

    use super::*;

    struct MockProvider {
        prompt: Mutex<String>,
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }
        fn display_name(&self) -> &str {
            "Mock"
        }
        fn default_model(&self) -> &str {
            "mock-1"
        }
        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                streaming: false,
                tools: false,
                vision: false,
                embeddings: false,
                thinking: false,
                json_mode: false,
                function_calling: false,
            }
        }
        async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
            Ok(Vec::new())
        }
        async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Error> {
            *self.prompt.lock().unwrap() = request.messages[0].text_content();
            Ok(ChatResponse {
                message: Message::assistant("feat: generated message"),
                usage: None,
                finish_reason: Some("stop".into()),
            })
        }
        async fn chat_stream(
            &self,
            _request: ChatRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            Ok(rx)
        }
        async fn embed(
            &self,
            _input: Vec<String>,
            _model: &str,
        ) -> Result<EmbeddingResponse, Error> {
            Err(Error::unsupported("not supported"))
        }
    }

    #[tokio::test]
    async fn renders_diff_and_returns_provider_text() {
        let provider = MockProvider {
            prompt: Mutex::new(String::new()),
        };
        let output = generate_from_diff(
            &provider,
            "mock-1",
            pleiades_agent_prompts::BuiltinPrompts::commit_message(),
            "diff --git a/a b/a",
            None,
        )
        .await
        .unwrap();
        assert_eq!(output, "feat: generated message");
        assert!(
            provider
                .prompt
                .lock()
                .unwrap()
                .contains("diff --git a/a b/a")
        );
    }

    #[tokio::test]
    async fn reads_staged_diff() {
        let directory = tempfile::tempdir().unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            let status = Command::new("git")
                .args(args)
                .current_dir(directory.path())
                .status()
                .await
                .unwrap();
            assert!(status.success());
        }
        std::fs::write(directory.path().join("file.txt"), "content\n").unwrap();
        let status = Command::new("git")
            .args(["add", "file.txt"])
            .current_dir(directory.path())
            .status()
            .await
            .unwrap();
        assert!(status.success());
        let diff = crate::staged_diff(directory.path()).await.unwrap();
        assert!(diff.contains("+content"));
    }
}
