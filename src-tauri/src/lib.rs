use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{path::BaseDirectory, AppHandle, Emitter, Manager, State};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};

#[derive(Default)]
struct RuntimeManager {
    process: Mutex<RuntimeProcess>,
}

#[derive(Default)]
struct RuntimeProcess {
    child: Option<CommandChild>,
    pid: Option<u32>,
    version: Option<String>,
    endpoint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuntimeStatus {
    pub ready: bool,
    pub running: bool,
    pub node_version: Option<String>,
    pub nvm_version: String,
    pub dsh_version: Option<String>,
    pub nvm_dir: String,
    pub dsh_home: String,
    pub runtime_dir: String,
    pub endpoint: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DshVersionInfo {
    pub version: String,
    pub active: bool,
    pub is_default: bool,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDshRequest {
    pub node_version: Option<String>,
    pub version: Option<String>,
    pub profile: Option<String>,
    pub workspace: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Clone, Serialize)]
struct RuntimeLog {
    stream: String,
    line: String,
    timestamp: u64,
}

#[derive(Clone, Serialize)]
struct RuntimeStateEvent {
    status: String,
    pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<String>,
}

fn runtime_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

#[tauri::command]
fn default_workspace(app: AppHandle) -> Result<String, String> {
    let directory = app
        .path()
        .home_dir()
        .map_err(|error| error.to_string())?
        .join(".dsh-launcher");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.to_string_lossy().into_owned())
}

fn nvm_archive(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resolve("nvm-v0.40.7.tar.gz", BaseDirectory::Resource)
        .map_err(|error| error.to_string())
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn emit_log(app: &AppHandle, stream: &str, line: impl Into<String>) {
    let _ = app.emit(
        "runtime-log",
        RuntimeLog {
            stream: stream.to_owned(),
            line: line.into(),
            timestamp: timestamp(),
        },
    );
}

fn emit_state(
    app: &AppHandle,
    status: &str,
    pid: Option<u32>,
    message: Option<String>,
    endpoint: Option<String>,
) {
    let _ = app.emit(
        "runtime-state",
        RuntimeStateEvent {
            status: status.to_owned(),
            pid,
            message,
            endpoint,
        },
    );
}

fn dsh_endpoint(line: &str) -> Option<String> {
    let value = line.split_once("dsh web: ")?.1.split_whitespace().next()?;
    if !value.starts_with("http://127.0.0.1:") {
        return None;
    }
    let port = value
        .trim_start_matches("http://127.0.0.1:")
        .split(['/', '?', '#'])
        .next()?;
    if port.is_empty() || !port.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    Some(value.to_owned())
}

fn parse_protocol_line(app: &AppHandle, line: &[u8], values: &mut Vec<Value>) {
    let text = String::from_utf8_lossy(line).trim().to_owned();
    if text.is_empty() {
        return;
    }
    match serde_json::from_str::<Value>(&text) {
        Ok(value) if value.get("kind").and_then(Value::as_str) == Some("log") => {
            let stream = value
                .get("stream")
                .and_then(Value::as_str)
                .unwrap_or("stdout");
            let line = value
                .get("line")
                .and_then(Value::as_str)
                .unwrap_or_default();
            emit_log(app, stream, line);
        }
        Ok(value) => values.push(value),
        Err(_) => emit_log(app, "stdout", text),
    }
}

async fn run_sidecar(app: &AppHandle, args: Vec<String>) -> Result<Vec<Value>, String> {
    let command = app
        .shell()
        .sidecar("dsh-runtime")
        .map_err(|error| error.to_string())?
        .args(args);
    let (mut receiver, _child) = command.spawn().map_err(|error| error.to_string())?;
    let mut values = Vec::new();
    let mut exit_code = None;
    while let Some(event) = receiver.recv().await {
        match event {
            CommandEvent::Stdout(line) => parse_protocol_line(app, &line, &mut values),
            CommandEvent::Stderr(line) => emit_log(app, "stderr", String::from_utf8_lossy(&line)),
            CommandEvent::Error(error) => emit_log(app, "stderr", error),
            CommandEvent::Terminated(payload) => exit_code = payload.code,
            _ => {}
        }
    }
    if exit_code.unwrap_or(1) != 0 {
        return Err(format!(
            "dsh-runtime exited with code {}",
            exit_code.unwrap_or(1)
        ));
    }
    Ok(values)
}

fn status_from_values(values: Vec<Value>) -> Result<RuntimeStatus, String> {
    let value = values
        .into_iter()
        .rev()
        .find(|value| value.get("kind").and_then(Value::as_str) == Some("status"))
        .ok_or_else(|| "dsh-runtime did not return status".to_string())?;
    serde_json::from_value(value).map_err(|error| error.to_string())
}

fn versions_from_values(values: Vec<Value>) -> Result<Vec<DshVersionInfo>, String> {
    let value = values
        .into_iter()
        .rev()
        .find(|value| value.get("kind").and_then(Value::as_str) == Some("versions"))
        .ok_or_else(|| "dsh-runtime did not return a version list".to_string())?;
    serde_json::from_value(
        value
            .get("versions")
            .cloned()
            .unwrap_or(Value::Array(Vec::new())),
    )
    .map_err(|error| error.to_string())
}

fn remote_versions_from_values(values: Vec<Value>) -> Result<Vec<String>, String> {
    let value = values
        .into_iter()
        .rev()
        .find(|value| value.get("kind").and_then(Value::as_str) == Some("remote-versions"))
        .ok_or_else(|| "dsh-runtime did not return remote versions".to_string())?;
    serde_json::from_value(
        value
            .get("versions")
            .cloned()
            .unwrap_or(Value::Array(Vec::new())),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
async fn runtime_status(
    app: AppHandle,
    manager: State<'_, RuntimeManager>,
) -> Result<RuntimeStatus, String> {
    let directory = runtime_dir(&app)?;
    let values = run_sidecar(
        &app,
        vec!["inspect".into(), directory.to_string_lossy().into_owned()],
    )
    .await?;
    let mut status = status_from_values(values)?;
    let process = manager
        .process
        .lock()
        .map_err(|_| "runtime process lock poisoned")?;
    status.running = process.child.is_some();
    status.pid = process.pid;
    if process.version.is_some() {
        status.dsh_version.clone_from(&process.version);
    }
    status.endpoint.clone_from(&process.endpoint);
    Ok(status)
}

#[tauri::command]
async fn setup_runtime(
    app: AppHandle,
    node_version: Option<String>,
) -> Result<RuntimeStatus, String> {
    emit_state(&app, "setting-up", None, None, None);
    let directory = runtime_dir(&app)?;
    let archive = nvm_archive(&app)?;
    let values = run_sidecar(
        &app,
        vec![
            "setup".into(),
            directory.to_string_lossy().into_owned(),
            archive.to_string_lossy().into_owned(),
            node_version.unwrap_or_else(|| "24.21.0".into()),
        ],
    )
    .await;
    match values {
        Ok(values) => {
            let status = status_from_values(values)?;
            emit_state(&app, "ready", None, None, None);
            Ok(status)
        }
        Err(error) => {
            emit_state(&app, "error", None, Some(error.clone()), None);
            Err(error)
        }
    }
}

#[tauri::command]
async fn list_dsh_versions(
    app: AppHandle,
    manager: State<'_, RuntimeManager>,
) -> Result<Vec<DshVersionInfo>, String> {
    let directory = runtime_dir(&app)?;
    let values = run_sidecar(
        &app,
        vec!["list".into(), directory.to_string_lossy().into_owned()],
    )
    .await?;
    let mut versions = versions_from_values(values)?;
    let active = manager
        .process
        .lock()
        .map_err(|_| "runtime process lock poisoned")?
        .version
        .clone();
    for item in &mut versions {
        item.active = active.as_deref() == Some(item.version.as_str());
    }
    Ok(versions)
}

#[tauri::command]
async fn list_remote_dsh_versions(app: AppHandle) -> Result<Vec<String>, String> {
    let directory = runtime_dir(&app)?;
    let values = run_sidecar(
        &app,
        vec!["remote".into(), directory.to_string_lossy().into_owned()],
    )
    .await?;
    remote_versions_from_values(values)
}

async fn mutate_version(
    app: &AppHandle,
    action: &str,
    version: String,
) -> Result<Vec<DshVersionInfo>, String> {
    let directory = runtime_dir(app)?;
    let values = run_sidecar(
        app,
        vec![
            action.into(),
            directory.to_string_lossy().into_owned(),
            version,
        ],
    )
    .await?;
    versions_from_values(values)
}

#[tauri::command]
async fn install_dsh_version(
    app: AppHandle,
    version: String,
) -> Result<Vec<DshVersionInfo>, String> {
    mutate_version(&app, "install", version).await
}

#[tauri::command]
async fn set_default_dsh_version(
    app: AppHandle,
    version: String,
) -> Result<Vec<DshVersionInfo>, String> {
    mutate_version(&app, "set-default", version).await
}

#[tauri::command]
async fn remove_dsh_version(
    app: AppHandle,
    manager: State<'_, RuntimeManager>,
    version: String,
) -> Result<Vec<DshVersionInfo>, String> {
    {
        let process = manager
            .process
            .lock()
            .map_err(|_| "runtime process lock poisoned")?;
        if process.child.is_some()
            && process
                .version
                .as_deref()
                .is_none_or(|active| active == version)
        {
            return Err(format!(
                "cannot remove DSH {version} while it is running or starting"
            ));
        }
    }
    mutate_version(&app, "remove", version).await
}

#[tauri::command]
async fn start_dsh(
    app: AppHandle,
    manager: State<'_, RuntimeManager>,
    request: StartDshRequest,
) -> Result<(), String> {
    {
        let process = manager
            .process
            .lock()
            .map_err(|_| "runtime process lock poisoned")?;
        if process.child.is_some() {
            return Err("DSH is already running".into());
        }
    }
    let directory = runtime_dir(&app)?;
    let mut args = vec!["start".into(), directory.to_string_lossy().into_owned()];
    if let Some(node_version) = &request.node_version {
        args.extend(["--node-version".into(), node_version.clone()]);
    }
    if let Some(version) = &request.version {
        args.extend(["--version".into(), version.clone()]);
    }
    if let Some(profile) = &request.profile {
        args.extend(["--profile".into(), profile.clone()]);
    }
    if let Some(workspace) = &request.workspace {
        args.extend(["--workspace".into(), workspace.clone()]);
    }
    args.push("--".into());
    args.extend(request.args);

    let command = app
        .shell()
        .sidecar("dsh-runtime")
        .map_err(|error| error.to_string())?
        .args(args);
    let (mut receiver, child) = command.spawn().map_err(|error| error.to_string())?;
    let pid = child.pid();
    {
        let mut process = manager
            .process
            .lock()
            .map_err(|_| "runtime process lock poisoned")?;
        process.child = Some(child);
        process.pid = Some(pid);
        process.version = request.version;
        process.endpoint = None;
    }
    emit_state(&app, "starting", Some(pid), None, None);

    let task_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut protocol = Vec::new();
        let mut exit_code = None;
        while let Some(event) = receiver.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let text = String::from_utf8_lossy(&line);
                    if let Some(endpoint) = dsh_endpoint(&text) {
                        if let Ok(mut process) = task_app.state::<RuntimeManager>().process.lock() {
                            process.endpoint = Some(endpoint.clone());
                        }
                        emit_state(&task_app, "running", Some(pid), None, Some(endpoint));
                    }
                    parse_protocol_line(&task_app, &line, &mut protocol);
                    if let Some(value) = protocol.pop() {
                        if value.get("kind").and_then(Value::as_str) == Some("status") {
                            if let Ok(status) = serde_json::from_value::<RuntimeStatus>(value) {
                                if let Ok(mut process) =
                                    task_app.state::<RuntimeManager>().process.lock()
                                {
                                    process.version.clone_from(&status.dsh_version);
                                }
                            }
                        }
                    }
                }
                CommandEvent::Stderr(line) => {
                    emit_log(&task_app, "stderr", String::from_utf8_lossy(&line))
                }
                CommandEvent::Error(error) => emit_log(&task_app, "stderr", error),
                CommandEvent::Terminated(payload) => exit_code = payload.code,
                _ => {}
            }
        }
        if let Ok(mut process) = task_app.state::<RuntimeManager>().process.lock() {
            process.child = None;
            process.pid = None;
            process.version = None;
            process.endpoint = None;
        }
        let message = exit_code
            .filter(|code| *code != 0)
            .map(|code| format!("DSH exited with code {code}"));
        emit_state(&task_app, "stopped", None, message, None);
    });
    Ok(())
}

#[tauri::command]
fn stop_dsh(app: AppHandle, manager: State<'_, RuntimeManager>) -> Result<(), String> {
    let (child, pid) = {
        let mut process = manager
            .process
            .lock()
            .map_err(|_| "runtime process lock poisoned")?;
        let pid = process.pid;
        process.pid = None;
        process.version = None;
        process.endpoint = None;
        (process.child.take(), pid)
    };
    if let Some(child) = child {
        #[cfg(not(windows))]
        let _ = pid;
        #[cfg(windows)]
        if let Some(pid) = pid {
            let status = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .status()
                .map_err(|error| error.to_string())?;
            if !status.success() {
                return Err(format!("taskkill failed with status {status}"));
            }
        }
        #[cfg(not(windows))]
        child.kill().map_err(|error| error.to_string())?;
        #[cfg(windows)]
        drop(child);
        emit_state(&app, "stopping", None, None, None);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeManager::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            default_workspace,
            runtime_status,
            setup_runtime,
            list_dsh_versions,
            list_remote_dsh_versions,
            install_dsh_version,
            set_default_dsh_version,
            remove_dsh_version,
            start_dsh,
            stop_dsh
        ])
        .run(tauri::generate_context!())
        .expect("error while running dsh-launcher");
}

#[cfg(test)]
mod tests {
    use super::dsh_endpoint;

    #[test]
    fn extracts_loopback_dsh_endpoint() {
        assert_eq!(
            dsh_endpoint("dsh web: http://127.0.0.1:4317 ready"),
            Some("http://127.0.0.1:4317".into())
        );
    }

    #[test]
    fn rejects_non_loopback_or_malformed_endpoint() {
        assert_eq!(dsh_endpoint("dsh web: http://0.0.0.0:4317"), None);
        assert_eq!(dsh_endpoint("dsh web: http://127.0.0.1:not-a-port"), None);
    }
}
