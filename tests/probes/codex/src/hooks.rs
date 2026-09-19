use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn hook(run: &Path) -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let input: Value = serde_json::from_str(&input)?;
    let name = input["hook_event_name"]
        .as_str()
        .ok_or("hook name missing")?;
    let file = format!(
        "{}-{}-{}.json",
        name,
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    fs::write(
        run.join("hooks").join(file),
        serde_json::to_vec_pretty(&input)?,
    )?;
    if name == "UserPromptSubmit" && input["prompt"] == "tool-phase" {
        fs::write(run.join("prompt-entered"), "entered")?;
        let deadline = Instant::now() + Duration::from_secs(3);
        while !run.join("release-prompt").exists() {
            if Instant::now() > deadline {
                return Err("prompt gate timeout".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    if name == "PreCompact" && run.join("abort-compaction").exists() {
        println!(
            "{}",
            json!({"continue":false,"stopReason":"synthetic aborted compaction"})
        );
        return Ok(());
    }
    // A deterministic marker proves hook output reached the actual model request.
    if [
        "SessionStart",
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "SubagentStart",
    ]
    .contains(&name)
    {
        println!(
            "{}",
            json!({"hookSpecificOutput":{"hookEventName":name,"additionalContext":format!("EXOMEMORY_HOOK_{name}")}})
        );
    } else if name == "PermissionRequest" {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !run.join("release-permission").exists() {
            if Instant::now() > deadline {
                return Err("permission insertion gate timeout".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        println!(
            "{}",
            json!({"hookSpecificOutput":{"hookEventName":name,"decision":{"behavior":"allow"}}})
        );
    } else {
        println!("{{}}");
    }
    Ok(())
}

struct Host {
    child: Child,
    input: ChildStdin,
    output: mpsc::Receiver<String>,
    log: fs::File,
    run: PathBuf,
}
impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Host {
    fn send(&mut self, value: Value) -> Result<()> {
        writeln!(self.log, "{}", json!({"send":value}))?;
        writeln!(self.input, "{value}")?;
        self.input.flush()?;
        Ok(())
    }
    fn until(&mut self, predicate: impl Fn(&Value) -> bool) -> Result<Value> {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let line = self
                .output
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))?;
            let value: Value = serde_json::from_str(&line)?;
            writeln!(self.log, "{}", json!({"receive":value}))?;
            if value["method"] == "hook/started"
                && value["params"]["run"]["eventName"] == "permissionRequest"
            {
                // Insert while the synchronous hook is waiting, before approving the tool.
                self.rpc(1000, "thread/inject_items", json!({"threadId":value["params"]["threadId"],"items":[{"type":"message","role":"developer","content":[{"type":"input_text","text":"EXOMEMORY_PERMISSION_INJECTION"}]}]}))?;
                fs::write(self.run.join("release-permission"), "released")?;
            }
            if predicate(&value) {
                return Ok(value);
            }
        }
    }
    fn rpc(&mut self, id: u64, method: &str, params: Value) -> Result<Value> {
        self.send(json!({"id":id,"method":method,"params":params}))?;
        let value = self.until(|v| v["id"] == id)?;
        if value.get("error").is_some() {
            return Err(value.to_string().into());
        }
        Ok(value["result"].clone())
    }
}

fn request(stream: &mut TcpStream) -> Result<(String, Value)> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse()?;
        }
        if line.to_ascii_lowercase().starts_with("content-encoding:") {
            return Err(format!("unexpected compression: {line}").into());
        }
    }
    if length > 4_000_000 {
        return Err("request too large".into());
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok((first, serde_json::from_slice(&body)?))
}

fn respond(stream: &mut TcpStream, path: &str, body: &Value, number: usize) -> Result<()> {
    let text = body.to_string();
    let tool_done = text.contains("function_call_output");
    let item = if text.contains("permission-phase") && !tool_done {
        json!({"type":"function_call","id":format!("fc_{number}"),"call_id":format!("call_{number}"),"name":"exec_command","arguments":json!({"cmd":"printf permission-probe","sandbox_permissions":"require_escalated","justification":"Exercise the isolated permission hook","max_output_tokens":100}).to_string()})
    } else if text.contains("child-phase") && !tool_done {
        json!({"type":"function_call","id":format!("fc_{number}"),"call_id":format!("call_{number}"),"namespace":"multi_agent_v1","name":"spawn_agent","arguments":json!({"message":"synthetic-child: return the scripted fixture","fork_context":false}).to_string()})
    } else if text.contains("tool-phase") && !tool_done && !path.contains("compact") {
        json!({"type":"function_call","id":format!("fc_{number}"),"call_id":format!("call_{number}"),"name":"exec_command","arguments":"{\"cmd\":\"printf exomemory-probe\",\"max_output_tokens\":100}"})
    } else {
        json!({"type":"message","id":format!("msg_{number}"),"role":"assistant","status":"completed","content":[{"type":"output_text","text":"Synthetic probe complete.","annotations":[]}]})
    };
    let (kind, output) = if path.contains("/compact") {
        ("application/json", json!({"id":format!("cmp_{number}"),"object":"response.compaction","output":[item],"usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}).to_string())
    } else {
        let response = json!({"id":format!("resp_{number}"),"object":"response","status":"completed","output":[item.clone()],"usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}});
        let events = [
            json!({"type":"response.created","response":{"id":format!("resp_{number}"),"status":"in_progress"}}),
            json!({"type":"response.output_item.done","output_index":0,"item":item}),
            json!({"type":"response.completed","response":response}),
        ];
        (
            "text/event-stream",
            events
                .iter()
                .map(|v| format!("event: {}\ndata: {v}\n\n", v["type"].as_str().unwrap()))
                .collect::<String>(),
        )
    };
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{output}",
        output.len()
    )?;
    stream.flush()?;
    Ok(())
}

pub fn run() -> Result<()> {
    let repo = fs::canonicalize(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.."))?;
    let run = repo.join(format!(
        "tests/.runtime/hooks-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    ));
    for dir in ["scope", "codex", "hooks", "requests", "tmp"] {
        fs::create_dir_all(run.join(dir))?;
    }
    println!("Codex hook probe: {}", run.display());
    let version = Command::new("codex").arg("--version").output()?;
    fs::write(run.join("version.txt"), &version.stdout)?;
    if version.stdout != b"codex-cli 0.155.1\n" {
        return Err("probe requires Codex 0.155.1".into());
    }
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    listener.set_nonblocking(true)?;
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let server_run = run.clone();
    // Only the isolated Codex child uses this scripted local provider; no API key is loaded.
    let server = std::thread::spawn(move || -> std::result::Result<(), String> {
        let mut number = 0;
        while !stopped.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    number += 1;
                    let result = (|| -> Result<()> {
                        let (path, body) = request(&mut stream)?;
                        fs::write(
                            server_run.join("requests").join(format!("{number}.json")),
                            serde_json::to_vec_pretty(&json!({"path":path,"body":body}))?,
                        )?;
                        respond(&mut stream, &path, &body, number)
                    })();
                    if let Err(e) = result {
                        return Err(e.to_string());
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(())
    });
    let result = drive(&run, address);
    stop.store(true, Ordering::SeqCst);
    let server_result = server.join().map_err(|_| "provider thread panic")?;
    if let Err(e) = server_result {
        return Err(e.into());
    }
    result
}

fn drive(run: &Path, address: std::net::SocketAddr) -> Result<()> {
    let exe = std::env::current_exe()?;
    let quote = |p: &Path| format!("'{}'", p.display().to_string().replace('\'', "'\\''"));
    let command = format!("{} --hook {}", quote(&exe), quote(run));
    let names = [
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
    ];
    let mut hooks = serde_json::Map::new();
    for name in names {
        hooks.insert(
            name.into(),
            json!([{"hooks":[{"type":"command","command":command,"timeout":3}]}]),
        );
    }
    fs::write(
        run.join("codex/hooks.json"),
        serde_json::to_vec_pretty(&json!({"hooks":hooks}))?,
    )?;
    let config = format!(
        "model = \"probe\"\nmodel_provider = \"probe\"\n[model_providers.probe]\nname = \"local scripted probe\"\nbase_url = \"http://{address}/v1\"\nwire_api = \"responses\"\nrequest_max_retries = 0\nstream_max_retries = 0\n[features]\nhooks = true\nshell_snapshot = false\n"
    );
    fs::write(run.join("codex/config.toml"), config)?;
    let mut child = Command::new("codex")
        .args(["--dangerously-bypass-hook-trust", "app-server", "--stdio"])
        .env("CODEX_HOME", run.join("codex"))
        .env("TMPDIR", run.join("tmp"))
        .current_dir(run.join("scope"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(fs::File::create(run.join("stderr.log"))?)
        .spawn()?;
    let input = child.stdin.take().unwrap();
    let output = BufReader::new(child.stdout.take().unwrap());
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in output.lines().map_while(std::result::Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let mut host = Host {
        child,
        input,
        output: receiver,
        log: fs::File::create(run.join("rpc.jsonl"))?,
        run: run.to_owned(),
    };
    host.rpc(1,"initialize",json!({"clientInfo":{"name":"exomemory_hook_probe","version":"0.1.0"},"capabilities":{"experimentalApi":true}}))?;
    host.send(json!({"method":"initialized","params":{}}))?;
    let thread = host.rpc(2,"thread/start",json!({"cwd":run.join("scope"),"ephemeral":false,"config":{"bypass_hook_trust":true},"approvalPolicy":"never","sandbox":"danger-full-access"}))?;
    let id = thread["thread"]["id"].clone();
    host.rpc(
        3,
        "turn/start",
        json!({"threadId":id,"input":[{"type":"text","text":"tool-phase","text_elements":[]}]}),
    )?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while !run.join("prompt-entered").exists() {
        if Instant::now() > deadline {
            return Err("prompt hook not entered".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    // Hold the prompt hook and confirm the provider has received no request.
    std::thread::sleep(Duration::from_millis(100));
    if fs::read_dir(run.join("requests"))?.next().is_some() {
        return Err("model advanced through held prompt hook".into());
    }
    fs::write(run.join("release-prompt"), "released")?;
    let complete = host.until(|v| v["method"] == "turn/completed")?;
    if complete["params"]["turn"]["status"] != "completed" {
        return Err(complete.to_string().into());
    }
    fs::write(run.join("abort-compaction"), "abort")?;
    host.rpc(40, "thread/compact/start", json!({"threadId":id}))?;
    host.until(|v| v["method"] == "turn/completed")?;
    if fs::read_dir(run.join("hooks"))?.any(|f| {
        f.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("PostCompact-")
    }) {
        return Err("aborted compaction emitted completion hook".into());
    }
    fs::remove_file(run.join("abort-compaction"))?;
    host.rpc(4, "thread/compact/start", json!({"threadId":id}))?;
    host.until(|v| v["method"] == "turn/completed")?;
    host.rpc(5,"turn/start",json!({"threadId":id,"input":[{"type":"text","text":"after-compaction","text_elements":[]}]}))?;
    host.until(|v| v["method"] == "turn/completed")?;
    host.rpc(6, "thread/archive", json!({"threadId":id}))?;
    for (base, prompt, sandbox, approval) in [
        (10, "permission-phase", "read-only", "on-request"),
        (20, "child-phase", "danger-full-access", "never"),
    ] {
        let thread = host.rpc(base,"thread/start",json!({"cwd":run.join("scope"),"config":{"bypass_hook_trust":true},"approvalPolicy":approval,"sandbox":sandbox}))?;
        let thread_id = thread["thread"]["id"].clone();
        host.rpc(base+1,"turn/start",json!({"threadId":thread_id,"input":[{"type":"text","text":prompt,"text_elements":[]}]}))?;
        host.until(|v| v["method"] == "turn/completed" && v["params"]["threadId"] == thread_id)?;
        if prompt == "child-phase" {
            // A child completion is independent of the parent's turn completion.
            let deadline = Instant::now() + Duration::from_secs(5);
            while !fs::read_dir(run.join("hooks"))?.any(|f| {
                f.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("SubagentStop-")
            }) {
                if Instant::now() > deadline {
                    return Err("child stop hook not observed".into());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let child_hook = fs::read_dir(run.join("hooks"))?
                .find(|f| {
                    f.as_ref().is_ok_and(|f| {
                        f.file_name()
                            .to_string_lossy()
                            .starts_with("SubagentStart-")
                    })
                })
                .ok_or("child start missing")??;
            let child_input: Value = serde_json::from_slice(&fs::read(child_hook.path())?)?;
            let child_id = child_input["agent_id"].clone();
            host.rpc(
                base + 3,
                "thread/compact/start",
                json!({"threadId":child_id}),
            )?;
            host.until(|v| {
                v["method"] == "item/completed"
                    && v["params"]["threadId"] == child_id
                    && v["params"]["item"]["type"] == "contextCompaction"
            })?;
            // The compaction item can complete before its post-compaction hook runs.
            host.until(|v| {
                v["method"] == "hook/completed"
                    && v["params"]["threadId"] == child_id
                    && v["params"]["run"]["eventName"] == "postCompact"
            })?;
            let mut child_reset = false;
            for file in fs::read_dir(run.join("hooks"))? {
                let hook: Value = serde_json::from_slice(&fs::read(file?.path())?)?;
                child_reset |= hook["hook_event_name"] == "PostCompact"
                    && hook["agent_id"] == child_id
                    && hook["session_id"] == thread_id;
            }
            if !child_reset {
                return Err(
                    "child completion lacks parent session and child context identity".into(),
                );
            }
        }
        host.rpc(base + 2, "thread/archive", json!({"threadId":thread_id}))?;
    }
    let mut observed = Vec::<String>::new();
    for file in fs::read_dir(run.join("hooks"))? {
        let data: Value = serde_json::from_slice(&fs::read(file?.path())?)?;
        observed.push(data["hook_event_name"].as_str().unwrap().into());
    }
    observed.sort();
    observed.dedup();
    for required in names {
        if !observed.iter().any(|v| v == required) {
            return Err(format!("missing hook: {required}").into());
        }
    }
    let requests = fs::read_dir(run.join("requests"))?
        .map(|f| fs::read_to_string(f.unwrap().path()))
        .collect::<std::io::Result<Vec<_>>>()?
        .join("\n");
    for marker in [
        "EXOMEMORY_HOOK_SessionStart",
        "EXOMEMORY_HOOK_UserPromptSubmit",
        "EXOMEMORY_HOOK_PreToolUse",
        "EXOMEMORY_HOOK_PostToolUse",
        "EXOMEMORY_HOOK_SubagentStart",
        "EXOMEMORY_PERMISSION_INJECTION",
    ] {
        if !requests.contains(marker) {
            return Err(format!("marker absent from model input: {marker}").into());
        }
    }
    // Invocation alone is insufficient: the host must report successful synchronous handling.
    for line in fs::read_to_string(run.join("rpc.jsonl"))?.lines() {
        let value: Value = serde_json::from_str(line)?;
        let value = &value["receive"];
        if value["method"] == "hook/completed" {
            let status = &value["params"]["run"]["status"];
            if (status != "completed" && status != "stopped")
                || value["params"]["run"]["executionMode"] != "sync"
            {
                return Err(format!("hook failed or ran asynchronously: {value}").into());
            }
        }
    }
    fs::write(
        run.join("evidence.json"),
        serde_json::to_vec_pretty(
            &json!({"version":"0.155.1","observed_hooks":observed,"prompt_barrier_checked":true,"aborted_compaction_has_no_completion":true,"permission_injection_reaches_model":true,"child_compaction_checked":true,"provider":"local scripted HTTP; no real inference"}),
        )?,
    )?;
    println!("Observed hooks: {}", observed.join(", "));
    Ok(())
}
