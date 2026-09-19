use exomemory::{transport::*, types::*};
use std::{fs, io::Read, path::PathBuf};
const HELP: &str = r#"exoctl help
exoctl OPTIONS list-trigger-types
exoctl OPTIONS create --json FILE|-
exoctl OPTIONS GHOST_KEY
exoctl OPTIONS meta GHOST_KEY

OPTIONS (all required, before command):
  --runtime DIR --scope-root DIR --socket PATH --tenant ID --session ID --cwd DIR
Paths must be absolute and confined to one explicit tests/.runtime/ run.
Create reads a complete JSON draft from FILE or stdin (-):
  {"body":"...","blurb":"...","uses":[],"topics":[],
   "triggers":[{"kind":"regex","id":"stable-id","pattern":"test"}],
   "backend":{"kind":"sqlite","instance":"local"},
   "scope":{"kind":"dir","root":"/absolute/permitted/directory"},
   "provenance":{"origin":"source","actor":"author"},
   "trust":"untrusted","overwritable":true}
Non-empty validated triggers and an explicit backend are required. Global scope
is disabled in this test profile. Complete records are returned as tool data.

stdout: one JSON object {"version":1,"result":{"Ok":VALUE}} or
{"version":1,"result":{"Err":{"code":CODE,"message":TEXT}}}.
Create and reads return {metadata,body}; metadata includes actual revision,
tenant, scope, provenance, trust, triggers, backend, overwrite permission,
ranking penalty and read estimate. meta returns metadata only.
Exit statuses: 0 success; 2 invalid_input; 3 unavailable; 4 forbidden;
5 unsupported; 6 not_found; 7 conflict; 8 internal.
On response loss, writes may have committed: status 3 is not proof of rollback.
"#;
fn run(args: Vec<String>) -> Result<serde_json::Value> {
    let (options, command) = parse_options(
        &args,
        &[
            "--runtime",
            "--scope-root",
            "--socket",
            "--tenant",
            "--session",
            "--cwd",
        ],
    )?;
    let policy = configured_policy(&options)?;
    let context = Context {
        tenant: required(&options, "--tenant")?,
        session: required(&options, "--session")?,
        cwd: PathBuf::from(required(&options, "--cwd")?),
    };
    let operation = match command.as_slice() {
        [op] if op == "list-trigger-types" => Operation::ListTriggerTypes,
        [op, flag, path] if op == "create" && flag == "--json" => {
            let input: Box<dyn Read> = if path == "-" {
                Box::new(std::io::stdin())
            } else {
                Box::new(
                    fs::File::open(path)
                        .map_err(|e| Error::new(ErrorCode::InvalidInput, e.to_string()))?,
                )
            };
            let mut bytes = Vec::new();
            input.take(MAX_WIRE_BYTES + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_WIRE_BYTES {
                return Err(Error::new(ErrorCode::InvalidInput, "draft too large"));
            }
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            validate_trigger_kinds(&value)?;
            Operation::Create {
                draft: serde_json::from_value(value)?,
            }
        }
        [op, key] if op == "meta" => Operation::Meta { key: key.clone() },
        [key] if key != "create" && key != "meta" => Operation::Read { key: key.clone() },
        _ => {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "invalid command; see exoctl help",
            ));
        }
    };
    call(
        &policy,
        &PathBuf::from(required(&options, "--socket")?),
        &Request {
            version: PROTOCOL_VERSION,
            context,
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
