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
