//! OpenDeck action for the VectorCraft command line interface.
//!
//! Arguments are stored as a JSON string array and sent directly to the CLI.
//! Deliberately never invoking a shell preserves paths, JSON parameters, and user input exactly.

use std::{path::PathBuf, process::Stdio, time::Duration};

use openaction::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    process::Command,
};

const DEFAULT_EXECUTABLE: &str = "vectorcraft-cli";
const DEFAULT_DESKTOP_EXECUTABLE: &str = "vectorcraft";
const CATALOG_ARGUMENTS: &[&str] = &["commands"];
const DEFAULT_ARGUMENTS: &str = r#"["commands"]"#;
const MAX_ARGUMENTS: usize = 128;
const MAX_ARGUMENT_BYTES: usize = 16 * 1024;
const MAX_CATALOG_BYTES: usize = 2 * 1024 * 1024;
const DEFAULT_CONTROL_PORT: u16 = 7979;
const MAX_CONTROL_PARAMS_BYTES: usize = 16 * 1024;

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

/// A direct control-channel action for a running VectorCraft desktop session.
/// Unlike the batch CLI action, this preserves the user's document, selection and undo history.
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct LiveControlSettings {
    control_host: String,
    control_port: u16,
    method: String,
    command: String,
    params: String,
    rotary_method: String,
    rotary_command: String,
    rotary_params: String,
    rotary_step: f64,
    title: String,
}

impl Default for LiveControlSettings {
    fn default() -> Self {
        Self {
            control_host: "127.0.0.1".into(),
            control_port: DEFAULT_CONTROL_PORT,
            method: "engine.execute".into(),
            command: "edit.undo".into(),
            params: "{}".into(),
            rotary_method: "".into(),
            rotary_command: "".into(),
            rotary_params: "{}".into(),
            rotary_step: 1.0,
            title: "Undo".into(),
        }
    }
}

impl LiveControlSettings {
    fn host(&self) -> Result<&str, ExecutionError> {
        let host = self.control_host.trim();
        let host = if host.is_empty() { "127.0.0.1" } else { host };
        if loopback_host(host) {
            Ok(host)
        } else {
            Err(ExecutionError::Configuration("VectorCraft exposes its control channel only on localhost. Use 127.0.0.1, localhost, or ::1.".into()))
        }
    }

    fn port(&self) -> u16 {
        if self.control_port == 0 {
            DEFAULT_CONTROL_PORT
        } else {
            self.control_port
        }
    }

    fn title(&self) -> String {
        let title = self.title.trim();
        if !title.is_empty() {
            title.to_owned()
        } else if !self.command.trim().is_empty() {
            self.command.trim().to_owned()
        } else {
            "VectorCraft".into()
        }
    }
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

fn parse_control_params(raw: &str) -> Result<Value, ExecutionError> {
    if raw.len() > MAX_CONTROL_PARAMS_BYTES {
        return Err(ExecutionError::Configuration(format!(
            "Control parameters may not exceed {MAX_CONTROL_PARAMS_BYTES} bytes."
        )));
    }
    let value: Value = serde_json::from_str(raw.trim()).map_err(|error| {
        ExecutionError::Configuration(format!("Parameters must be a JSON object: {error}"))
    })?;
    if !value.is_object() {
        return Err(ExecutionError::Configuration(
            "Parameters must be a JSON object.".into(),
        ));
    }
    Ok(value)
}

fn substitute_rotary_tokens(value: Value, ticks: i16, step: f64) -> Value {
    match value {
        Value::String(token) if token == "$TICKS" => json!(ticks),
        Value::String(token) if token == "$STEP" => json!(step),
        Value::String(token) if token == "$AMOUNT" => json!(f64::from(ticks) * step),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| substitute_rotary_tokens(item, ticks, step))
                .collect(),
        ),
        Value::Object(items) => Value::Object(
            items
                .into_iter()
                .map(|(key, value)| (key, substitute_rotary_tokens(value, ticks, step)))
                .collect(),
        ),
        value => value,
    }
}

fn loopback_host(host: &str) -> bool {
    matches!(
        host.to_ascii_lowercase().as_str(),
        "127.0.0.1" | "::1" | "localhost"
    )
}

fn control_payload(method: &str, command: &str, params: Value) -> Result<Value, ExecutionError> {
    let method = method.trim();
    if method.is_empty() {
        return Err(ExecutionError::Configuration(
            "Choose a VectorCraft control method.".into(),
        ));
    }
    let params = if method == "engine.execute" {
        let command = command.trim();
        if command.is_empty() {
            return Err(ExecutionError::Configuration(
                "An engine.execute action needs a VectorCraft command ID.".into(),
            ));
        }
        json!({"command": command, "params": params})
    } else {
        params
    };
    Ok(json!({"id": 1, "method": method, "params": params}))
}

async fn send_control_request(
    host: &str,
    port: u16,
    payload: Value,
) -> Result<String, ExecutionError> {
    let operation = async {
        let mut stream = TcpStream::connect((host, port)).await.map_err(|error| {
            ExecutionError::Process(format!("Could not reach VectorCraft at {host}:{port}: {error}. Start it with --control {port}."))
        })?;
        let encoded = serde_json::to_vec(&payload).map_err(|error| {
            ExecutionError::Configuration(format!("Could not encode the control request: {error}"))
        })?;
        stream.write_all(&encoded).await.map_err(|error| {
            ExecutionError::Process(format!("Could not send the control request: {error}"))
        })?;
        stream.write_all(b"\n").await.map_err(|error| {
            ExecutionError::Process(format!("Could not finish the control request: {error}"))
        })?;
        let mut line = String::new();
        BufReader::new(stream)
            .read_line(&mut line)
            .await
            .map_err(|error| {
                ExecutionError::Process(format!("Could not read VectorCraft's reply: {error}"))
            })?;
        let reply: Value = serde_json::from_str(&line).map_err(|error| {
            ExecutionError::Process(format!(
                "VectorCraft returned an invalid control reply: {error}"
            ))
        })?;
        if reply.get("ok").and_then(Value::as_bool) == Some(true) {
            let result = reply
                .get("result")
                .cloned()
                .unwrap_or(Value::Null)
                .to_string();
            return Ok(summary(result.as_bytes(), &[]));
        }
        Err(ExecutionError::Process(
            reply
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("VectorCraft rejected the control request.")
                .to_owned(),
        ))
    };
    tokio::time::timeout(Duration::from_secs(3), operation)
        .await
        .map_err(|_| ExecutionError::TimedOut(3))?
}

async fn run_live_press(settings: &LiveControlSettings) -> Result<String, ExecutionError> {
    let params = parse_control_params(&settings.params)?;
    send_control_request(
        settings.host()?,
        settings.port(),
        control_payload(&settings.method, &settings.command, params)?,
    )
    .await
}

async fn run_live_rotation(
    settings: &LiveControlSettings,
    ticks: i16,
) -> Result<String, ExecutionError> {
    if ticks == 0 || settings.rotary_method.trim().is_empty() {
        return Ok(String::new());
    }
    let params = substitute_rotary_tokens(
        parse_control_params(&settings.rotary_params)?,
        ticks,
        settings.rotary_step,
    );
    send_control_request(
        settings.host()?,
        settings.port(),
        control_payload(&settings.rotary_method, &settings.rotary_command, params)?,
    )
    .await
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

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct LaunchSettings {
    executable: String,
    control_host: String,
    control_port: u16,
    working_directory: String,
    title: String,
}
impl Default for LaunchSettings {
    fn default() -> Self {
        Self {
            executable: String::new(),
            control_host: "127.0.0.1".into(),
            control_port: DEFAULT_CONTROL_PORT,
            working_directory: String::new(),
            title: "Open VectorCraft".into(),
        }
    }
}
impl LaunchSettings {
    fn executable(&self) -> &str {
        let value = self.executable.trim();
        if value.is_empty() {
            DEFAULT_DESKTOP_EXECUTABLE
        } else {
            value
        }
    }
    fn host(&self) -> Result<&str, ExecutionError> {
        let value = self.control_host.trim();
        let host = if value.is_empty() { "127.0.0.1" } else { value };
        if loopback_host(host) {
            Ok(host)
        } else {
            Err(ExecutionError::Configuration("VectorCraft exposes its control channel only on localhost. Use 127.0.0.1, localhost, or ::1.".into()))
        }
    }
    fn port(&self) -> Result<u16, ExecutionError> {
        if self.control_port == 0 {
            Err(ExecutionError::Configuration(
                "Choose a control port from 1 to 65535.".into(),
            ))
        } else {
            Ok(self.control_port)
        }
    }
    fn arguments(&self) -> Result<Vec<String>, ExecutionError> {
        Ok(vec!["--control".into(), self.port()?.to_string()])
    }
}
async fn launch_vectorcraft(settings: &LaunchSettings) -> Result<String, ExecutionError> {
    let host = settings.host()?;
    let port = settings.port()?;
    if matches!(
        tokio::time::timeout(Duration::from_millis(250), TcpStream::connect((host, port))).await,
        Ok(Ok(_))
    ) {
        return Ok(format!("VectorCraft is already ready at {host}:{port}."));
    }
    let mut command = Command::new(settings.executable());
    command
        .args(settings.arguments()?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(false);
    if !settings.working_directory.trim().is_empty() {
        command.current_dir(PathBuf::from(settings.working_directory.trim()));
    }
    command.spawn().map_err(|error| {
        ExecutionError::Process(format!("Could not open {}: {error}", settings.executable()))
    })?;
    Ok(format!("Starting VectorCraft at {host}:{port}."))
}
async fn report_launch(
    instance: &Instance,
    settings: &LaunchSettings,
    result: Result<String, ExecutionError>,
    inspector: bool,
) {
    match result {
        Ok(message) => {
            let _ = instance.set_title(Some(settings.title.clone()), None).await;
            let _ = instance.show_ok().await;
            if inspector {
                let _ = instance
                    .send_to_property_inspector(
                        json!({"kind":"launch","ok":true,"message":message}),
                    )
                    .await;
            }
        }
        Err(error) => {
            let message = error.to_string();
            log::warn!("VectorCraft launcher failed: {message}");
            let _ = instance.set_title(Some("Error".to_string()), None).await;
            let _ = instance.show_alert().await;
            if inspector {
                let _ = instance
                    .send_to_property_inspector(
                        json!({"kind":"launch","ok":false,"message":message}),
                    )
                    .await;
            }
        }
    }
}
struct LaunchVectorCraft;
#[async_trait]
impl Action for LaunchVectorCraft {
    const UUID: &'static str = "wt.luizgustavo.getcraft.vectorcraft.launch";
    type Settings = LaunchSettings;
    async fn key_down(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        report_launch(
            instance,
            settings,
            launch_vectorcraft(settings).await,
            false,
        )
        .await;
        Ok(())
    }
    async fn will_appear(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        let _ = instance.set_title(Some(settings.title.clone()), None).await;
        Ok(())
    }
    async fn send_to_plugin(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
        payload: &Value,
    ) -> OpenActionResult<()> {
        if payload.get("request").and_then(Value::as_str) == Some("launch") {
            let effective = payload
                .get("settings")
                .and_then(|value| serde_json::from_value::<LaunchSettings>(value.clone()).ok())
                .unwrap_or_else(|| settings.clone());
            report_launch(
                instance,
                &effective,
                launch_vectorcraft(&effective).await,
                true,
            )
            .await;
        }
        Ok(())
    }
}

struct RunVectorCraftCli;

struct VectorCraftLiveControl;

async fn report_live(
    instance: &Instance,
    settings: &LiveControlSettings,
    result: Result<String, ExecutionError>,
    to_inspector: bool,
) {
    match result {
        Ok(output) => {
            let _ = instance.set_title(Some(settings.title()), None).await;
            let _ = instance.show_ok().await;
            if to_inspector {
                let _ = instance
                    .send_to_property_inspector(
                        json!({"kind":"liveResult", "ok":true, "message":output}),
                    )
                    .await;
            }
        }
        Err(error) => {
            let message = error.to_string();
            log::warn!("VectorCraft live control failed: {message}");
            let _ = instance.set_title(Some("Offline".to_string()), None).await;
            let _ = instance.show_alert().await;
            if to_inspector {
                let _ = instance
                    .send_to_property_inspector(
                        json!({"kind":"liveResult", "ok":false, "message":message}),
                    )
                    .await;
            }
        }
    }
}

#[async_trait]
impl Action for VectorCraftLiveControl {
    const UUID: &'static str = "wt.luizgustavo.getcraft.vectorcraft.live";
    type Settings = LiveControlSettings;

    async fn key_down(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        report_live(instance, settings, run_live_press(settings).await, false).await;
        Ok(())
    }

    async fn dial_down(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        self.key_down(instance, settings).await
    }

    async fn dial_rotate(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
        ticks: i16,
        _pressed: bool,
    ) -> OpenActionResult<()> {
        report_live(
            instance,
            settings,
            run_live_rotation(settings, ticks).await,
            false,
        )
        .await;
        Ok(())
    }

    async fn will_appear(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
    ) -> OpenActionResult<()> {
        let _ = instance.set_title(Some(settings.title()), None).await;
        Ok(())
    }

    async fn send_to_plugin(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
        payload: &Value,
    ) -> OpenActionResult<()> {
        if payload.get("request").and_then(Value::as_str) == Some("testLive") {
            let effective = payload
                .get("settings")
                .and_then(|value| serde_json::from_value::<LiveControlSettings>(value.clone()).ok())
                .unwrap_or_else(|| settings.clone());
            report_live(instance, &effective, run_live_press(&effective).await, true).await;
        }
        Ok(())
    }
}

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
    register_action(LaunchVectorCraft).await;
    register_action(RunVectorCraftCli).await;
    register_action(VectorCraftLiveControl).await;
    run(std::env::args().collect()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_uses_the_configured_control_port() {
        let settings = LaunchSettings {
            control_port: 24567,
            ..Default::default()
        };
        assert_eq!(settings.arguments().unwrap(), ["--control", "24567"]);
    }

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

    #[test]
    fn rotary_tokens_become_numbers_in_nested_json() {
        let value = substitute_rotary_tokens(
            json!({"dx":"$AMOUNT", "values":["$TICKS", "$STEP"]}),
            -3,
            2.5,
        );
        assert_eq!(value, json!({"dx":-7.5, "values":[-3, 2.5]}));
    }

    #[test]
    fn engine_requests_wrap_the_command_and_params() {
        assert_eq!(
            control_payload("engine.execute", "edit.undo", json!({})).unwrap(),
            json!({"id":1, "method":"engine.execute", "params":{"command":"edit.undo", "params":{}}})
        );
    }

    #[test]
    fn direct_control_requests_keep_their_params() {
        assert_eq!(
            control_payload("ui.tool.select", "", json!({"tool":"pen"})).unwrap(),
            json!({"id":1, "method":"ui.tool.select", "params":{"tool":"pen"}})
        );
    }

    #[tokio::test]
    async fn sends_one_json_line_to_the_local_control_service() {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(socket);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let mut socket = reader.into_inner();
            socket
                .write_all(b"{\"id\":1,\"ok\":true,\"result\":{\"done\":true}}\n")
                .await
                .unwrap();
            serde_json::from_str::<Value>(&line).unwrap()
        });

        let reply = send_control_request(
            "127.0.0.1",
            port,
            json!({"id":1, "method":"ui.inspect", "params":{}}),
        )
        .await
        .unwrap();
        let request = server.await.unwrap();
        assert_eq!(request["method"], "ui.inspect");
        assert_eq!(request["params"], json!({}));
        assert_eq!(reply, "{\"done\":true}");
    }
}
