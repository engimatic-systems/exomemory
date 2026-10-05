mod support;
use exomemory::{attempts::AttemptStore, codex::HookConfig, types::*};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::{fs::OpenOptionsExt, net::UnixStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use support::*;
use tungstenite::{Message, WebSocket};

struct Host {
    _child: HostProcess,
    socket: WebSocket<UnixStream>,
    transcript: fs::File,
    observed: Vec<Value>,
}
struct HostProcess(Child);
impl Drop for HostProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
struct PrivateAuth(PathBuf);
impl Drop for PrivateAuth {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{}:{hash:016x}", bytes.len())
}
impl Host {
    fn send(&mut self, value: Value) {
        use std::io::Write;
        writeln!(self.transcript, "{}", json!({"send":value})).unwrap();
        self.socket
            .send(Message::Text(value.to_string().into()))
            .unwrap();
    }
    fn until(&mut self, predicate: impl Fn(&Value) -> bool) -> Value {
        use std::io::Write;
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            assert!(Instant::now() < deadline, "Codex event timeout");
            self.socket
                .get_mut()
                .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
                .unwrap();
            if let Message::Text(text) = self.socket.read().unwrap() {
                let value: Value = serde_json::from_str(&text).unwrap();
                writeln!(self.transcript, "{}", json!({"receive":value})).unwrap();
                self.observed.push(value.clone());
                if predicate(&value) {
                    return value;
                }
            }
        }
    }
    fn rpc(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({"id":id,"method":method,"params":params}));
        let reply = self.until(|v| v["id"] == id);
        assert!(reply.get("error").is_none(), "{reply}");
        reply["result"].clone()
    }
    fn turn(&mut self, id: u64, thread: &Value, prompt: &str) {
        self.rpc(
            id,
            "turn/start",
            json!({"threadId":thread,"input":[{"type":"text","text":prompt,"text_elements":[]}]}),
        );
        let done =
            self.until(|v| v["method"] == "turn/completed" && &v["params"]["threadId"] == thread);
        assert_eq!(done["params"]["turn"]["status"], "completed", "{done}");
    }
}

#[test]
#[ignore = "Runs real GPT 5.6 Luna inference with existing Codex authentication"]
fn real_luna_reads_unsolicited_preview_then_suppresses_and_resurfaces_after_compaction() {
    use std::io::Write;
    let run = Run::new();
    println!("Real Codex/Luna acceptance run: {}", run.root.display());
    let version = Command::new("codex").arg("--version").output().unwrap();
    assert_eq!(version.stdout, b"codex-cli 0.155.1\n");
    fs::write(run.root.join("logs/codex-version.txt"), version.stdout).unwrap();
    let daemon = Daemon::start(&run);
    let mut draft = draft(&run);
    draft.body="M3_FULL_RECORD_MARKER: the synthetic test suite took 12 seconds. This body is available only through a full record read.".into();
    let (_, created) = ctl(
        &run,
        &daemon.socket,
        &run.scope,
        &["create", "--json", "-"],
        Some(&draft),
    );
    let record: Record = serde_json::from_value(created["result"]["Ok"].clone()).unwrap();
    let codex_home = run.root.join("codex");
    fs::create_dir(&codex_home).unwrap();
    fs::create_dir(run.root.join("tmp")).unwrap();
    let hook_config = HookConfig {
        profile: Default::default(),
        instructions: None,
        runtime: run.root.clone(),
        scope_root: run.scope.clone(),
        tenant: run.policy.tenant.clone(),
        data_socket: daemon.socket.clone(),
        codex_socket: run.root.join("codex.sock"),
        attempts: run.root.join("harness/attempts.sqlite"),
        bounds: bounds(),
    };
    let config_path = run.root.join("hook-config.json");
    fs::write(
        &config_path,
        serde_json::to_vec_pretty(&hook_config).unwrap(),
    )
    .unwrap();
    let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
    let command = format!(
        "{} --config {}",
        quote(env!("CARGO_BIN_EXE_exo-codex-hook")),
        quote(config_path.to_str().unwrap())
    );
    let mut hooks = serde_json::Map::new();
    for name in [
        "SessionStart",
        "SessionEnd",
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "PermissionRequest",
        "SubagentStart",
        "SubagentStop",
        "PreCompact",
        "PostCompact",
        "Stop",
    ] {
        hooks.insert(name.into(),json!([{"hooks":[{"type":"command","command":command,"timeout":if name=="SessionEnd" {3} else {10}}]}]));
    }
    fs::write(
        codex_home.join("hooks.json"),
        serde_json::to_vec_pretty(&json!({"hooks":hooks})).unwrap(),
    )
    .unwrap();
    fs::write(
        codex_home.join("config.toml"),
        "model = \"gpt-5.6-luna\"\n[features]\nhooks = true\nshell_snapshot = false\n",
    )
    .unwrap();
    let auth = codex_home.join("auth.json");
    let _auth_cleanup = PrivateAuth(auth.clone());
    let source_home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".codex"));
    // Existing credentials are copied privately for this child and removed on completion.
    let mut auth_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&auth)
        .unwrap();
    auth_file
        .write_all(
            &fs::read(source_home.join("auth.json"))
                .expect("existing Codex authentication required"),
        )
        .unwrap();
    drop(auth_file);
    let mut child = HostProcess(
        Command::new("codex")
            .args([
                "app-server",
                "--listen",
                &format!("unix://{}", hook_config.codex_socket.display()),
            ])
            .env("CODEX_HOME", &codex_home)
            .env("TMPDIR", run.root.join("tmp"))
            .current_dir(&run.scope)
            .stdin(Stdio::null())
            .stdout(Stdio::from(
                fs::File::create(run.root.join("logs/codex.stdout")).unwrap(),
            ))
            .stderr(Stdio::from(
                fs::File::create(run.root.join("logs/codex.stderr")).unwrap(),
            ))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    let stream = loop {
        if let Ok(stream) = UnixStream::connect(&hook_config.codex_socket) {
            break stream;
        }
        if child.0.try_wait().unwrap().is_some() || Instant::now() > deadline {
            let _ = child.0.kill();
            let _ = child.0.wait();
            let _ = fs::remove_file(&auth);
            panic!("Codex startup failed");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    let (socket, _) = tungstenite::client("ws://localhost/", stream).unwrap();
    let mut host = Host {
        _child: child,
        socket,
        transcript: fs::File::create(run.root.join("logs/codex-rpc.jsonl")).unwrap(),
        observed: Vec::new(),
    };
    host.rpc(1,"initialize",json!({"clientInfo":{"name":"exomemory_live_test","version":"0.1.0"},"capabilities":{"experimentalApi":true}}));
    host.send(json!({"method":"initialized","params":{}}));
    let ctl_base = format!(
        "{} --runtime {} --scope-root {} --socket {} --tenant {} --session live --cwd {}",
        quote(env!("CARGO_BIN_EXE_exoctl")),
        quote(run.root.to_str().unwrap()),
        quote(run.scope.to_str().unwrap()),
        quote(daemon.socket.to_str().unwrap()),
        quote(&run.policy.tenant),
        quote(run.scope.to_str().unwrap())
    );
    let instructions = format!(
        "This is an isolated memory integration test. Stay in the supplied test directory. If an optional memory preview is supplied, retrieve its full record using exec_command with: {ctl_base} GHOST_KEY. Do not inspect databases, hooks, or parent directories. Quote the full-record marker in your answer after reading. Keep answers short."
    );
    let thread=host.rpc(2,"thread/start",json!({"cwd":run.scope,"model":"gpt-5.6-luna","config":{"bypass_hook_trust":true},"developerInstructions":instructions,"approvalPolicy":"never","sandbox":"danger-full-access"}));
    assert_eq!(thread["model"], "gpt-5.6-luna");
    let id = thread["thread"]["id"].clone();
    host.turn(3, &id, "Plan the test suite work.");
    let mut context = run.context.clone();
    context.session = id.as_str().unwrap().into();
    let mut store = AttemptStore::open(
        &run.policy,
        &hook_config.attempts,
        hook_config.bounds.clone(),
    )
    .unwrap();
    let first = store.state(&context, &context.session).unwrap();
    assert!(first.attempted.contains(&record.metadata.ghost_key));
    let read_seen = host.observed.iter().any(|v| {
        v["method"] == "item/completed"
            && v["params"]["item"]["type"] == "commandExecution"
            && v["params"]["item"]["aggregatedOutput"]
                .as_str()
                .is_some_and(|s| {
                    s.contains("M3_FULL_RECORD_MARKER") && s.contains(&record.metadata.ghost_key)
                })
    });
    assert!(
        read_seen,
        "Luna did not return the full record through tool output"
    );
    host.turn(
        4,
        &id,
        "Continue planning the test suite; no further tools are needed.",
    );
    let second = store.state(&context, &context.session).unwrap();
    assert_eq!(first.last_attempted, second.last_attempted);
    host.rpc(5, "thread/compact/start", json!({"threadId":id}));
    host.until(|v| {
        v["method"] == "hook/completed" && v["params"]["run"]["eventName"] == "postCompact"
    });
    assert!(
        store
            .state(&context, &context.session)
            .unwrap()
            .attempted
            .is_empty()
    );
    host.until(|v| v["method"] == "turn/completed");
    host.turn(
        6,
        &id,
        "Plan the test suite again using any newly surfaced preview.",
    );
    let after = store.state(&context, &context.session).unwrap();
    assert!(
        after.last_attempted[&record.metadata.ghost_key]
            > first.last_attempted[&record.metadata.ghost_key]
    );
    host.rpc(7, "thread/archive", json!({"threadId":id}));
    for value in &host.observed {
        if value["method"] == "hook/completed" {
            assert_eq!(value["params"]["run"]["status"], "completed", "{value}");
        }
    }
    fs::write(run.root.join("harness/codex-evidence.json"),serde_json::to_vec_pretty(&json!({"codex":"0.155.1","model":"gpt-5.6-luna","real_inference":true,"full_read_tool_output":true,"before":first,"suppressed":second,"after_compaction":after})).unwrap()).unwrap();
}

#[test]
#[ignore = "Runs installed-hook trust and real GPT 5.6 Luna inference in an isolated home"]
fn installed_hooks_are_trusted_without_bypass_and_luna_uses_the_installed_cli() {
    use std::io::Write;
    let run = Run::new();
    let home = &run.scope;
    println!("Installed Luna acceptance run: {}", run.root.display());
    let native = std::env::var("EXOMEMORY_NATIVE_CODEX")
        .unwrap_or_else(|_| "/opt/homebrew/bin/codex".into());
    let output = Command::new(env!("CARGO_BIN_EXE_exo-install"))
        .args([
            "install",
            "--home",
            home.to_str().unwrap(),
            "--codex",
            &native,
            "--no-start",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let runtime = home.join(".local/share/exomemory");
    let codex_home = home.join(".codex");
    fs::write(
        codex_home.join("config.toml"),
        "model = 'gpt-5.6-luna'\n[features]\nhooks = true\nshell_snapshot = false\n",
    )
    .unwrap();
    let auth = codex_home.join("auth.json");
    let auth_cleanup = PrivateAuth(auth.clone());
    let source = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".codex"));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&auth)
        .unwrap()
        .write_all(&fs::read(source.join("auth.json")).unwrap())
        .unwrap();
    let daemon = HostProcess(
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
                runtime.join("exo.sock").to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::from(
                fs::File::create(run.root.join("logs/installed-daemon.log")).unwrap(),
            ))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(runtime.join("exo.sock")).is_err() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut input = draft(&run);
    input.body = "INSTALL_FULL_RECORD_MARKER".into();
    input.blurb = "A stored measurement for the installation trial".into();
    input.triggers = vec![Trigger::Regex {
        id: "install-trial".into(),
        pattern: "(?i)trialprobe".into(),
    }];
    let mut create = Command::new(home.join(".local/bin/exoctl"))
        .args(["create", "--json", "-"])
        .env("HOME", home)
        .current_dir(home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
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
        "{}",
        String::from_utf8_lossy(&created.stdout)
    );
    let record: Value = serde_json::from_slice(&created.stdout).unwrap();
    let key = record["result"]["Ok"]["metadata"]["ghost_key"]
        .as_str()
        .unwrap();
    let path = format!(
        "{}:{}",
        home.join(".local/bin").display(),
        std::env::var("PATH").unwrap()
    );
    let child = HostProcess(
        Command::new(&native)
            .args([
                "app-server",
                "--listen",
                &format!("unix://{}", runtime.join("codex.sock").display()),
            ])
            .env("CODEX_HOME", &codex_home)
            .env("HOME", home)
            .env("PATH", &path)
            .env("EXOMEMORY_HOOK_CONFIG", runtime.join("hook-config.json"))
            .env("TMPDIR", &run.root)
            .current_dir(home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(
                fs::File::create(run.root.join("logs/installed-codex.log")).unwrap(),
            ))
            .spawn()
            .unwrap(),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_exo-install"))
        .args(["trust", "--home", home.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stream = UnixStream::connect(runtime.join("codex.sock")).unwrap();
    let (socket, _) = tungstenite::client("ws://localhost/", stream).unwrap();
    let mut host = Host {
        _child: child,
        socket,
        transcript: fs::File::create(run.root.join("logs/installed-rpc.jsonl")).unwrap(),
        observed: Vec::new(),
    };
    host.rpc(1,"initialize",json!({"clientInfo":{"name":"installed_trial_test","version":"0.1"},"capabilities":{"experimentalApi":true}}));
    host.send(json!({"method":"initialized","params":{}}));
    let listed = host.rpc(2, "hooks/list", json!({"cwds":[home]}));
    let text = listed.to_string();
    assert!(
        !text.contains("\"untrusted\"") && !text.contains("\"modified\""),
        "{listed}"
    );
    let thread=host.rpc(3,"thread/start",json!({"cwd":home,"model":"gpt-5.6-luna","approvalPolicy":"never","sandbox":"danger-full-access","developerInstructions":"This is an isolated installation test. Stay inside the current directory. Do not inspect configuration, databases, or parent directories. Retrieve useful preview data using the installed exoctl command and report the full body marker. Keep the answer short."}));
    let id = thread["thread"]["id"].clone();
    host.turn(
        4,
        &id,
        "Use any memory for trialprobe to tell me the saved measurement.",
    );
    assert!(
        host.observed.iter().any(|v| v["method"] == "item/completed"
            && v["params"]["item"]["type"] == "commandExecution"
            && v["params"]["item"]["aggregatedOutput"]
                .as_str()
                .is_some_and(|s| s.contains("INSTALL_FULL_RECORD_MARKER") && s.contains(key))),
        "installed CLI did not return full record"
    );
    let settings: HookConfig =
        serde_json::from_slice(&fs::read(runtime.join("hook-config.json")).unwrap()).unwrap();
    let policy = settings
        .profile
        .policy(
            &settings.runtime,
            &settings.scope_root,
            settings.tenant.clone(),
        )
        .unwrap();
    let context = Context {
        tenant: settings.tenant.clone(),
        session: id.as_str().unwrap().into(),
        cwd: home.to_path_buf(),
    };
    let mut attempts =
        AttemptStore::open(&policy, &settings.attempts, settings.bounds.clone()).unwrap();
    let first = attempts.state(&context, &context.session).unwrap();
    assert!(first.attempted.contains(key));
    host.turn(
        5,
        &id,
        "Continue considering trialprobe, but do not run any tools.",
    );
    let suppressed = attempts.state(&context, &context.session).unwrap();
    assert_eq!(first.last_attempted, suppressed.last_attempted);
    host.rpc(6, "thread/compact/start", json!({"threadId":id}));
    host.until(|value| {
        value["method"] == "hook/completed" && value["params"]["run"]["eventName"] == "postCompact"
    });
    assert!(
        attempts
            .state(&context, &context.session)
            .unwrap()
            .attempted
            .is_empty()
    );
    host.until(|value| value["method"] == "turn/completed");
    host.turn(
        7,
        &id,
        "Use any newly surfaced memory for trialprobe; the full record was already read.",
    );
    let after_compaction = attempts.state(&context, &context.session).unwrap();
    assert!(after_compaction.last_attempted[key] > first.last_attempted[key]);

    let metadata = Command::new(home.join(".local/bin/exoctl"))
        .args(["meta", key])
        .env("HOME", home)
        .current_dir(home)
        .output()
        .unwrap();
    assert!(metadata.status.success());
    let metadata: Value = serde_json::from_slice(&metadata.stdout).unwrap();
    assert_eq!(
        metadata["result"]["Ok"]["provenance"],
        serde_json::to_value(&input.provenance).unwrap()
    );
    assert_eq!(
        metadata["result"]["Ok"]["triggers"],
        serde_json::to_value(&input.triggers).unwrap()
    );

    host.rpc(8, "thread/archive", json!({"threadId":id}));
    for value in &host.observed {
        if value["method"] == "hook/completed" {
            assert_eq!(value["params"]["run"]["status"], "completed", "{value}");
        }
    }
    let hook_events: Vec<_> = host
        .observed
        .iter()
        .filter(|value| value["method"] == "hook/completed")
        .filter_map(|value| {
            value["params"]["run"]["eventName"]
                .as_str()
                .map(str::to_owned)
        })
        .collect();
    let shell = Command::new("/bin/zsh")
        .args(["-lic", "command -v codex"])
        .env("HOME", home)
        .env("ZDOTDIR", home)
        .env("PATH", &path)
        .current_dir(home)
        .output()
        .unwrap();
    assert!(shell.status.success());
    assert_eq!(
        String::from_utf8_lossy(&shell.stdout).trim(),
        home.join(".local/bin/codex").to_str().unwrap()
    );
    let launcher = fs::read(home.join(".local/bin/codex")).unwrap();
    let client = fs::read(home.join(".local/bin/exoctl")).unwrap();
    assert!(!launcher.starts_with(b"#!") && !client.starts_with(b"#!"));
    let help = Command::new(home.join(".local/bin/exoctl"))
        .arg("--help")
        .env("HOME", home)
        .output()
        .unwrap();
    assert!(help.status.success());
    drop(host);
    drop(daemon);
    drop(auth_cleanup);
    assert!(!auth.exists());
    fs::write(run.root.join("harness/installed-evidence.json"),serde_json::to_vec_pretty(&json!({
        "codex":"0.155.1",
        "model":"gpt-5.6-luna",
        "real_inference":true,
        "run_directory":run.root.clone(),
        "compiled":{"codex":{"path":home.join(".local/bin/codex"),"fingerprint":fingerprint(&launcher)},"exoctl":{"path":home.join(".local/bin/exoctl"),"fingerprint":fingerprint(&client)}},
        "help_fingerprint":fingerprint(&help.stdout),
        "fresh_login_resolution":home.join(".local/bin/codex"),
        "trusted_without_bypass":true,
        "installed_cli_full_read":true,
        "metadata":metadata["result"]["Ok"],
        "attempts":{"first":first,"suppressed":suppressed,"after_compaction":after_compaction},
        "hook_events":hook_events,
        "hook_definitions":listed,
        "auth_removed":true
    })).unwrap()).unwrap();
}
