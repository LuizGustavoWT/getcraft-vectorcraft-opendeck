//! OpenDeck action for the VectorCraft command line interface.
//!
//! Arguments are stored as a JSON string array and sent directly to the CLI.
//! Deliberately never invoking a shell preserves paths, JSON parameters, and user input exactly.

use std::{path::PathBuf, process::Stdio, time::Duration};

use openaction::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use tokio::process::Command;

const DEFAULT_EXECUTABLE: &str = "vectorcraft-cli";
const CATALOG_ARGUMENTS: &[&str] = &["commands"];
const DEFAULT_ARGUMENTS: &str = r#"["commands"]"#;
const MAX_ARGUMENTS: usize = 128;
const MAX_ARGUMENT_BYTES: usize = 16 * 1024;
const MAX_CATALOG_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct CommandSettings {
    /// CLI executable or absolute path. Empty uses the app's conventional CLI name.
    executable: String,
    /// JSON string array, e.g. ["exec", "timeline.play", "{}"].
    arguments: String,
    /// Optional directory in which to run the command.
    working_directory: String,
    /// A user-visible title shown before the optional command output.
    title: String,
    /// Keep stdout/stderr (trimmed) on the Stream Deck key after a successful run.
    show_output: bool,
    /// Wall-clock limit for a process. Zero uses 30 seconds; maximum is 300 seconds.
    timeout_seconds: u64,
}

impl Default for CommandSettings {
    fn default() -> Self {
        Self {
            executable: String::new(),
            arguments: DEFAULT_ARGUMENTS.to_string(),
            working_directory: String::new(),
            title: String::new(),
            show_output: false,
            timeout_seconds: 30,
        }
    }
}

impl CommandSettings {
    fn executable(&self) -> &str {
        let executable = self.executable.trim();
        if executable.is_empty() {
            DEFAULT_EXECUTABLE
        } else {
            executable
        }
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(if self.timeout_seconds == 0 {
            30
        } else {
            self.timeout_seconds.clamp(1, 300)
        })
    }

    fn title(&self) -> Option<String> {
        (!self.title.trim().is_empty()).then(|| self.title.trim().to_string())
    }
}

#[derive(Debug)]
enum ExecutionError {
    Configuration(String),
    Process(String),
    TimedOut(u64),
    Failed { code: Option<i32>, output: String },
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration(message) | Self::Process(message) => f.write_str(message),
            Self::TimedOut(seconds) => write!(f, "Timed out after {seconds} seconds"),
            Self::Failed { code, output } => {
                write!(
                    f,
                    "VectorCraft CLI exited with {}",
                    code.map_or_else(|| "a signal".into(), |code| format!("code {code}"))
                )?;
                if !output.is_empty() {
                    write!(f, ": {output}")?;
                }
                Ok(())
            }
        }
    }
}

fn parse_arguments(raw: &str) -> Result<Vec<String>, ExecutionError> {
    let parsed: Value = serde_json::from_str(raw.trim()).map_err(|error| {
        ExecutionError::Configuration(format!(
            "Arguments must be a JSON array of strings: {error}"
        ))
    })?;
    let values = parsed.as_array().ok_or_else(|| {
        ExecutionError::Configuration("Arguments must be a JSON array of strings.".into())
    })?;
    if values.is_empty() {
        return Err(ExecutionError::Configuration(
            "Add at least one CLI argument, for example [\"commands\", \"--json\"].".into(),
        ));
    }
    if values.len() > MAX_ARGUMENTS {
        return Err(ExecutionError::Configuration(format!(
            "At most {MAX_ARGUMENTS} arguments are allowed."
        )));
    }
    values
        .iter()
        .map(|value| {
            let value = value.as_str().ok_or_else(|| {
                ExecutionError::Configuration(
                    "Every argument must be a string; numbers and JSON values must be quoted."
                        .into(),
                )
            })?;
            if value.len() > MAX_ARGUMENT_BYTES {
                return Err(ExecutionError::Configuration(format!(
                    "An argument is longer than {MAX_ARGUMENT_BYTES} bytes."
                )));
            }
            Ok(value.to_string())
        })
        .collect()
}

fn summary(stdout: &[u8], stderr: &[u8]) -> String {
    let output = if !stderr.is_empty() { stderr } else { stdout };
    let text = String::from_utf8_lossy(output).replace(['\n', '\r'], " ");
    let text = text.trim();
    let mut end = text.len().min(120);
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    let suffix = if text.len() > end { "…" } else { "" };
    format!("{}{}", &text[..end], suffix)
}

async fn execute(
    settings: &CommandSettings,
    arguments: Vec<String>,
) -> Result<String, ExecutionError> {
    let mut command = Command::new(settings.executable());
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if !settings.working_directory.trim().is_empty() {
        command.current_dir(PathBuf::from(settings.working_directory.trim()));
    }
    let seconds = settings.timeout().as_secs();
    let output = tokio::time::timeout(settings.timeout(), command.output())
        .await
        .map_err(|_| ExecutionError::TimedOut(seconds))?
        .map_err(|error| {
            ExecutionError::Process(format!(
                "Could not start {}: {error}",
                settings.executable()
            ))
        })?;
    let output_summary = summary(&output.stdout, &output.stderr);
    if output.status.success() {
        Ok(output_summary)
    } else {
        Err(ExecutionError::Failed {
            code: output.status.code(),
            output: output_summary,
        })
    }
}

async fn report(
    instance: &Instance,
    settings: &CommandSettings,
    result: Result<String, ExecutionError>,
    to_inspector: bool,
) {
    match result {
        Ok(output) => {
            let title = if settings.show_output && !output.is_empty() {
                output.clone()
            } else {
                settings.title().unwrap_or_else(|| "Done".to_string())
            };
            let _ = instance.set_title(Some(title), None).await;
            let _ = instance.show_ok().await;
            if to_inspector {
                let _ = instance
                    .send_to_property_inspector(
                        json!({"kind":"result", "ok":true, "message": output}),
                    )
                    .await;
            }
        }
        Err(error) => {
            let message = error.to_string();
            log::warn!("VectorCraft CLI action failed: {message}");
            let _ = instance.set_title(Some("Error".to_string()), None).await;
            let _ = instance.show_alert().await;
            if to_inspector {
                let _ = instance
                    .send_to_property_inspector(
                        json!({"kind":"result", "ok":false, "message": message}),
                    )
                    .await;
            }
        }
    }
}

async fn run_action(instance: &Instance, settings: &CommandSettings, to_inspector: bool) {
    let result = parse_arguments(&settings.arguments).and_then(|arguments| {
        // Validation above is synchronous; the process work stays on the async runtime.
        Ok(arguments)
    });
    let result = match result {
        Ok(arguments) => execute(settings, arguments).await,
        Err(error) => Err(error),
    };
    report(instance, settings, result, to_inspector).await;
}

fn catalog_items(stdout: &str) -> Vec<Value> {
    let Ok(value) = serde_json::from_str::<Value>(stdout) else {
        return Vec::new();
    };
    let items = value
        .as_array()
        .cloned()
        .or_else(|| value.get("commands").and_then(Value::as_array).cloned())
        .unwrap_or_default();
    items
        .into_iter()
        .filter_map(|item| {
            if let Some(id) = item.as_str() {
                return Some(json!({"id": id, "label": id}));
            }
            let id = ["id", "command", "tool", "name"]
                .into_iter()
                .find_map(|key| item.get(key).and_then(Value::as_str))?;
            let label = item
                .get("label")
                .or_else(|| item.get("name"))
                .and_then(Value::as_str)
                .unwrap_or(id);
            Some(json!({"id": id, "label": label}))
        })
        .collect()
}

async fn send_catalog(instance: &Instance, settings: &CommandSettings) {
    let mut command = Command::new(settings.executable());
    command
        .args(CATALOG_ARGUMENTS)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if !settings.working_directory.trim().is_empty() {
        command.current_dir(PathBuf::from(settings.working_directory.trim()));
    }
    let result = tokio::time::timeout(Duration::from_secs(15), command.output()).await;
    let reply = match result {
        Ok(Ok(output)) if output.status.success() && output.stdout.len() <= MAX_CATALOG_BYTES => {
            let text = String::from_utf8_lossy(&output.stdout);
            let items = catalog_items(&text);
            if items.is_empty() {
                json!({"kind":"catalog", "ok":false, "message":"The installed CLI did not return a JSON command catalog."})
            } else {
                json!({"kind":"catalog", "ok":true, "items":items})
            }
        }
        Ok(Ok(output)) => {
            json!({"kind":"catalog", "ok":false, "message":format!("Could not read the command catalog: {}", summary(&output.stdout, &output.stderr))})
        }
        Ok(Err(error)) => {
            json!({"kind":"catalog", "ok":false, "message":format!("Could not start {}: {error}", settings.executable())})
        }
        Err(_) => {
            json!({"kind":"catalog", "ok":false, "message":"Timed out while loading the command catalog."})
        }
    };
    let _ = instance.send_to_property_inspector(reply).await;
}

struct RunVectorCraftCli;

#[async_trait]
impl Action for RunVectorCraftCli {
    const UUID: &'static str = "wt.luizgustavo.getcraft.vectorcraft.run";
    type Settings = CommandSettings;

    async fn key_down(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        run_action(instance, settings, false).await;
        Ok(())
    }

    async fn dial_down(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        self.key_down(instance, settings).await
    }

    async fn will_appear(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        if let Some(title) = settings.title() {
            let _ = instance.set_title(Some(title), None).await;
        }
        Ok(())
    }

    async fn send_to_plugin(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
        payload: &Value,
    ) -> OpenActionResult<()> {
        let effective = payload
            .get("settings")
            .and_then(|value| serde_json::from_value::<CommandSettings>(value.clone()).ok())
            .unwrap_or_else(|| settings.clone());
        match payload.get("request").and_then(Value::as_str) {
            Some("catalog") => send_catalog(instance, &effective).await,
            Some("test") => run_action(instance, &effective, true).await,
            _ => {}
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> OpenActionResult<()> {
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stdout,
        ColorChoice::Never,
    );
    register_action(RunVectorCraftCli).await;
    run(std::env::args().collect()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_string_argument_vector() {
        assert_eq!(
            parse_arguments(r#"["exec","timeline.play","{}"]"#).unwrap(),
            ["exec", "timeline.play", "{}"]
        );
    }

    #[test]
    fn rejects_non_string_arguments() {
        assert!(parse_arguments(r#"["exec", 4]"#).is_err());
    }

    #[test]
    fn stderr_is_preferred_in_a_failure_summary() {
        assert_eq!(summary(b"normal", b"bad\nnews"), "bad news");
    }

    #[test]
    fn has_a_usable_default_command() {
        assert_eq!(
            parse_arguments(&CommandSettings::default().arguments).unwrap(),
            CATALOG_ARGUMENTS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn honors_a_one_second_timeout() {
        assert_eq!(
            CommandSettings {
                timeout_seconds: 1,
                ..Default::default()
            }
            .timeout(),
            Duration::from_secs(1)
        );
    }
}
