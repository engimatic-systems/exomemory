mod support;
use exomemory::{
    policy::{Policy, Profile},
    types::*,
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
};
use support::*;
fn installer(home: &std::path::Path, operation: &str, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_exo-install"))
        .args([operation, "--home", home.to_str().unwrap(), "--no-start"])
        .args(extra)
        .output()
        .unwrap()
}
#[test]
fn local_profile_requires_private_runtime_and_keeps_directory_boundaries() {
    let run = Run::new();
    let runtime = run.root.join("personal");
    fs::create_dir(&runtime).unwrap();
    fs::set_permissions(&runtime, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Policy::local_profile(&runtime, &run.scope, "personal".into()).is_err());
    fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700)).unwrap();
    let policy = Profile::Local
        .policy(&runtime, &run.scope, run.policy.tenant.clone())
        .unwrap();
    let mut db = exomemory::sqlite::Sqlite::open(&policy, &runtime.join("memory.sqlite")).unwrap();
    use exomemory::data::Storage;
    let a = run.scope.join("project-a");
    let b = run.scope.join("project-b");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    let mut context = run.context.clone();
    context.cwd = a.clone();
    let mut input = draft(&run);
    input.scope = Scope::Dir { root: a.clone() };
    let record = db.create(&policy, &context, input.clone()).unwrap();
    context.cwd = b;
    assert_eq!(
        db.read(&policy, &context, &record.metadata.ghost_key)
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    let mut query = event(&run, 1);
    query.context = context.clone();
    assert!(
        db.candidates(&policy, &query)
            .unwrap()
            .candidates
            .is_empty()
    );
    assert_eq!(
        db.annoy(&policy, &context, &record.metadata.ghost_key)
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    assert_eq!(
        db.overwrite(
            &policy,
            &context,
            &record.metadata.ghost_key,
            1,
            input.clone()
        )
        .unwrap_err()
        .code,
        ErrorCode::Forbidden
    );
    context.cwd = a.join("..");
    assert!(policy.context(&context).is_err());
    let other = Run::new();
    std::os::unix::fs::symlink(&other.scope, run.scope.join("escape")).unwrap();
    context.cwd = run.scope.join("escape");
    assert!(policy.context(&context).is_err());
    input.scope = Scope::Global;
    assert!(db.create(&policy, &run.context, input).is_err());
    assert!(Policy::test_profile(&runtime, &run.scope, "personal".into()).is_err());
}
#[test]
fn installation_preserves_existing_configuration_and_uninstall_preserves_memories() {
    let run = Run::new();
    let home = &run.scope;
    fs::create_dir_all(home.join(".codex")).unwrap();
    fs::create_dir_all(home.join(".local/bin")).unwrap();
    fs::write(
        home.join(".codex/hooks.json"),
        r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"echo prior-hook"}]}]}}"#,
    )
    .unwrap();
    fs::write(home.join(".codex/config.toml"), "model = 'user-choice'\n").unwrap();
    fs::write(home.join(".local/bin/exoctl"), "prior-cli").unwrap();
    let native = run.root.join("native-codex");
    fs::write(&native,"#!/bin/sh\nif [ \"${1-}\" = --version ]; then echo 'codex-cli 0.155.1'; else printf '%s\\n' \"$@\"; fi\n").unwrap();
    fs::set_permissions(&native, fs::Permissions::from_mode(0o700)).unwrap();
    let result = installer(home, "install", &["--codex", native.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        !installer(home, "install", &["--codex", native.to_str().unwrap()])
            .status
            .success()
    );
    assert_eq!(
        fs::read_to_string(home.join(".codex/config.toml")).unwrap(),
        "model = 'user-choice'\n"
    );
    let hooks: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".codex/hooks.json")).unwrap()).unwrap();
    assert_eq!(hooks["hooks"]["Stop"].as_array().unwrap().len(), 2);
    let launched = Command::new(home.join(".local/bin/codex"))
        .arg("my prompt")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&launched.stdout).starts_with("--remote\nunix://"));
    assert!(String::from_utf8_lossy(&launched.stdout).contains("--cd\n"));
    let explicit = Command::new(home.join(".local/bin/codex"))
        .args(["--cd", home.to_str().unwrap(), "hello"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&explicit.stdout)
            .matches("--cd\n")
            .count(),
        1
    );
    let native_help = Command::new(home.join(".local/bin/codex"))
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&native_help.stdout), "--help\n");
    let config = home.join(".local/share/exomemory/hook-config.json");
    let inert = Command::new(home.join(".local/share/exomemory/bin/exo-codex-hook"))
        .arg("--env")
        .env_remove("EXOMEMORY_HOOK_CONFIG")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(inert.status.success());
    assert_eq!(String::from_utf8_lossy(&inert.stdout).trim(), "{}");
    let settings: exomemory::codex::HookConfig =
        serde_json::from_slice(&fs::read(config).unwrap()).unwrap();
    let policy = settings
        .profile
        .policy(&settings.runtime, &settings.scope_root, settings.tenant)
        .unwrap();
    let mut db =
        exomemory::sqlite::Sqlite::open(&policy, &settings.runtime.join("memory.sqlite")).unwrap();
    use exomemory::data::Storage;
    let mut context = run.context.clone();
    context.tenant = "personal".into();
    let record = db.create(&policy, &context, draft(&run)).unwrap();
    let result = installer(home, "uninstall", &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read_to_string(home.join(".local/bin/exoctl")).unwrap(),
        "prior-cli"
    );
    assert!(!home.join(".local/bin/codex").exists());
    let restored: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".codex/hooks.json")).unwrap()).unwrap();
    assert_eq!(restored["hooks"]["Stop"].as_array().unwrap().len(), 1);
    assert_eq!(
        restored["hooks"]["Stop"][0]["hooks"][0]["command"],
        "echo prior-hook"
    );
    assert_eq!(
        db.read(&policy, &context, &record.metadata.ghost_key)
            .unwrap(),
        record
    );
}

#[test]
#[cfg(target_os = "macos")]
fn login_shell_homebrew_order_is_repaired_detected_and_reversibly_managed() {
    let run = Run::new();
    let home = &run.scope;
    let native_dir = home.join("brew/bin");
    fs::create_dir_all(&native_dir).unwrap();
    let native = native_dir.join("codex");
    fs::write(&native, "#!/bin/sh\nif [ \"${1-}\" = --version ]; then echo 'codex-cli 0.155.1'; else printf '%s\n' \"$@\"; fi\n").unwrap();
    fs::set_permissions(&native, fs::Permissions::from_mode(0o700)).unwrap();
    let original_profile = format!(
        "# Homebrew shellenv prepends this directory after .zshenv.\nexport PATH='{}':\"$PATH\"\n",
        native_dir.display()
    );
    fs::write(
        home.join(".zshenv"),
        format!(
            "export PATH='{}':\"$PATH\"\n",
            home.join(".local/bin").display()
        ),
    )
    .unwrap();
    fs::write(home.join(".zprofile"), &original_profile).unwrap();
    fs::write(home.join(".zshrc"), "# existing interactive setup\n").unwrap();
    let result = installer(home, "install", &["--codex", native.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let correct_path = format!(
        "{}:{}:/usr/bin:/bin",
        home.join(".local/bin").display(),
        native_dir.display()
    );
    let check = |operation: &str, path: &str| {
        Command::new(env!("CARGO_BIN_EXE_exo-install"))
            .args([operation, "--home", home.to_str().unwrap(), "--no-start"])
            .env("PATH", path)
            .env("ZDOTDIR", home)
            .output()
            .unwrap()
    };
    assert!(check("check-routing", &correct_path).status.success());
    // Reconstruct the old installation: healthy launcher PATH in this process, broken new terminals.
    fs::write(home.join(".zprofile"), &original_profile).unwrap();
    fs::write(home.join(".zshrc"), "# existing interactive setup\n").unwrap();
    let manifest = home.join(".local/share/exomemory/installation.json");
    let mut saved: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    saved["files"].as_array_mut().unwrap().retain(|f| {
        ![home.join(".zprofile"), home.join(".zshrc")]
            .iter()
            .any(|p| f["path"] == p.to_str().unwrap())
    });
    fs::write(&manifest, serde_json::to_vec(&saved).unwrap()).unwrap();
    let broken = check("status", &correct_path);
    assert_eq!(broken.status.code(), Some(3));
    let error = String::from_utf8_lossy(&broken.stderr);
    assert!(error.contains("fresh login shell selects"), "{error}");
    assert!(error.contains(native.to_str().unwrap()));
    for _ in 0..2 {
        let repair = installer(home, "repair-path", &[]);
        assert!(
            repair.status.success(),
            "{}",
            String::from_utf8_lossy(&repair.stderr)
        );
    }
    for name in [".zprofile", ".zshrc"] {
        assert_eq!(
            fs::read_to_string(home.join(name))
                .unwrap()
                .matches("# >>> Exomemory PATH >>>")
                .count(),
            1
        );
    }
    assert!(check("check-routing", &correct_path).status.success());
    let wrong_path = format!(
        "{}:{}:/usr/bin:/bin",
        native_dir.display(),
        home.join(".local/bin").display()
    );
    let wrong = check("status", &wrong_path);
    assert_eq!(wrong.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&wrong.stderr).contains("PATH selects"));
    let interactive = Command::new("/bin/zsh")
        .args(["-ic", "command -v codex"])
        .env("ZDOTDIR", home)
        .env("PATH", &wrong_path)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&interactive.stdout).trim(),
        home.join(".local/bin/codex").to_str().unwrap()
    );
    let routed = Command::new("/bin/zsh")
        .args(["-lic", "codex fixture-prompt"])
        .env("ZDOTDIR", home)
        .env("PATH", &wrong_path)
        .current_dir(home)
        .output()
        .unwrap();
    let arguments = String::from_utf8_lossy(&routed.stdout);
    assert!(arguments.starts_with("--remote\nunix://"), "{arguments}");
    assert!(
        arguments.contains(
            home.join(".local/share/exomemory/codex.sock")
                .to_str()
                .unwrap()
        )
    );
    // A later user edit must survive removal of just the installer-owned block.
    let path = home.join(".zprofile");
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("# later user setting\n");
    fs::write(&path, text).unwrap();
    assert!(installer(home, "uninstall", &[]).status.success());
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        format!("{original_profile}# later user setting\n")
    );
    assert_eq!(
        fs::read_to_string(home.join(".zshrc")).unwrap(),
        "# existing interactive setup\n"
    );
}

#[test]
fn unhosted_startup_reports_inactive_recall_without_blocking_or_injecting() {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_exo-codex-hook"))
        .arg("--env")
        .env_remove("EXOMEMORY_HOOK_CONFIG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"hook_event_name":"SessionStart"}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "{}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("recall is inactive"));
}
