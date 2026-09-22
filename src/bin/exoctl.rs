use exomemory::{
    client::{ClientConfig, USER_CONFIG_RELATIVE},
    transport::*,
    types::*,
};
use std::{collections::BTreeMap, fs, io::Read, path::PathBuf};
const HELP: &str = r#"exoctl help
exoctl OPTIONS list-trigger-types
exoctl OPTIONS create --json FILE|-
exoctl OPTIONS overwrite GHOST_KEY --if-revision REVISION --json FILE|-
exoctl OPTIONS GHOST_KEY
exoctl OPTIONS meta GHOST_KEY
exoctl OPTIONS annoy GHOST_KEY

OPTIONS (all required, before command):
  --runtime DIR --scope-root DIR --socket PATH --tenant ID --session ID --cwd DIR
Paths must be absolute. --profile test (default) confines them to tests/.runtime/.
--profile local uses an explicit private runtime and permitted directory root.
With no connection options, exoctl reads EXOMEMORY_CONFIG or
$HOME/.config/exomemory/exoctl.json. Explicit options override file values.
Installed commands derive cwd from the process and use CODEX_THREAD_ID when set,
or the documented non-Codex session name "terminal". Controlled tests must pass
all explicit OPTIONS and never fall back to installed configuration.
Create and overwrite read a complete JSON draft from FILE or stdin (-):
  {"body":"...","blurb":"...","uses":[],"topics":[],
   "triggers":[{"kind":"regex","id":"stable-id","pattern":"test"}],
   "backend":{"kind":"sqlite","instance":"local"},
   "scope":{"kind":"dir","root":"/absolute/permitted/directory"},
   "provenance":{"origin":"source","actor":"author"},
   "trust":"untrusted","overwritable":true}
Non-empty validated triggers and an explicit backend are required. Global scope
is disabled in both profiles. Complete records are returned as tool data.

stdout: one JSON object {"version":1,"result":{"Ok":VALUE}} or
{"version":1,"result":{"Err":{"code":CODE,"message":TEXT}}}.
Create and reads return {metadata,body}; metadata includes actual revision,
tenant, scope, provenance, trust, triggers, backend, overwrite permission,
ranking penalty and read estimate. meta returns metadata only.
Annoy increments a penalty up to 4294967295 and returns penalty, feedback_version,
state_version and session attribution. It never changes matching or body revision.
Overwrite returns the complete replacement. A stale overwrite exits 7 and returns
Err.current metadata (including current revision); reconcile before trying again.
Exit statuses: 0 success; 2 invalid_input; 3 unavailable; 4 forbidden;
5 unsupported; 6 not_found; 7 conflict; 8 internal.
On response loss, writes may have committed: status 3 is not proof of rollback.
"#;
fn read_draft(path: &str) -> Result<Draft> {
    let input: Box<dyn Read> = if path == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(
            fs::File::open(path).map_err(|e| Error::new(ErrorCode::InvalidInput, e.to_string()))?,
        )
    };
    let mut bytes = Vec::new();
    input.take(MAX_WIRE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_WIRE_BYTES {
        return Err(Error::new(ErrorCode::InvalidInput, "draft too large"));
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    validate_trigger_kinds(&value)?;
    Ok(serde_json::from_value(value)?)
}

struct Resolved {
    policy: exomemory::policy::Policy,
    context: Context,
    socket: PathBuf,
}

fn config_error(path: &std::path::Path, error: impl std::fmt::Display) -> Error {
    Error::new(
        ErrorCode::InvalidInput,
        format!("invalid configuration {}: {error}", path.display()),
    )
}

fn load_config(path: &std::path::Path) -> Result<ClientConfig> {
    let bytes = fs::read(path).map_err(|e| config_error(path, e))?;
    serde_json::from_slice(&bytes).map_err(|e| config_error(path, e))
}

fn fill(options: &mut BTreeMap<String, String>, name: &str, value: impl ToString) {
    options
        .entry(name.into())
        .or_insert_with(|| value.to_string());
}

fn resolve(mut options: BTreeMap<String, String>) -> Result<Resolved> {
    const EXPLICIT: [&str; 6] = [
        "--runtime",
        "--scope-root",
        "--socket",
        "--tenant",
        "--session",
        "--cwd",
    ];
    let fully_explicit = EXPLICIT.iter().all(|name| options.contains_key(*name));
    if fully_explicit {
        eprintln!("exoctl: configuration source: explicit command options");
    } else {
        let (path, source) = if let Some(path) = std::env::var_os("EXOMEMORY_CONFIG") {
            (PathBuf::from(path), "EXOMEMORY_CONFIG")
        } else {
            let home = std::env::var_os("HOME").ok_or_else(|| {
                Error::new(
                    ErrorCode::InvalidInput,
                    "installed configuration unavailable: HOME is not set; pass all explicit options or set EXOMEMORY_CONFIG",
                )
            })?;
            (
                PathBuf::from(home).join(USER_CONFIG_RELATIVE),
                "installed user configuration",
            )
        };
        let config = load_config(&path)?;
        eprintln!("exoctl: configuration source: {source} {}", path.display());
        fill(
            &mut options,
            "--profile",
            match config.profile {
                exomemory::policy::Profile::Test => "test",
                exomemory::policy::Profile::Local => "local",
            },
        );
        fill(&mut options, "--runtime", config.runtime.display());
        fill(&mut options, "--scope-root", config.scope_root.display());
        fill(&mut options, "--socket", config.socket.display());
        fill(&mut options, "--tenant", config.tenant);
        if !options.contains_key("--session") {
            let session = std::env::var("CODEX_THREAD_ID")
                .ok()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "terminal".into());
            options.insert("--session".into(), session);
        }
        if !options.contains_key("--cwd") {
            options.insert(
                "--cwd".into(),
                std::env::current_dir()
                    .map_err(|e| Error::new(ErrorCode::InvalidInput, e.to_string()))?
                    .display()
                    .to_string(),
            );
        }
    }
    let policy = configured_policy(&options)?;
    let context = Context {
        tenant: required(&options, "--tenant")?,
        session: required(&options, "--session")?,
        cwd: PathBuf::from(required(&options, "--cwd")?),
    };
    let socket = PathBuf::from(required(&options, "--socket")?);
    Ok(Resolved {
        policy,
        context,
        socket,
    })
}

fn run(args: Vec<String>) -> Result<serde_json::Value> {
    let (options, command) = parse_options(
        &args,
        &[
            "--profile",
            "--runtime",
            "--scope-root",
            "--socket",
            "--tenant",
            "--session",
            "--cwd",
        ],
    )?;
    let resolved = resolve(options)?;
    let operation = match command.as_slice() {
        [op] if op == "list-trigger-types" => Operation::ListTriggerTypes,
        [op, flag, path] if op == "create" && flag == "--json" => Operation::Create {
            draft: read_draft(path)?,
        },
        [op, key, flag, revision, json, path]
            if op == "overwrite" && flag == "--if-revision" && json == "--json" =>
        {
            let if_revision = revision
                .parse()
                .map_err(|_| Error::new(ErrorCode::InvalidInput, "invalid revision"))?;
            Operation::Overwrite {
                key: key.clone(),
                if_revision,
                draft: read_draft(path)?,
            }
        }
        [op, key] if op == "annoy" => Operation::Annoy { key: key.clone() },
        [op, key] if op == "meta" => Operation::Meta { key: key.clone() },
        [key] if !["create", "meta", "overwrite", "annoy"].contains(&key.as_str()) => {
            Operation::Read { key: key.clone() }
        }
        _ => {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "invalid command; see exoctl help",
            ));
        }
    };
    call(
        &resolved.policy,
        &resolved.socket,
        &Request {
            version: PROTOCOL_VERSION,
            context: resolved.context,
            operation,
        },
    )
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["help"] || args == ["--help"] {
        println!("{HELP}");
        return;
    }
    let result = run(args);
    let status = result.as_ref().err().map_or(0, Error::exit_status);
    println!("{}", serde_json::to_string(&response(result)).unwrap());
    std::process::exit(status);
}
