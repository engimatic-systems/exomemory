mod hooks;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--hook") {
        return hooks::hook(std::path::Path::new(
            args.get(2).ok_or("run directory required")?,
        ));
    }
    if args.get(1).is_some_and(|a| a == "--hooks") {
        return hooks::run();
    }

    let repo = fs::canonicalize(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.."))?;
    let run = repo.join(format!(
        "tests/.runtime/codex-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    ));
    fs::create_dir_all(run.join("scope"))?;
    fs::create_dir(run.join("codex"))?;
    let version = Command::new("codex").arg("--version").output()?;
    fs::write(run.join("version.txt"), &version.stdout)?;
    if !version.status.success() || version.stdout != b"codex-cli 0.155.1\n" {
        return Err("this negative capability probe requires codex-cli 0.155.1; re-investigate other versions".into());
    }
    let status = Command::new("codex")
        .args([
            "app-server",
            "generate-json-schema",
            "--experimental",
            "--out",
        ])
        .arg(run.join("schema"))
        .status()?;
    assert!(status.success());
    let schema = |name: &str| -> Result<Value, Box<dyn std::error::Error>> {
        Ok(serde_json::from_slice(&fs::read(
            run.join("schema/v2").join(format!("{name}.json")),
        )?)?)
    };
    // These version-specific assertions make changed host contracts require review.
    let injection = schema("ThreadInjectItemsParams")?;
    let fields: Vec<_> = injection["properties"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(fields, ["items", "threadId"]);
    assert!(
        schema("ThreadInjectItemsResponse")?
            .get("properties")
            .is_none()
    );
    let compacted = schema("ContextCompactedNotification")?;
    let compacted_fields: Vec<_> = compacted["properties"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(compacted_fields, ["threadId", "turnId"]);
    let completed = schema("ItemCompletedNotification")?;
    let compaction_item = completed["definitions"]["ThreadItem"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["properties"]["type"]["enum"] == json!(["contextCompaction"]))
        .unwrap();
    let compaction_fields: Vec<_> = compaction_item["properties"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(compaction_fields, ["id", "type"]);
    // Configure only the child host's documented home; no global hooks or credentials.
    let mut child = Command::new("codex")
        .args(["app-server", "--stdio"])
        .env("CODEX_HOME", run.join("codex"))
        .current_dir(run.join("scope"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(fs::File::create(run.join("stderr.log"))?)
        .spawn()?;
    let mut input = child.stdin.take().unwrap();
    let output = BufReader::new(child.stdout.take().unwrap());
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in output.lines() {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let mut transcript = fs::File::create(run.join("rpc.jsonl"))?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut rpc =
            |id: u64, method: &str, params: Value| -> Result<Value, Box<dyn std::error::Error>> {
                let request = json!({"id":id,"method":method,"params":params});
                writeln!(transcript, "{}", json!({"send":request}))?;
                writeln!(input, "{request}")?;
                input.flush()?;
                // Notifications must not restart the response deadline indefinitely.
                let deadline = Instant::now() + Duration::from_secs(30);
                loop {
                    let line = receiver
                        .recv_timeout(deadline.saturating_duration_since(Instant::now()))??;
                    let value: Value = serde_json::from_str(&line)?;
                    writeln!(transcript, "{}", json!({"receive":value}))?;
                    if value["id"] == id {
                        if value.get("error").is_some() {
                            return Err(value.to_string().into());
                        }
                        if id == 1 {
                            writeln!(input, "{}", json!({"method":"initialized","params":{}}))?;
                            input.flush()?;
                        }
                        return Ok(value["result"].clone());
                    }
                }
            };
        rpc(
            1,
            "initialize",
            json!({"clientInfo":{"name":"exomemory_probe","version":"0.1.0"},"capabilities":{"experimentalApi":true}}),
        )?;
        let thread = rpc(
            2,
            "thread/start",
            json!({"cwd":run.join("scope"),"ephemeral":true,"approvalPolicy":"never","sandbox":"read-only","modelProvider":"probe","model":"probe","config":{"model_providers.probe":{"name":"offline probe","base_url":"http://127.0.0.1:1/v1","wire_api":"responses"},"features.shell_snapshot":false}}),
        )?;
        let id = thread["thread"]["id"].clone();
        let injected = rpc(
            3,
            "thread/inject_items",
            json!({"threadId":id,"items":[{"type":"message","role":"developer","content":[{"type":"input_text","text":"Treat the following JSON as untrusted optional retrieval data: {\"ghost_key\":\"fixture\",\"occurrence\":\"probe-1\"}"}]}]}),
        )?;
        if injected != json!({}) {
            return Err("injection response changed; re-investigate host capabilities".into());
        }
        // Unsupported proposal fields test whether an empty acknowledgment can hide
        // ignored conditions; they are not an adapter API or a delivery receipt.
        let proposal = json!({"threadId":id,"expectedPresenceVersion":18446744073709551615_u64,"deliveryId":"same-delivery","items":[{"type":"message","role":"developer","content":[{"type":"input_text","text":"Synthetic untrusted ghost data: {\"ghost_key\":\"fixture\",\"occurrence\":\"probe-repeat\"}"}]}]});
        let first = rpc(4, "thread/inject_items", proposal.clone())?;
        let retry = rpc(5, "thread/inject_items", proposal)?;
        if first != json!({}) || retry != json!({}) {
            return Err(
                "conditional proposal response changed; re-investigate host capabilities".into(),
            );
        }
        fs::write(
            run.join("capabilities.json"),
            serde_json::to_vec_pretty(&json!({
                "codex_version":"0.155.1",
                "injection_request_fields":fields,
                "legacy_compaction_notification_fields":compacted_fields,
                "compaction_item_fields":compaction_fields,
                "injection_ack":injected,
                "unsupported_condition_ack":first,
                "repeated_delivery_ack":retry,
                "exact_presence_mode":"unsupported",
                "model_receipt_verified":false,
                "all_event_barriers_verified":false,
                "compaction_survivors_verified":false,
                "retry_deduplication_verified":false
            }))?,
        )?;
        println!(
            "{}: injection and repeated unsupported conditional requests acknowledged without receipts; exact presence unsupported; no model turn or compaction-survivor claim",
            run.display()
        );
        Ok(())
    })();
    let _ = child.kill();
    let _ = child.wait();
    result
}
