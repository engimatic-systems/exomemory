use crate::{data::Storage, policy::Policy, sqlite::Sqlite, types::*};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    os::unix::{
        fs::{FileTypeExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
pub const MAX_WIRE_BYTES: u64 = 2 * 1024 * 1024;
pub fn parse_options(
    args: &[String],
    allowed: &[&str],
) -> Result<(BTreeMap<String, String>, Vec<String>)> {
    let mut options = BTreeMap::new();
    let mut i = 0;
    while i < args.len() && args[i].starts_with("--") {
        if !allowed.contains(&args[i].as_str())
            || i + 1 == args.len()
            || options
                .insert(args[i].clone(), args[i + 1].clone())
                .is_some()
        {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "unknown, duplicate or incomplete option",
            ));
        }
        i += 2;
    }
    Ok((options, args[i..].to_vec()))
}
pub fn required(options: &BTreeMap<String, String>, name: &str) -> Result<String> {
    options
        .get(name)
        .cloned()
        .ok_or_else(|| Error::new(ErrorCode::InvalidInput, format!("required {name}")))
}
pub fn configured_policy(options: &BTreeMap<String, String>) -> Result<Policy> {
    Policy::test_profile(
        &PathBuf::from(required(options, "--runtime")?),
        &PathBuf::from(required(options, "--scope-root")?),
        required(options, "--tenant")?,
    )
}
pub fn response(result: Result<serde_json::Value>) -> Response {
    Response {
        version: PROTOCOL_VERSION,
        result,
    }
}
pub fn frame(reader: impl Read) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    BufReader::new(reader.take(MAX_WIRE_BYTES + 1)).read_until(b'\n', &mut bytes)?;
    if bytes.len() as u64 > MAX_WIRE_BYTES || bytes.last() != Some(&b'\n') {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "oversize or incomplete JSON line",
        ));
    }
    Ok(bytes)
}
pub fn validate_trigger_kinds(value: &serde_json::Value) -> Result<()> {
    if let Some(triggers) = value.get("triggers").and_then(|v| v.as_array()) {
        for trigger in triggers {
            if let Some(kind) = trigger.get("kind").and_then(|v| v.as_str())
                && kind != "regex"
            {
                return Err(Error::new(
                    ErrorCode::Unsupported,
                    "unsupported trigger kind",
                ));
            }
        }
    }
    Ok(())
}
pub fn call(
    policy: &Policy,
    socket: &std::path::Path,
    request: &Request,
) -> Result<serde_json::Value> {
    policy.context(&request.context)?;
    let socket = policy.runtime_path(socket)?;
    let mut stream = UnixStream::connect(socket)
        .map_err(|e| Error::new(ErrorCode::Unavailable, e.to_string()))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut bytes = serde_json::to_vec(request)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > MAX_WIRE_BYTES {
        return Err(Error::new(ErrorCode::InvalidInput, "request too large"));
    }
    stream.write_all(&bytes).map_err(|e| {
        Error::new(
            ErrorCode::Unavailable,
            format!("request delivery uncertain: {e}"),
        )
    })?;
    let reply = frame(&stream).map_err(|e| {
        Error::new(
            ErrorCode::Unavailable,
            format!("response unavailable; a write may have committed: {e}"),
        )
    })?;
    let reply: Response = serde_json::from_slice(&reply)?;
    if reply.version != PROTOCOL_VERSION {
        return Err(Error::new(
            ErrorCode::Unsupported,
            "response protocol version mismatch",
        ));
    }
    reply.result
}
fn dispatch(db: &mut Sqlite, policy: &Policy, request: Request) -> Result<serde_json::Value> {
    if request.version != PROTOCOL_VERSION {
        return Err(Error::new(
            ErrorCode::Unsupported,
            "request protocol version mismatch",
        ));
    }
    policy.context(&request.context)?;
    match request.operation {
        Operation::ListTriggerTypes => Ok(
            serde_json::json!({"types":[{"kind":"regex","parameters":{"id":"nonempty stable source identifier","pattern":"Rust regex syntax, Unicode, case-sensitive unless inline flags"},"normalization":"CRLF and CR become LF; no case folding or Unicode normalization","backend":{"kind":"sqlite","instance":"local"}}]}),
        ),
        Operation::Create { draft } => Ok(serde_json::to_value(db.create(
            policy,
            &request.context,
            draft,
        )?)?),
        Operation::Overwrite {
            key,
            if_revision,
            draft,
        } => Ok(serde_json::to_value(db.overwrite(
            policy,
            &request.context,
            &key,
            if_revision,
            draft,
        )?)?),
        Operation::Read { key } => Ok(serde_json::to_value(db.read(
            policy,
            &request.context,
            &key,
        )?)?),
        Operation::Meta { key } => Ok(serde_json::to_value(
            db.read(policy, &request.context, &key)?.metadata,
        )?),
        Operation::Candidates { event } => {
            if event.context != request.context {
                return Err(Error::new(
                    ErrorCode::Forbidden,
                    "event context differs from request context",
                ));
            }
            Ok(serde_json::to_value(db.candidates(policy, &event)?)?)
        }
    }
}
pub fn serve(policy: Policy, database: PathBuf, socket: PathBuf) -> Result<()> {
    let database = policy.runtime_path(&database)?;
    let socket = policy.runtime_path(&socket)?;
    if database == socket {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "database and socket paths must differ",
        ));
    }
    let db = Arc::new(Mutex::new(Sqlite::open(&policy, &database)?));
    if let Ok(metadata) = fs::symlink_metadata(&socket) {
        if !metadata.file_type().is_socket() {
            return Err(Error::new(
                ErrorCode::Unavailable,
                "endpoint exists and is not a socket",
            ));
        }
        match UnixStream::connect(&socket) {
            Ok(_) => {
                return Err(Error::new(
                    ErrorCode::Unavailable,
                    "endpoint already serves connections",
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                fs::remove_file(&socket)?
            }
            Err(e) => return Err(Error::new(ErrorCode::Unavailable, e.to_string())),
        }
    }
    let listener = UnixListener::bind(&socket)?;
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let db = db.clone();
        let policy = policy.clone();
        std::thread::spawn(move || {
            let result = (|| -> Result<serde_json::Value> {
                stream.set_read_timeout(Some(Duration::from_secs(5)))?;
                stream.set_write_timeout(Some(Duration::from_secs(5)))?;
                let value: serde_json::Value = serde_json::from_slice(&frame(&stream)?)?;
                if let Some(draft) = value.pointer("/operation/draft") {
                    validate_trigger_kinds(draft)?;
                }
                let request: Request = serde_json::from_value(value)?;
                let mut db = db
                    .lock()
                    .map_err(|_| Error::new(ErrorCode::Internal, "database worker poisoned"))?;
                dispatch(&mut db, &policy, request)
            })();
            if let Ok(mut bytes) = serde_json::to_vec(&response(result)) {
                bytes.push(b'\n');
                let _ = stream.write_all(&bytes);
            }
        });
    }
    Ok(())
}
