use exomemory::{
    codex::{HookConfig, handle},
    types::*,
};
use std::io::Read;

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["help"] || args == ["--help"] {
        println!(
            "exo-codex-hook --config FILE\nRead one Codex 0.155.1 hook JSON object from stdin. FILE contains explicit runtime, scope_root, tenant, data_socket, codex_socket, attempts and bounds. All paths stay inside one tests/.runtime run. The host must expose a local Unix WebSocket App Server socket and use synchronous hooks. All eleven lifecycle hooks use this command; only PostCompact clears attempts. Delivery is best effort with no retry. stdout is an empty hook response; failures go to stderr. Use gpt-5.6-luna for live inference."
        );
        return Ok(());
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
    // An empty response preserves the host's ordinary permission and stop behavior.
    println!("{{}}");
    result.map(|_| ())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("preview hook: {error}");
        std::process::exit(error.exit_status());
    }
}
