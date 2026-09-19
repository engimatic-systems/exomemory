use exomemory::{transport::*, types::*};
use std::path::PathBuf;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["help"] || args == ["--help"] {
        println!(
            "exod --runtime DIR --scope-root DIR --tenant ID --database FILE --socket PATH\nPaths are explicit. --profile test (default) confines all paths to tests/.runtime/. --profile local requires a private runtime and an explicit permitted scope root.\nServes version-1 JSON lines on a mode-0600 Unix socket. Stale refused sockets are replaced; active sockets and other files are rejected."
        );
        return;
    }
    let result = (|| -> Result<()> {
        let (options, rest) = parse_options(
            &args,
            &[
                "--profile",
                "--runtime",
                "--scope-root",
                "--tenant",
                "--database",
                "--socket",
            ],
        )?;
        if !rest.is_empty() {
            return Err(Error::new(ErrorCode::InvalidInput, "unexpected arguments"));
        }
        let policy = configured_policy(&options)?;
        serve(
            policy,
            PathBuf::from(required(&options, "--database")?),
            PathBuf::from(required(&options, "--socket")?),
        )
    })();
    if let Err(error) = result {
        println!(
            "{}",
            serde_json::to_string(&response(Err(error.clone()))).unwrap()
        );
        std::process::exit(error.exit_status());
    }
}
