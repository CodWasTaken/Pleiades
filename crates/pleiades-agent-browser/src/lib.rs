//! Optional Playwright browser verification.
//!
//! Pleiades does not bundle a browser or npm packages. This crate shells out to
//! `node` and expects the workspace or environment to provide the `playwright`
//! npm package and installed browsers. Missing prerequisites are surfaced as
//! actionable errors instead of fake success.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pleiades_agent_core::Error;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserReport {
    pub url: String,
    pub title: String,
    pub status: Option<u16>,
    pub console: Vec<String>,
    pub failed_requests: Vec<String>,
    pub html_chars: usize,
    pub screenshot_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct BrowserService {
    workspace: PathBuf,
    last_url: Arc<Mutex<Option<String>>>,
    last_report: Arc<Mutex<Option<BrowserReport>>>,
}

impl BrowserService {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            workspace,
            last_url: Arc::new(Mutex::new(None)),
            last_report: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn open(&self, url: &str) -> Result<BrowserReport, Error> {
        validate_url(url)?;
        let report = self.run_playwright(url, None).await?;
        *self.last_url.lock().await = Some(report.url.clone());
        *self.last_report.lock().await = Some(report.clone());
        Ok(report)
    }

    pub async fn screenshot(&self) -> Result<BrowserReport, Error> {
        let url =
            self.last_url.lock().await.clone().ok_or_else(|| {
                Error::invalid_input("usage: /browser open <url> before screenshot")
            })?;
        let directory = self.workspace.join(".pleiades/browser/screenshots");
        std::fs::create_dir_all(&directory).map_err(Error::from)?;
        let path = directory.join(format!("screenshot-{}.png", now_ms()));
        let report = self.run_playwright(&url, Some(path)).await?;
        *self.last_report.lock().await = Some(report.clone());
        Ok(report)
    }

    pub async fn inspect(&self) -> Result<BrowserReport, Error> {
        self.last_report
            .lock()
            .await
            .clone()
            .ok_or_else(|| Error::invalid_input("usage: /browser open <url> before inspect"))
    }

    pub async fn console(&self) -> Result<Vec<String>, Error> {
        Ok(self.inspect().await?.console)
    }

    pub async fn close(&self) {
        *self.last_url.lock().await = None;
        *self.last_report.lock().await = None;
    }

    async fn run_playwright(
        &self,
        url: &str,
        screenshot_path: Option<PathBuf>,
    ) -> Result<BrowserReport, Error> {
        let script = playwright_script();
        let mut command = Command::new("node");
        command
            .arg("-e")
            .arg(script)
            .current_dir(&self.workspace)
            .env("PLEIADES_BROWSER_URL", url);
        if let Some(path) = &screenshot_path {
            command.env("PLEIADES_SCREENSHOT_PATH", path);
        }
        let output = command.output().await.map_err(|error| {
            Error::tool(format!(
                "failed to start node for Playwright verification: {error}. Install Node.js and the Playwright npm package."
            ))
        })?;
        if !output.status.success() {
            return Err(Error::tool(format!(
                "Playwright verification failed: {}\nInstall with `npm install -D playwright` and `npx playwright install chromium` in the workspace or make Playwright resolvable to node.",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_report(stdout.trim(), screenshot_path)
    }
}

fn validate_url(url: &str) -> Result<(), Error> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(Error::invalid_input(
            "browser URLs must start with http:// or https://",
        ))
    }
}

fn parse_report(raw: &str, screenshot_path: Option<PathBuf>) -> Result<BrowserReport, Error> {
    let mut report: BrowserReport = serde_json::from_str(raw).map_err(Error::from)?;
    if report.screenshot_path.is_none() {
        report.screenshot_path = screenshot_path;
    }
    Ok(report)
}

fn playwright_script() -> &'static str {
    r#"
const { chromium } = require('playwright');
(async () => {
  const url = process.env.PLEIADES_BROWSER_URL;
  const screenshotPath = process.env.PLEIADES_SCREENSHOT_PATH || null;
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  const consoleMessages = [];
  const failedRequests = [];
  page.on('console', msg => consoleMessages.push(`${msg.type()}: ${msg.text()}`));
  page.on('requestfailed', req => failedRequests.push(`${req.method()} ${req.url()} ${req.failure()?.errorText || ''}`));
  const response = await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
  const title = await page.title();
  const html = await page.content();
  if (screenshotPath) {
    await page.screenshot({ path: screenshotPath, fullPage: true });
  }
  const result = {
    url: page.url(),
    title,
    status: response ? response.status() : null,
    console: consoleMessages,
    failed_requests: failedRequests,
    html_chars: html.length,
    screenshot_path: screenshotPath
  };
  await browser.close();
  console.log(JSON.stringify(result));
})().catch(error => {
  console.error(error && error.stack ? error.stack : String(error));
  process.exit(1);
});
"#
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http_urls() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("https://example.com").is_ok());
    }

    #[test]
    fn parses_playwright_json_report() {
        let report = parse_report(
            r#"{"url":"https://example.com/","title":"Example","status":200,"console":["error: boom"],"failed_requests":[],"html_chars":42,"screenshot_path":null}"#,
            Some(PathBuf::from("shot.png")),
        )
        .unwrap();
        assert_eq!(report.status, Some(200));
        assert_eq!(report.screenshot_path, Some(PathBuf::from("shot.png")));
    }
}
