use crate::{
    attempts::AttemptStore,
    policy::{Policy, Profile},
    render::{Bounds, WRAPPER},
    transport,
    types::*,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions, TryLockError},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HookConfig {
    #[serde(default)]
    pub profile: Profile,
    #[serde(default)]
    pub instructions: Option<String>,
    pub runtime: PathBuf,
    pub scope_root: PathBuf,
    pub tenant: String,
    pub data_socket: PathBuf,
    pub codex_socket: PathBuf,
    pub attempts: PathBuf,
    pub bounds: Bounds,
}

fn unavailable(error: impl std::fmt::Display) -> Error {
    Error::new(ErrorCode::Unavailable, error.to_string())
}

/// A local Unix WebSocket connection to the explicitly configured Codex host.
pub struct CodexClient {
    socket: WebSocket<UnixStream>,
    next_id: u64,
}
impl CodexClient {
    pub fn connect(path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(path).map_err(unavailable)?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        let (socket, _) = tungstenite::client("ws://localhost/", stream).map_err(unavailable)?;
        let mut client = Self { socket, next_id: 1 };
        let response = client.rpc("initialize", json!({"clientInfo":{"name":"exomemory_hook","version":"0.1.0"},"capabilities":{"experimentalApi":true}}))?;
        if !response["userAgent"]
            .as_str()
            .is_some_and(|s| s.contains("/0.155.1 "))
        {
            return Err(Error::new(
                ErrorCode::Unsupported,
                "Codex adapter requires verified 0.155.1 host",
            ));
        }
        client.send(json!({"method":"initialized","params":{}}))?;
        Ok(client)
    }
    fn send(&mut self, value: Value) -> Result<()> {
        self.socket
            .send(Message::Text(value.to_string().into()))
            .map_err(unavailable)
    }
    pub fn rpc(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({"id":id,"method":method,"params":params}))?;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            self.socket.get_mut().set_read_timeout(Some(
                deadline
                    .saturating_duration_since(Instant::now())
                    .max(Duration::from_millis(1)),
            ))?;
            if Instant::now() >= deadline {
                return Err(unavailable("Codex RPC timeout; delivery may have occurred"));
            }
            let message = self.socket.read().map_err(unavailable)?;
            if let Message::Text(text) = message {
                let value: Value = serde_json::from_str(&text)?;
                if value["id"] == id {
                    if let Some(error) = value.get("error") {
                        return Err(unavailable(error));
                    }
                    return Ok(value["result"].clone());
                }
            }
        }
    }
}

pub fn normalize(policy: &Policy, input: &Value, number: u64) -> Result<Event> {
    let field = |name: &str| {
        input[name]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::new(ErrorCode::InvalidInput, format!("missing hook {name}")))
    };
    let kind = match field("hook_event_name")? {
        "UserPromptSubmit" => EventKind::Prompt,
        "PreToolUse" => EventKind::ToolCall,
        "PostToolUse" => EventKind::ToolResponse,
        "PermissionRequest" => EventKind::Permission,
        "SessionStart" => EventKind::SessionStart,
        "SessionEnd" => EventKind::SessionEnd,
        "SubagentStart" => EventKind::SubagentStart,
        "SubagentStop" => EventKind::SubagentEnd,
        "PreCompact" => EventKind::CompactionStarted,
        "PostCompact" => EventKind::CompactionCompleted,
        "Stop" => EventKind::TurnComplete,
        _ => {
            return Err(Error::new(
                ErrorCode::Unsupported,
                "unsupported Codex hook event",
            ));
        }
    };
    let session = field("session_id")?.to_owned();
    let context_id = match input.get("agent_id") {
        None | Some(Value::Null) => session.clone(),
        Some(Value::String(id)) if !id.is_empty() => id.clone(),
        _ => {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "invalid child context id",
            ));
        }
    };
    let context = Context {
        tenant: policy.tenant.clone(),
        session,
        cwd: field("cwd")?.into(),
    };
    policy.context(&context)?;
    // Only event content is scanned; transport paths and identifiers are not trigger text.
    let text = [
        "prompt",
        "tool_name",
        "tool_input",
        "tool_response",
        "last_assistant_message",
        "source",
        "trigger",
        "reason",
        "agent_type",
    ]
    .iter()
    .filter_map(|key| input.get(*key))
    .map(|value| {
        value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| value.to_string())
    })
    .collect::<Vec<_>>()
    .join("\n");
    Ok(Event {
        context,
        context_id,
        number,
        kind,
        text,
    })
}

fn lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(e) => return Err(unavailable(e)),
        }
    }
}

/// Each synchronous hook runs this once; uncertain delivery keeps the committed mark.
pub fn handle(config: &HookConfig, input: &Value) -> Result<usize> {
    let policy =
        config
            .profile
            .policy(&config.runtime, &config.scope_root, config.tenant.clone())?;
    let attempts = policy.runtime_path(&config.attempts)?;
    let data_socket = policy.runtime_path(&config.data_socket)?;
    let codex_socket = policy.runtime_path(&config.codex_socket)?;
    let lock_path = policy.runtime_path(&PathBuf::from(format!("{}.lock", attempts.display())))?;
    if [&attempts, &data_socket, &codex_socket, &lock_path]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != 4
    {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "adapter runtime paths must differ",
        ));
    }
    let mut event = normalize(&policy, input, 0)?;
    // Reserve framing and escaped wrapper bytes even for the largest RPC request ID.
    let overhead = serde_json::to_vec(&json!({"id":u64::MAX,"method":"thread/inject_items","params":injection(&event.context_id,WRAPPER)}))?.len();
    if overhead > config.bounds.max_wrapper_bytes {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "Codex wrapper and framing exceed configured bound",
        ));
    }
    // The cross-process lock orders marking, one send, and completion reset together.
    let _lock = lock(&lock_path)?;
    let mut store = AttemptStore::open(&policy, &attempts, config.bounds.clone())?;
    event.number = store
        .state(&event.context, &event.context_id)?
        .last_event
        .checked_add(1)
        .ok_or_else(|| Error::new(ErrorCode::Conflict, "session event counter exhausted"))?;
    let candidates = if event.kind == EventKind::CompactionCompleted {
        CandidateSnapshot {
            state_version: 0,
            candidates: Vec::new(),
        }
    } else {
        serde_json::from_value(transport::call(
            &policy,
            &data_socket,
            &Request {
                version: PROTOCOL_VERSION,
                context: event.context.clone(),
                operation: Operation::Candidates {
                    event: event.clone(),
                },
            },
        )?)?
    };
    let ghosts = store.process(&event, &candidates)?;
    if ghosts.is_empty() {
        return Ok(0);
    }
    let mut text = WRAPPER.to_owned();
    for ghost in &ghosts {
        text.push_str(ghost.serialized());
    }
    let mut client = CodexClient::connect(&codex_socket)?;
    client.rpc("thread/inject_items", injection(&event.context_id, &text))?;
    Ok(ghosts.len())
}

fn injection(context_id: &str, text: &str) -> Value {
    json!({"threadId":context_id,"items":[{"type":"message","role":"developer","content":[{"type":"input_text","text":text}]}]})
}
