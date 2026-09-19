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
