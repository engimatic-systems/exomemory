use exomemory::{
    codex::{HookConfig, handle},
    types::*,
};
use std::io::Read;

fn run() -> Result<()> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["help"] || args == ["--help"] {
        println!(
            "exo-codex-hook --config FILE | --env\nRead one Codex 0.155.1 hook JSON object from stdin. FILE contains explicit runtime, scope_root, tenant, data_socket, codex_socket, attempts and bounds. The default test profile confines paths to tests/.runtime; local must be selected explicitly. --env reads EXOMEMORY_HOOK_CONFIG or returns an empty response when absent. The host must expose a local Unix WebSocket App Server socket and use synchronous hooks. All eleven lifecycle hooks use this command; only PostCompact clears attempts. Delivery is best effort with no retry. stdout preserves ordinary hook decisions; optional configured SessionStart instructions explain the installed CLI. Failures go to stderr. Use gpt-5.6-luna for live inference."
        );
        return Ok(());
    }
    // Global definitions are inert outside the installed host, which sets this variable.
    if args == ["--env"] {
        let Ok(path) = std::env::var("EXOMEMORY_HOOK_CONFIG") else {
            // One startup diagnostic explains inactive recall without injecting data or blocking Codex.
            let mut input = Vec::new();
            std::io::stdin()
                .take(exomemory::transport::MAX_WIRE_BYTES + 1)
                .read_to_end(&mut input)?;
            if serde_json::from_slice::<serde_json::Value>(&input)
                .ok()
                .is_some_and(|v| v["hook_event_name"] == "SessionStart")
            {
                eprintln!(
                    "Exomemory recall is inactive: this session is outside the Exomemory host. Run exomemory status and launch codex from a new terminal after repairing PATH."
                );
            }

            println!("{{}}");
            return Ok(());
        };
        args = vec!["--config".into(), path];
    }
    let [flag, path] = args.as_slice() else {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "expected --config FILE",
        ));
    };
    if flag != "--config" {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "expected --config FILE",
        ));
    }
    let config: HookConfig = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(exomemory::transport::MAX_WIRE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > exomemory::transport::MAX_WIRE_BYTES {
        return Err(Error::new(ErrorCode::InvalidInput, "hook input too large"));
    }
    let input = serde_json::from_slice(&bytes)?;
    let result = handle(&config, &input);
    // Startup guidance explains the installed CLI; other events preserve ordinary hook decisions.
    let response = if input["hook_event_name"] == "SessionStart" {
        config.instructions.as_ref().map(|text| serde_json::json!({"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":text}})).unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    println!("{response}");
    result.map(|_| ())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("preview hook: {error}");
        std::process::exit(error.exit_status());
    }
}
