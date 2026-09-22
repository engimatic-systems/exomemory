mod support;
use exomemory::{
    client::USER_CONFIG_RELATIVE,
    policy::{Policy, Profile},
    types::*,
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Child, Command, Stdio},
};
use support::*;

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn installer(home: &std::path::Path, operation: &str, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_exo-install"))
        .args([operation, "--home", home.to_str().unwrap(), "--no-start"])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn installed_hook_delivers_only_the_public_ghost_shape() {
    use std::{
        io::Write,
        os::unix::net::{UnixListener, UnixStream},
        time::{Duration, Instant},
    };
    use tungstenite::Message;

    let run = Run::new();
    let home = &run.scope;
    let native = run.root.join("native-codex");
    fs::write(
        &native,
        "#!/bin/sh\nif [ \"${1-}\" = --version ]; then echo 'codex-cli 0.155.1'; fi\n",
    )
    .unwrap();
    fs::set_permissions(&native, fs::Permissions::from_mode(0o700)).unwrap();
    let installed = installer(home, "install", &["--codex", native.to_str().unwrap()]);
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );

    let runtime = home.join(".local/share/exomemory");
    let installed_exoctl = fs::read(home.join(".local/bin/exoctl")).unwrap();
    assert_eq!(
        installed_exoctl,
        fs::read(runtime.join("bin/exoctl")).unwrap()
    );
    assert!(!installed_exoctl.starts_with(b"#!"));
    let socket = runtime.join("exo.sock");
    let mut daemon = ChildGuard(
        Command::new(runtime.join("bin/exod"))
            .args([
                "--profile",
                "local",
                "--runtime",
                runtime.to_str().unwrap(),
                "--scope-root",
                home.to_str().unwrap(),
                "--tenant",
                "personal",
                "--database",
                runtime.join("memory.sqlite").to_str().unwrap(),
                "--socket",
                socket.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while UnixStream::connect(&socket).is_err() {
        assert!(
            daemon.0.try_wait().unwrap().is_none(),
            "installed daemon exited"
        );
        assert!(
            Instant::now() < deadline,
            "installed daemon startup timeout"
        );
        std::thread::sleep(Duration::from_millis(10));
    }

    let mut input = draft(&run);
    input.scope = Scope::Dir {
        root: home.to_path_buf(),
    };
    input.provenance.origin = "installed-origin".into();
    let mut create = Command::new(home.join(".local/bin/exoctl"))
        .args(["create", "--json", "-"])
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .current_dir(home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    create
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&input).unwrap())
        .unwrap();
    let created = create.wait_with_output().unwrap();
    assert!(
        created.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr),
    );
    let diagnostic = String::from_utf8_lossy(&created.stderr);
    assert!(
        diagnostic.contains("installed user configuration"),
        "{diagnostic}"
    );
    assert!(diagnostic.contains(USER_CONFIG_RELATIVE), "{diagnostic}");
    let created: serde_json::Value = serde_json::from_slice(&created.stdout).unwrap();
    let key = created["result"]["Ok"]["metadata"]["ghost_key"]
        .as_str()
        .unwrap()
        .to_owned();

    let codex_socket = runtime.join("codex.sock");
    let listener = UnixListener::bind(&codex_socket).unwrap();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        let initialize: serde_json::Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        socket
            .send(Message::Text(
                serde_json::json!({"id":initialize["id"],"result":{"userAgent":"exomemory_hook/0.155.1 (installed-test)"}})
                    .to_string()
                    .into(),
            ))
            .unwrap();
        socket.read().unwrap();
        let insertion: serde_json::Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        let text = insertion["params"]["items"][0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_owned();
        socket
            .send(Message::Text(
                serde_json::json!({"id":insertion["id"],"result":{}})
                    .to_string()
                    .into(),
            ))
            .unwrap();
        text
    });
    let mut hook = Command::new(runtime.join("bin/exo-codex-hook"))
        .arg("--env")
        .env("EXOMEMORY_HOOK_CONFIG", runtime.join("hook-config.json"))
        .current_dir(home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    hook.stdin
        .take()
        .unwrap()
        .write_all(
            &serde_json::to_vec(&serde_json::json!({
                "session_id":"installed-shape",
                "cwd":home,
                "hook_event_name":"UserPromptSubmit",
                "prompt":"test"
            }))
            .unwrap(),
        )
        .unwrap();
    let hook = hook.wait_with_output().unwrap();
    assert!(
        hook.status.success(),
        "{}",
        String::from_utf8_lossy(&hook.stderr)
    );
    let delivered = server.join().unwrap();
    let public: serde_json::Value =
        serde_json::from_str(delivered.lines().nth(1).unwrap()).unwrap();
    assert_eq!(public["data"]["ghost_key"], key);
    assert!(public["data"].get("source_id").is_none());
    assert!(public["data"].get("provenance").is_none());

    let metadata = Command::new(home.join(".local/bin/exoctl"))
        .args(["meta", &key])
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .current_dir(home)
        .output()
        .unwrap();
    assert!(metadata.status.success());
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout).unwrap();
    assert_eq!(
        metadata["result"]["Ok"]["provenance"]["origin"],
        "installed-origin"
    );
    let codex_session = Command::new(home.join(".local/bin/exoctl"))
        .args(["annoy", &key])
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .env("CODEX_THREAD_ID", "codex-thread")
        .current_dir(home)
        .output()
        .unwrap();
    assert!(codex_session.status.success());
    let codex_session: serde_json::Value = serde_json::from_slice(&codex_session.stdout).unwrap();
    assert_eq!(codex_session["result"]["Ok"]["session"], "codex-thread");
    let terminal_session = Command::new(home.join(".local/bin/exoctl"))
        .args(["annoy", &key])
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .current_dir(home)
        .output()
        .unwrap();
    assert!(terminal_session.status.success());
    let terminal_session: serde_json::Value =
        serde_json::from_slice(&terminal_session.stdout).unwrap();
    assert_eq!(terminal_session["result"]["Ok"]["session"], "terminal");

    let alternate = runtime.join("alternate-client.json");
    let mut alternate_config: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(USER_CONFIG_RELATIVE)).unwrap()).unwrap();
    alternate_config["tenant"] = "wrong-lower-precedence-value".into();
    fs::write(&alternate, serde_json::to_vec(&alternate_config).unwrap()).unwrap();
    let overridden = Command::new(home.join(".local/bin/exoctl"))
        .args(["--tenant", "personal", "list-trigger-types"])
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .env("EXOMEMORY_CONFIG", &alternate)
        .current_dir(home)
        .output()
        .unwrap();
    assert!(
        overridden.status.success(),
        "{}",
        String::from_utf8_lossy(&overridden.stdout)
    );
    assert!(String::from_utf8_lossy(&overridden.stderr).contains(alternate.to_str().unwrap()));

    let missing = runtime.join("missing-client.json");
    let explicit = Command::new(home.join(".local/bin/exoctl"))
        .args([
            "--profile",
            "local",
            "--runtime",
            runtime.to_str().unwrap(),
            "--scope-root",
            home.to_str().unwrap(),
            "--socket",
            socket.to_str().unwrap(),
            "--tenant",
            "personal",
            "--session",
            "explicit-test",
            "--cwd",
            home.to_str().unwrap(),
            "list-trigger-types",
        ])
        .env_clear()
        .env("EXOMEMORY_CONFIG", &missing)
        .current_dir(home)
        .output()
        .unwrap();
    assert!(explicit.status.success());
    assert!(String::from_utf8_lossy(&explicit.stderr).contains("explicit command options"));
    let missing = Command::new(home.join(".local/bin/exoctl"))
        .arg("list-trigger-types")
        .env_clear()
        .env("EXOMEMORY_CONFIG", &missing)
        .current_dir(home)
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stdout).contains("missing-client.json"));
    drop(daemon);
    assert!(installer(home, "uninstall", &[]).status.success());
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
fn legacy_exoctl_wrapper_upgrades_repairs_and_uninstalls_without_losing_the_store() {
    use exomemory::data::Storage;

    let run = Run::new();
    let home = &run.scope;
    fs::create_dir_all(home.join(".local/bin")).unwrap();
    fs::write(home.join(".local/bin/exoctl"), "prior-cli").unwrap();
    let native = run.root.join("native-codex");
    fs::write(
        &native,
        "#!/bin/sh\nif [ \"${1-}\" = --version ]; then echo 'codex-cli 0.155.1'; fi\n",
    )
    .unwrap();
    fs::set_permissions(&native, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        installer(home, "install", &["--codex", native.to_str().unwrap()])
            .status
            .success()
    );
    let runtime = home.join(".local/share/exomemory");
    let settings: exomemory::codex::HookConfig =
        serde_json::from_slice(&fs::read(runtime.join("hook-config.json")).unwrap()).unwrap();
    let policy = settings
        .profile
        .policy(&settings.runtime, &settings.scope_root, settings.tenant)
        .unwrap();
    let mut database =
        exomemory::sqlite::Sqlite::open(&policy, &runtime.join("memory.sqlite")).unwrap();
    let context = Context {
        tenant: "personal".into(),
        session: "upgrade-fixture".into(),
        cwd: home.to_path_buf(),
    };
    let mut input = draft(&run);
    input.scope = Scope::Dir {
        root: home.to_path_buf(),
    };
    let record = database.create(&policy, &context, input).unwrap();
    drop(database);

    let legacy = format!(
        "#!/bin/sh\nexec '{}' --profile local --runtime '{}' --scope-root '{}' --tenant personal --session terminal --cwd '{}' --socket '{}' \"$@\"\n",
        runtime.join("bin/exoctl").display(),
        runtime.display(),
        home.display(),
        home.display(),
        runtime.join("exo.sock").display()
    );
    fs::write(home.join(".local/bin/exoctl"), &legacy).unwrap();
    fs::set_permissions(
        home.join(".local/bin/exoctl"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let manifest_path = runtime.join("installation.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let config_path = home.join(USER_CONFIG_RELATIVE);
    manifest["files"]
        .as_array_mut()
        .unwrap()
        .retain(|file| file["path"].as_str() != config_path.to_str());
    let command = manifest["files"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|file| file["path"].as_str() == home.join(".local/bin/exoctl").to_str())
        .unwrap();
    command["installed"] = serde_json::to_value(legacy.as_bytes().to_vec()).unwrap();
    command
        .as_object_mut()
        .unwrap()
        .remove("installed_fingerprint");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    fs::remove_file(&config_path).unwrap();

    for operation in ["upgrade", "repair"] {
        let result = installer(home, operation, &[]);
        assert!(
            result.status.success(),
            "{operation}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let command = fs::read(home.join(".local/bin/exoctl")).unwrap();
    assert_eq!(command, fs::read(runtime.join("bin/exoctl")).unwrap());
    assert!(!command.starts_with(b"#!"));
    let config: exomemory::client::ClientConfig =
        serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    assert_eq!(config.profile, Profile::Local);
    assert_eq!(config.runtime, runtime);
    assert_eq!(config.scope_root, *home);

    let mut database =
        exomemory::sqlite::Sqlite::open(&policy, &runtime.join("memory.sqlite")).unwrap();
    assert_eq!(
        database
            .read(&policy, &context, &record.metadata.ghost_key)
            .unwrap(),
        record
    );
    assert!(installer(home, "uninstall", &[]).status.success());
    assert_eq!(
        fs::read_to_string(home.join(".local/bin/exoctl")).unwrap(),
        "prior-cli"
    );
    assert!(!config_path.exists());
    assert_eq!(
        database
            .read(&policy, &context, &record.metadata.ghost_key)
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
