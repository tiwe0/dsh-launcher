use flate2::read::GzDecoder;
use semver::Version;
use serde::Serialize;
use serde_json::Value;
use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    thread,
};

const DEFAULT_NODE_VERSION: &str = "24.21.0";
const INITIAL_DSH_VERSION: &str = "0.1.6-alpha.2";
const NVM_VERSION: &str = "0.40.7";
const NODE_MIRROR: &str = "https://npmmirror.com/mirrors/node";
const NPM_REGISTRY: &str = "https://registry.npmmirror.com";

#[derive(Serialize)]
struct Status {
    kind: &'static str,
    ready: bool,
    running: bool,
    node_version: Option<String>,
    nvm_version: &'static str,
    dsh_version: Option<String>,
    nvm_dir: String,
    dsh_home: String,
    runtime_dir: String,
    endpoint: Option<String>,
    pid: Option<u32>,
}

#[derive(Serialize)]
struct VersionInfo {
    version: String,
    active: bool,
    is_default: bool,
    status: &'static str,
}

#[derive(Serialize)]
struct VersionList {
    kind: &'static str,
    versions: Vec<VersionInfo>,
}

#[derive(Serialize)]
struct RemoteVersionList {
    kind: &'static str,
    versions: Vec<String>,
}

#[derive(Serialize)]
struct LogLine<'a> {
    kind: &'static str,
    stream: &'a str,
    line: &'a str,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            emit_log("stderr", &error);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<u8, String> {
    let mut args = env::args_os().skip(1);
    let action = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "missing action".to_string())?;
    let runtime_dir = required_path(&mut args, "runtime directory")?;

    match action.as_str() {
        "inspect" => {
            ensure_no_extra(args)?;
            emit_status(&runtime_dir, false, None, None);
            Ok(0)
        }
        "setup" => {
            let archive = required_path(&mut args, "NVM archive")?;
            let node_version = required_node_version(&mut args)?;
            ensure_no_extra(args)?;
            setup(&runtime_dir, &archive, &node_version)?;
            emit_status(&runtime_dir, false, None, None);
            Ok(0)
        }
        "list" => {
            ensure_no_extra(args)?;
            emit_versions(&runtime_dir, None)?;
            Ok(0)
        }
        "remote" => {
            ensure_no_extra(args)?;
            emit_remote_versions(&runtime_dir)?;
            Ok(0)
        }
        "install" => {
            let version = required_version(&mut args)?;
            ensure_no_extra(args)?;
            install_dsh(&runtime_dir, &version)?;
            emit_versions(&runtime_dir, None)?;
            Ok(0)
        }
        "set-default" => {
            let version = required_version(&mut args)?;
            ensure_no_extra(args)?;
            set_default(&runtime_dir, &version)?;
            emit_versions(&runtime_dir, None)?;
            Ok(0)
        }
        "remove" => {
            let version = required_version(&mut args)?;
            ensure_no_extra(args)?;
            remove_dsh(&runtime_dir, &version)?;
            emit_versions(&runtime_dir, None)?;
            Ok(0)
        }
        "start" => start(runtime_dir, args.collect()),
        _ => Err(format!("unsupported action: {action}")),
    }
}

fn required_path(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
    name: &str,
) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {name}"))
}

fn required_version(args: &mut impl Iterator<Item = std::ffi::OsString>) -> Result<String, String> {
    let raw = args
        .next()
        .ok_or_else(|| "missing DSH version".to_string())?
        .into_string()
        .map_err(|_| "DSH version must be UTF-8".to_string())?;
    validate_version(&raw)?;
    Ok(raw)
}

fn required_node_version(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
) -> Result<String, String> {
    let raw = args
        .next()
        .ok_or_else(|| "missing Node.js version".to_string())?
        .into_string()
        .map_err(|_| "Node.js version must be UTF-8".to_string())?;
    validate_node_version(&raw)?;
    Ok(raw)
}

fn validate_node_version(raw: &str) -> Result<Version, String> {
    if raw.len() > 32 || raw.starts_with('v') || raw.contains('/') || raw.contains('\\') {
        return Err("Node.js version must be a plain semantic version".into());
    }
    Version::parse(raw).map_err(|_| "Node.js version must be a valid semantic version".into())
}

fn validate_version(raw: &str) -> Result<Version, String> {
    if raw.len() > 64 || raw.starts_with('v') || raw.contains('/') || raw.contains('\\') {
        return Err("DSH version must be a plain semantic version".into());
    }
    Version::parse(raw).map_err(|_| "DSH version must be a valid semantic version".into())
}

fn ensure_no_extra(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    if args.next().is_some() {
        return Err("unexpected extra arguments".into());
    }
    Ok(())
}

fn nvm_dir(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("nvm")
}

fn dsh_root(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("dsh")
}

fn dsh_prefix(runtime_dir: &Path, version: &str) -> PathBuf {
    dsh_root(runtime_dir).join(version)
}

fn default_file(runtime_dir: &Path) -> PathBuf {
    dsh_root(runtime_dir).join("default-version")
}

fn default_node_file(runtime_dir: &Path) -> PathBuf {
    nvm_dir(runtime_dir).join("default-version")
}

fn node_bin_dir(runtime_dir: &Path, version: &str) -> PathBuf {
    let root = nvm_dir(runtime_dir)
        .join("versions")
        .join("node")
        .join(format!("v{version}"));
    if cfg!(windows) {
        root
    } else {
        root.join("bin")
    }
}

fn node_path(runtime_dir: &Path, version: &str) -> PathBuf {
    node_bin_dir(runtime_dir, version).join(executable("node"))
}

fn npm_path(runtime_dir: &Path, version: &str) -> PathBuf {
    let name = if cfg!(windows) { "npm.cmd" } else { "npm" };
    node_bin_dir(runtime_dir, version).join(name)
}

fn dsh_entry(runtime_dir: &Path, version: &str) -> PathBuf {
    dsh_prefix(runtime_dir, version)
        .join("node_modules")
        .join("@deepseek-ai")
        .join("dsh")
        .join("lib")
        .join("bin.js")
}

fn executable(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn node_ready(runtime_dir: &Path, version: &str) -> bool {
    node_path(runtime_dir, version).is_file()
}

fn default_node_version(runtime_dir: &Path) -> Option<String> {
    let value = fs::read_to_string(default_node_file(runtime_dir)).ok()?;
    let value = value.trim();
    validate_node_version(value).ok()?;
    node_ready(runtime_dir, value).then(|| value.to_owned())
}

fn set_default_node_version(runtime_dir: &Path, version: &str) -> Result<(), String> {
    validate_node_version(version)?;
    if !node_ready(runtime_dir, version) {
        return Err(format!("Node.js {version} is not installed"));
    }
    fs::write(default_node_file(runtime_dir), format!("{version}\n"))
        .map_err(|error| error.to_string())
}

fn installed(runtime_dir: &Path, version: &str) -> bool {
    dsh_entry(runtime_dir, version).is_file()
}

fn default_version(runtime_dir: &Path) -> Option<String> {
    let value = fs::read_to_string(default_file(runtime_dir)).ok()?;
    let value = value.trim();
    validate_version(value).ok()?;
    installed(runtime_dir, value).then(|| value.to_owned())
}

fn emit_status(runtime_dir: &Path, running: bool, pid: Option<u32>, active_version: Option<&str>) {
    let selected = active_version
        .map(str::to_owned)
        .or_else(|| default_version(runtime_dir));
    let node_version = default_node_version(runtime_dir);
    let ready = node_version.is_some()
        && selected
            .as_deref()
            .is_some_and(|version| installed(runtime_dir, version));
    emit(&Status {
        kind: "status",
        ready,
        running,
        node_version,
        nvm_version: if cfg!(windows) {
            "portable-node"
        } else {
            NVM_VERSION
        },
        dsh_version: selected,
        nvm_dir: nvm_dir(runtime_dir).to_string_lossy().into_owned(),
        dsh_home: runtime_dir.join("dsh-home").to_string_lossy().into_owned(),
        runtime_dir: runtime_dir.to_string_lossy().into_owned(),
        endpoint: None,
        pid,
    });
}

fn emit_versions(runtime_dir: &Path, active: Option<&str>) -> Result<(), String> {
    let default = default_version(runtime_dir);
    let root = dsh_root(runtime_dir);
    let mut versions = Vec::new();
    if root.is_dir() {
        for entry in fs::read_dir(&root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            if !entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                continue;
            }
            let Some(version) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if validate_version(&version).is_err() {
                continue;
            }
            versions.push(VersionInfo {
                status: if installed(runtime_dir, &version) {
                    "installed"
                } else {
                    "incomplete"
                },
                active: active == Some(version.as_str()),
                is_default: default.as_deref() == Some(version.as_str()),
                version,
            });
        }
    }
    versions.sort_by(|a, b| {
        let left = Version::parse(&a.version).expect("validated version");
        let right = Version::parse(&b.version).expect("validated version");
        right.cmp(&left)
    });
    emit(&VersionList {
        kind: "versions",
        versions,
    });
    Ok(())
}

fn parse_remote_versions(raw: &[u8]) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_slice(raw)
        .map_err(|error| format!("cannot parse npm version response: {error}"))?;
    let values = value
        .as_array()
        .ok_or_else(|| "npm version response was not an array".to_string())?;
    let mut versions = values
        .iter()
        .filter_map(Value::as_str)
        .filter_map(|raw| {
            validate_version(raw)
                .ok()
                .map(|version| (version, raw.to_owned()))
        })
        .collect::<Vec<_>>();
    versions.sort_by(|(left, _), (right, _)| right.cmp(left));
    versions.dedup_by(|(_, left), (_, right)| left == right);
    Ok(versions.into_iter().map(|(_, raw)| raw).collect())
}

fn emit_remote_versions(runtime_dir: &Path) -> Result<(), String> {
    let node_version = default_node_version(runtime_dir)
        .ok_or_else(|| "Node.js runtime is not ready; run setup first".to_string())?;
    let mut command = Command::new(npm_path(runtime_dir, &node_version));
    command.args([
        "view",
        "@deepseek-ai/dsh",
        "versions",
        "--json",
        "--registry",
        NPM_REGISTRY,
    ]);
    sanitize_environment(&mut command, runtime_dir)?;
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "online DSH version check failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    emit(&RemoteVersionList {
        kind: "remote-versions",
        versions: parse_remote_versions(&output.stdout)?,
    });
    Ok(())
}

fn emit_log(stream: &str, line: &str) {
    emit(&LogLine {
        kind: "log",
        stream,
        line,
    });
}

fn emit(value: &impl Serialize) {
    if let Ok(line) = serde_json::to_string(value) {
        println!("{line}");
    }
}

fn setup(runtime_dir: &Path, archive: &Path, node_version: &str) -> Result<(), String> {
    fs::create_dir_all(runtime_dir).map_err(|error| error.to_string())?;
    if cfg!(windows) {
        setup_windows_node(runtime_dir, node_version)?;
    } else {
        setup_nvm_node(runtime_dir, archive, node_version)?;
    }
    set_default_node_version(runtime_dir, node_version)?;
    install_dsh(runtime_dir, INITIAL_DSH_VERSION)?;
    if default_version(runtime_dir).is_none() {
        set_default(runtime_dir, INITIAL_DSH_VERSION)?;
    }
    Ok(())
}

fn setup_nvm_node(runtime_dir: &Path, archive: &Path, node_version: &str) -> Result<(), String> {
    let nvm = nvm_dir(runtime_dir);
    if !nvm.join("nvm.sh").is_file() {
        emit_log("stdout", "Preparing isolated NVM runtime 0.40.7...");
        let staging = runtime_dir.join("nvm-staging");
        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
        }
        fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
        let file =
            File::open(archive).map_err(|error| format!("cannot open NVM archive: {error}"))?;
        tar::Archive::new(GzDecoder::new(file))
            .unpack(&staging)
            .map_err(|error| format!("cannot unpack NVM archive: {error}"))?;
        let unpacked = staging.join(format!("nvm-{NVM_VERSION}"));
        if !unpacked.join("nvm.sh").is_file() {
            return Err(format!("NVM archive did not contain nvm-{NVM_VERSION}"));
        }
        if nvm.exists() {
            fs::remove_dir_all(&nvm).map_err(|error| error.to_string())?;
        }
        fs::rename(&unpacked, &nvm).map_err(|error| error.to_string())?;
        fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
    }
    if !node_ready(runtime_dir, node_version) {
        emit_log(
            "stdout",
            &format!("Installing isolated Node.js {node_version}..."),
        );
        let script = r#"
set -eu
export NVM_DIR="$DSH_NVM_DIR"
. "$NVM_DIR/nvm.sh"
nvm install "$DSH_NODE_VERSION"
nvm use "$DSH_NODE_VERSION"
node --version
"#;
        let mut command = Command::new("bash");
        command
            .args(["-c", script])
            .env("DSH_NVM_DIR", &nvm)
            .env("DSH_NODE_VERSION", node_version);
        sanitize_environment(&mut command, runtime_dir)?;
        if relay_spawn(command)? != 0 || !node_ready(runtime_dir, node_version) {
            return Err("Node.js runtime setup failed".into());
        }
    }
    Ok(())
}

fn setup_windows_node(runtime_dir: &Path, node_version: &str) -> Result<(), String> {
    if node_ready(runtime_dir, node_version) {
        return Ok(());
    }
    let architecture = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    let node_root = nvm_dir(runtime_dir).join("versions").join("node");
    let destination = node_root.join(format!("v{node_version}"));
    let staging = runtime_dir.join(format!("node-staging-{node_version}"));
    let archive = runtime_dir.join(format!("node-v{node_version}-win-{architecture}.zip"));
    let extracted = staging.join(format!("node-v{node_version}-win-{architecture}"));
    let url = format!("{NODE_MIRROR}/v{node_version}/node-v{node_version}-win-{architecture}.zip");
    for path in [&staging, &archive] {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        } else if path.exists() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    emit_log(
        "stdout",
        &format!("Installing isolated portable Node.js {node_version}..."),
    );
    let script = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest -UseBasicParsing -Uri $env:DSH_NODE_URL -OutFile $env:DSH_NODE_ARCHIVE
Expand-Archive -LiteralPath $env:DSH_NODE_ARCHIVE -DestinationPath $env:DSH_NODE_STAGING -Force
"#;
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .env("DSH_NODE_URL", url)
        .env("DSH_NODE_ARCHIVE", &archive)
        .env("DSH_NODE_STAGING", &staging);
    sanitize_environment(&mut command, runtime_dir)?;
    if relay_spawn(command)? != 0 || !extracted.join(executable("node")).is_file() {
        return Err("portable Node.js runtime setup failed".into());
    }
    fs::create_dir_all(&node_root).map_err(|error| error.to_string())?;
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(|error| error.to_string())?;
    }
    fs::rename(extracted, &destination).map_err(|error| error.to_string())?;
    fs::remove_dir_all(staging).map_err(|error| error.to_string())?;
    fs::remove_file(archive).map_err(|error| error.to_string())?;
    Ok(())
}

fn install_dsh(runtime_dir: &Path, version: &str) -> Result<(), String> {
    validate_version(version)?;
    let node_version = default_node_version(runtime_dir)
        .ok_or_else(|| "Node.js runtime is not ready; run setup first".to_string())?;
    if installed(runtime_dir, version) {
        return Ok(());
    }
    let prefix = dsh_prefix(runtime_dir, version);
    fs::create_dir_all(&prefix).map_err(|error| error.to_string())?;
    emit_log("stdout", &format!("Installing isolated DSH {version}..."));
    let npm = npm_path(runtime_dir, &node_version);
    let package = format!("@deepseek-ai/dsh@{version}");
    let mut command = Command::new(npm);
    command
        .args(["install", "--prefix"])
        .arg(&prefix)
        .arg("--save-exact")
        .arg(package);
    let mut paths = vec![node_bin_dir(runtime_dir, &node_version)];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    command.env(
        "PATH",
        env::join_paths(paths).map_err(|error| error.to_string())?,
    );
    sanitize_environment(&mut command, runtime_dir)?;
    let code = relay_spawn(command)?;
    if code != 0 || !installed(runtime_dir, version) {
        return Err(format!(
            "DSH {version} installation failed with exit code {code}"
        ));
    }
    Ok(())
}

fn set_default(runtime_dir: &Path, version: &str) -> Result<(), String> {
    validate_version(version)?;
    if !installed(runtime_dir, version) {
        return Err(format!("DSH {version} is not installed"));
    }
    fs::create_dir_all(dsh_root(runtime_dir)).map_err(|error| error.to_string())?;
    let temporary = dsh_root(runtime_dir).join("default-version.tmp");
    fs::write(&temporary, format!("{version}\n")).map_err(|error| error.to_string())?;
    fs::rename(temporary, default_file(runtime_dir)).map_err(|error| error.to_string())
}

fn remove_dsh(runtime_dir: &Path, version: &str) -> Result<(), String> {
    validate_version(version)?;
    if default_version(runtime_dir).as_deref() == Some(version) {
        return Err("cannot remove the default DSH version; select another default first".into());
    }
    let prefix = dsh_prefix(runtime_dir, version);
    if prefix.exists() {
        fs::remove_dir_all(prefix).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn start(runtime_dir: PathBuf, raw_args: Vec<std::ffi::OsString>) -> Result<u8, String> {
    let mut node_version: Option<String> = None;
    let mut version: Option<String> = None;
    let mut profile: Option<String> = None;
    let mut workspace: Option<PathBuf> = None;
    let mut dsh_args = Vec::new();
    let mut iter = raw_args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.to_str() {
            Some("--version") => version = Some(required_version(&mut iter)?),
            Some("--node-version") => node_version = Some(required_node_version(&mut iter)?),
            Some("--profile") => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing profile value".to_string())?
                    .into_string()
                    .map_err(|_| "profile must be UTF-8".to_string())?;
                if value.is_empty()
                    || value.len() > 64
                    || !value
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
                {
                    return Err("profile contains unsupported characters".into());
                }
                profile = Some(value);
            }
            Some("--workspace") => {
                let value = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "missing workspace value".to_string())?,
                );
                let canonical = value
                    .canonicalize()
                    .map_err(|error| format!("invalid workspace: {error}"))?;
                if !canonical.is_dir() {
                    return Err("workspace must be a directory".into());
                }
                workspace = Some(canonical);
            }
            Some("--") => {
                dsh_args.extend(iter);
                break;
            }
            _ => return Err("invalid start arguments".into()),
        }
    }
    let version = version
        .or_else(|| default_version(&runtime_dir))
        .ok_or_else(|| "no DSH version selected and no default configured".to_string())?;
    validate_version(&version)?;
    let node_version = node_version
        .or_else(|| default_node_version(&runtime_dir))
        .unwrap_or_else(|| DEFAULT_NODE_VERSION.to_owned());
    if !node_ready(&runtime_dir, &node_version) {
        return Err(format!("Node.js {node_version} is not installed"));
    }
    let entry = dsh_entry(&runtime_dir, &version);
    if !entry.is_file() {
        return Err(format!("DSH {version} is not installed"));
    }

    let mut command = Command::new(node_path(&runtime_dir, &node_version));
    command.arg(entry);
    if let Some(profile) = profile {
        command.args(["--profile", &profile]);
    }
    command.args(dsh_args);
    if let Some(workspace) = workspace {
        command.current_dir(workspace);
    }
    command.env("NVM_DIR", nvm_dir(&runtime_dir));
    sanitize_environment(&mut command, &runtime_dir)?;
    command.stdin(Stdio::null());
    emit_status(
        &runtime_dir,
        false,
        Some(std::process::id()),
        Some(&version),
    );

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::process::CommandExt;
        let _ = io::stdout().flush();
        let error = command.exec();
        Err(format!("cannot start DSH {version}: {error}"))
    }

    #[cfg(windows)]
    {
        relay_spawn(command)
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = command;
        Err("unsupported operating system".into())
    }
}

fn sanitize_environment(command: &mut Command, runtime_dir: &Path) -> Result<(), String> {
    let home = runtime_dir.join("home");
    let dsh_home = runtime_dir.join("dsh-home");
    let npm_cache = runtime_dir.join("npm-cache");
    let xdg_root = runtime_dir.join("xdg");
    for directory in [
        &home,
        &dsh_home,
        &npm_cache,
        &xdg_root.join("config"),
        &xdg_root.join("cache"),
        &xdg_root.join("data"),
    ] {
        fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    }
    command
        .env("HOME", home)
        .env("DSH_HOME", dsh_home)
        .env("NPM_CONFIG_CACHE", npm_cache)
        .env("NPM_CONFIG_REGISTRY", NPM_REGISTRY)
        .env("NPM_CONFIG_DISTURL", NODE_MIRROR)
        .env("NVM_NODEJS_ORG_MIRROR", NODE_MIRROR)
        .env("NODEJS_ORG_MIRROR", NODE_MIRROR)
        .env("XDG_CONFIG_HOME", xdg_root.join("config"))
        .env("XDG_CACHE_HOME", xdg_root.join("cache"))
        .env("XDG_DATA_HOME", xdg_root.join("data"))
        .env("NVM_NO_SOURCE_FALLBACK", "1")
        .env_remove("NPM_CONFIG_PREFIX")
        .env_remove("PREFIX")
        .env_remove("NODE_OPTIONS");
    Ok(())
}

fn relay_spawn(mut command: Command) -> Result<u8, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    relay_process(command.spawn().map_err(|error| error.to_string())?)
}

fn relay_process(mut child: std::process::Child) -> Result<u8, String> {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let out_thread = thread::spawn(move || relay_reader(stdout, "stdout"));
    let err_thread = thread::spawn(move || relay_reader(stderr, "stderr"));
    let status = child.wait().map_err(|error| error.to_string())?;
    let _ = out_thread.join();
    let _ = err_thread.join();
    Ok(status.code().unwrap_or(1).clamp(0, 255) as u8)
}

fn relay_reader(reader: Option<impl io::Read>, stream: &'static str) {
    if let Some(reader) = reader {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            emit_log(stream, &line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_remote_versions, sanitize_environment, validate_node_version, validate_version,
        NODE_MIRROR, NPM_REGISTRY,
    };
    use std::{collections::HashMap, process::Command};

    #[test]
    fn accepts_semantic_versions() {
        assert!(validate_version("0.1.6-alpha.2").is_ok());
        assert!(validate_version("1.2.3+build.4").is_ok());
    }

    #[test]
    fn rejects_paths_shell_text_and_v_prefixes() {
        for value in ["v1.2.3", "../1.2.3", "1.2.3;rm", "1.2", ""] {
            assert!(validate_version(value).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn accepts_only_plain_node_semver_versions() {
        assert!(validate_node_version("24.21.0").is_ok());
        assert!(validate_node_version("v24.21.0").is_err());
        assert!(validate_node_version("24").is_err());
        assert!(validate_node_version("../24.21.0").is_err());
    }

    #[test]
    fn isolated_commands_default_to_domestic_mirrors() {
        let runtime = std::env::temp_dir().join(format!("dsh-runtime-env-{}", std::process::id()));
        let mut command = Command::new("true");
        sanitize_environment(&mut command, &runtime).expect("configure isolated environment");
        let variables = command
            .get_envs()
            .filter_map(|(key, value)| {
                value.map(|value| {
                    (
                        key.to_string_lossy().into_owned(),
                        value.to_string_lossy().into_owned(),
                    )
                })
            })
            .collect::<HashMap<_, _>>();
        assert_eq!(
            variables.get("NVM_NODEJS_ORG_MIRROR").map(String::as_str),
            Some(NODE_MIRROR)
        );
        assert_eq!(
            variables.get("NODEJS_ORG_MIRROR").map(String::as_str),
            Some(NODE_MIRROR)
        );
        assert_eq!(
            variables.get("NPM_CONFIG_REGISTRY").map(String::as_str),
            Some(NPM_REGISTRY)
        );
        let _ = std::fs::remove_dir_all(runtime);
    }

    #[test]
    fn parses_and_sorts_remote_dsh_versions() {
        let versions =
            parse_remote_versions(br#"["0.1.5-rc.2","invalid","0.1.6-alpha.1","0.1.6-alpha.2"]"#)
                .expect("parse remote versions");
        assert_eq!(versions, ["0.1.6-alpha.2", "0.1.6-alpha.1", "0.1.5-rc.2"]);
    }
}
