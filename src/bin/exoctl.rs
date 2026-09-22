use exomemory::{
    client::{ClientConfig, USER_CONFIG_RELATIVE},
    transport::*,
    types::*,
};
use std::{collections::BTreeMap, fs, io::Read, path::PathBuf};
const OVERVIEW: &str = include_str!("../../spec/overview.md");
const SHORT_USAGE: &str = r#"Usage: exoctl [CONNECTION_OPTIONS] COMMAND
Commands: list-trigger-types | create --json FILE|- | GHOST_KEY |
          meta GHOST_KEY | annoy GHOST_KEY |
          overwrite GHOST_KEY --if-revision REVISION --json FILE|-
Connection overrides: [--profile test|local] [--runtime DIR] [--scope-root DIR]
  [--socket PATH] [--tenant ID] [--session ID] [--cwd DIR]
Run `exoctl --help` for configuration, draft fields, results, and examples."#;
const HELP_BEFORE_OVERVIEW: &str = r#"Exomemory command line

Usage:
  exoctl list-trigger-types
  exoctl create --json FILE|-
  exoctl GHOST_KEY
  exoctl meta GHOST_KEY
  exoctl annoy GHOST_KEY
  exoctl overwrite GHOST_KEY --if-revision REVISION --json FILE|-
  exoctl help | --help

Normal installed configuration:
  Explicit connection options override EXOMEMORY_CONFIG, which overrides
  $HOME/.config/exomemory/exoctl.json. The installed file supplies the local
  runtime, permitted directory root, socket, and tenant. cwd comes from the
  process. CODEX_THREAD_ID supplies the session when present; otherwise the
  session is `terminal`.

Create a JSON draft, then inspect the returned ghost_key and revision. Use the
bare key for a complete read, `meta` for origin and trigger metadata, `annoy` for
negative ranking feedback, and `overwrite --if-revision` for a deliberate
compare-and-swap update.

"#;
const HELP_AFTER_OVERVIEW: &str = r#"
Non-empty validated triggers and an explicit backend are required. Global scope
is disabled in both current profiles. Complete records are returned as tool data.

Create and overwrite read a complete JSON draft from FILE or stdin (-):
  {"body":"...","blurb":"...","uses":[],"topics":[],
   "triggers":[{"kind":"regex","id":"stable-id","pattern":"test"}],
   "backend":{"kind":"sqlite","instance":"local"},
   "scope":{"kind":"dir","root":"/absolute/permitted/directory"},
   "provenance":{"origin":"source","actor":"author"},
   "trust":"untrusted","overwritable":true}
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

Controlled and administrator connection options (before COMMAND):
  --profile test|local --runtime DIR --scope-root DIR --socket PATH
  --tenant ID --session ID --cwd DIR
All six path/identity values must be explicit for a controlled test invocation.
Paths must be absolute. The test profile accepts only isolated repository harness
runs; local requires a private runtime and an explicit permitted directory root.
"#;

const OPTIONS: [&str; 7] = [
    "--profile",
    "--runtime",
    "--scope-root",
    "--socket",
    "--tenant",
    "--session",
    "--cwd",
];

fn parse_cli_options(args: &[String]) -> Result<(BTreeMap<String, String>, Vec<String>)> {
    let mut options = BTreeMap::new();
    let mut index = 0;
    while index < args.len() && args[index].starts_with("--") {
        let name = &args[index];
        if !OPTIONS.contains(&name.as_str()) {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                format!("unknown option {name}; run `exoctl --help` for valid options"),
            ));
        }
        if options.contains_key(name) {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                format!("option {name} was supplied more than once; keep one explicit value"),
            ));
        }
        let value = args.get(index + 1).filter(|value| !value.starts_with("--"));
        let Some(value) = value else {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                format!("option {name} requires a value before the command"),
            ));
        };
        options.insert(name.clone(), value.clone());
        index += 2;
    }
    Ok((options, args[index..].to_vec()))
}
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

fn overwrite(command: &[String]) -> Result<Operation> {
    let key = command
        .get(1)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            Error::new(
                ErrorCode::InvalidInput,
                "overwrite requires GHOST_KEY --if-revision REVISION --json FILE|-",
            )
        })?;
    if command.get(2).map(String::as_str) != Some("--if-revision") {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "overwrite requires --if-revision REVISION after GHOST_KEY",
        ));
    }
    let revision = command.get(3).ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidInput,
            "overwrite option --if-revision requires a revision value",
        )
    })?;
    if revision.starts_with("--") {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "overwrite option --if-revision requires a revision value",
        ));
    }
    let if_revision = revision.parse().map_err(|_| {
        Error::new(
            ErrorCode::InvalidInput,
            format!("invalid revision {revision:?}; use a positive integer revision from meta"),
        )
    })?;
    if if_revision == 0 {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "invalid revision 0; use a positive integer revision from meta",
        ));
    }
    if command.get(4).map(String::as_str) != Some("--json") {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "overwrite requires --json FILE|- after the revision",
        ));
    }
    let path = command.get(5).ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidInput,
            "overwrite option --json requires a file path or - for stdin",
        )
    })?;
    if path.starts_with("--") {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "overwrite option --json requires a file path or - for stdin",
        ));
    }
    if command.len() != 6 {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            format!(
                "unexpected overwrite argument {:?}; expected overwrite GHOST_KEY --if-revision REVISION --json FILE|-",
                command[6]
            ),
        ));
    }
    Ok(Operation::Overwrite {
        key: key.clone(),
        if_revision,
        draft: read_draft(path)?,
    })
}

fn operation(command: &[String]) -> Result<Operation> {
    let Some(name) = command.first().map(String::as_str) else {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            format!("missing command\n{SHORT_USAGE}"),
        ));
    };
    match name {
        "list-trigger-types" if command.len() == 1 => Ok(Operation::ListTriggerTypes),
        "list-trigger-types" => Err(Error::new(
            ErrorCode::InvalidInput,
            "list-trigger-types takes no arguments",
        )),
        "create" if command.len() == 1 => Err(Error::new(
            ErrorCode::InvalidInput,
            "create requires --json FILE|-",
        )),
        "create" if command.get(1).map(String::as_str) != Some("--json") => Err(Error::new(
            ErrorCode::InvalidInput,
            format!(
                "unexpected create option {:?}; expected create --json FILE|-",
                command[1]
            ),
        )),
        "create" if command.len() == 2 => Err(Error::new(
            ErrorCode::InvalidInput,
            "create option --json requires a file path or - for stdin",
        )),
        "create" if command[2].starts_with("--") => Err(Error::new(
            ErrorCode::InvalidInput,
            "create option --json requires a file path or - for stdin",
        )),
        "create" if command.len() > 3 => Err(Error::new(
            ErrorCode::InvalidInput,
            format!(
                "unexpected create argument {:?}; expected create --json FILE|-",
                command[3]
            ),
        )),
        "create" => Ok(Operation::Create {
            draft: read_draft(&command[2])?,
        }),
        "overwrite" => overwrite(command),
        "meta" | "annoy" if command.len() == 1 => Err(Error::new(
            ErrorCode::InvalidInput,
            format!("{name} requires GHOST_KEY"),
        )),
        "meta" | "annoy" if command.len() > 2 => Err(Error::new(
            ErrorCode::InvalidInput,
            format!(
                "unexpected {name} argument {:?}; expected {name} GHOST_KEY",
                command[2]
            ),
        )),
        "meta" => Ok(Operation::Meta {
            key: command[1].clone(),
        }),
        "annoy" => Ok(Operation::Annoy {
            key: command[1].clone(),
        }),
        _ if command.len() == 1 => Ok(Operation::Read {
            key: command[0].clone(),
        }),
        _ => Err(Error::new(
            ErrorCode::InvalidInput,
            format!(
                "unexpected argument {:?}; a full read accepts one GHOST_KEY",
                command[1]
            ),
        )),
    }
}

fn run(args: Vec<String>) -> Result<serde_json::Value> {
    let (options, command) = parse_cli_options(&args)?;
    let operation = operation(&command)?;
    let resolved = resolve(options)?;
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
    if args.is_empty() {
        println!("{SHORT_USAGE}");
        std::process::exit(2);
    }
    if args == ["help"] || args == ["--help"] {
        print!("{HELP_BEFORE_OVERVIEW}{OVERVIEW}{HELP_AFTER_OVERVIEW}");
        return;
    }
    let result = run(args);
    let status = result.as_ref().err().map_or(0, Error::exit_status);
    println!("{}", serde_json::to_string(&response(result)).unwrap());
    std::process::exit(status);
}
