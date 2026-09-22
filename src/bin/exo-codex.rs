use exomemory::{
    client::{CODEX_LAUNCHER_CONFIG, CodexLauncherConfig},
    types::{Error, ErrorCode, Result},
};
use std::{
    fs,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::Command,
};

fn config_path(executable: &Path) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("EXOMEMORY_CODEX_CONFIG") {
        return Ok(path.into());
    }
    let bin = executable.parent().ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidInput,
            "installed Codex launcher has no parent directory",
        )
    })?;
    let local = bin.parent().ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidInput,
            "installed Codex launcher is not under a .local directory",
        )
    })?;
    Ok(local.join("share/exomemory").join(CODEX_LAUNCHER_CONFIG))
}

fn load(executable: &Path) -> Result<CodexLauncherConfig> {
    let path = config_path(executable)?;
    let bytes = fs::read(&path).map_err(|error| {
        Error::new(
            ErrorCode::Unavailable,
            format!("Codex launcher configuration {}: {error}", path.display()),
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        Error::new(
            ErrorCode::InvalidInput,
            format!(
                "invalid Codex launcher configuration {}: {error}",
                path.display()
            ),
        )
    })
}

fn validate_upstream(executable: &Path, upstream: &Path) -> Result<PathBuf> {
    if !upstream.is_absolute() {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "official Codex executable path must be absolute",
        ));
    }
    let launcher = fs::canonicalize(executable).map_err(|error| {
        Error::new(
            ErrorCode::Unavailable,
            format!("cannot resolve installed Codex launcher: {error}"),
        )
    })?;
    let upstream = fs::canonicalize(upstream).map_err(|error| {
        Error::new(
            ErrorCode::Unavailable,
            format!("cannot resolve official Codex executable: {error}"),
        )
    })?;
    if launcher == upstream {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "official Codex executable resolves to the Exomemory launcher; refusing recursion",
        ));
    }
    let metadata = fs::metadata(&upstream)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(Error::new(
            ErrorCode::Unavailable,
            format!(
                "official Codex executable is not executable: {}",
                upstream.display()
            ),
        ));
    }
    Ok(upstream)
}

fn native_routing(args: &[String]) -> bool {
    const COMMANDS: [&str; 30] = [
        "exec",
        "e",
        "review",
        "login",
        "logout",
        "mcp",
        "plugin",
        "app-server",
        "remote-control",
        "completion",
        "update",
        "doctor",
        "sandbox",
        "debug",
        "apply",
        "cloud",
        "exec-server",
        "features",
        "help",
        "queue",
        "archive",
        "delete",
        "unarchive",
        "agents",
        "migrate-rollouts",
        "app",
        "--help",
        "-h",
        "--version",
        "-V",
    ];
    args.iter().any(|argument| {
        COMMANDS.contains(&argument.as_str())
            || argument == "--remote"
            || argument.starts_with("--remote=")
    })
}

fn has_cwd(args: &[String]) -> bool {
    args.iter().any(|argument| {
        argument == "-C"
            || argument == "--cd"
            || argument.starts_with("--cd=")
            || (argument.starts_with("-C") && argument.len() > 2)
    })
}

fn run() -> Result<()> {
    let executable = std::env::current_exe()?;
    let config = load(&executable)?;
    let upstream = validate_upstream(&executable, &config.upstream)?;
    if !config.host_socket.is_absolute() {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "Codex host socket path must be absolute",
        ));
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut command = Command::new(upstream);
    if !native_routing(&args) {
        command
            .arg("--remote")
            .arg(format!("unix://{}", config.host_socket.display()));
        if !has_cwd(&args) {
            command
                .arg("--cd")
                .arg(fs::canonicalize(std::env::current_dir()?).map_err(|error| {
                    Error::new(
                        ErrorCode::InvalidInput,
                        format!("cannot resolve current directory: {error}"),
                    )
                })?);
        }
    }
    command.args(args);
    Err(command.exec().into())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Exomemory Codex launcher: {error}");
        std::process::exit(error.exit_status());
    }
}
