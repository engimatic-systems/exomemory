# Codex best-effort adapter acceptance

The supported test host is `codex-cli 0.155.1`; real inference uses
`gpt-5.6-luna`, per the user's instruction. The repeatable live command is:

```sh
cargo test --test codex_live -- --ignored --nocapture
```

This opt-in test uses existing Codex authentication and makes real model calls.
It starts an isolated SQLite daemon and Codex host under a unique printed
`tests/.runtime/r*` directory. A private credential copy is removed afterward;
logs never intentionally include credential files or request headers. The host's
configuration, hooks, temporary files, memory scope, sockets, and databases all
belong to that run. No global hook or normal memory store is used.

The test creates a synthetic record through exoctl, then asks Luna to plan test
work without supplying the new memory key. A matching hook injects its bounded
preview. Luna uses the preview key to retrieve the complete record through an
exoctl tool call. The test checks the full body marker and key in actual tool
output. A second turn must preserve the last-attempt number. Manual compaction
must run PostCompact and clear the set; a later matching turn must record a
new attempt. All observed hook handlers must finish successfully. The scenario
then archives the test thread, exercising SessionEnd too.

`harness/codex-evidence.json` records the host/model, before/after attempt state,
and full-read result for the direct-host scenario. The installed-path scenario
records compiled-command identities, help identity, metadata, trusted hook
definitions, full-read confirmation, suppression and post-compaction attempt
states, login-shell resolution, and authentication cleanup in
`harness/installed-evidence.json`. `logs/codex-rpc.jsonl` and
`logs/installed-rpc.jsonl` record their respective model and tool events.
`verification/CODEX-HOOKS.md` separately documents scripted-provider checks for
all eleven event types, permission insertion, a held prompt barrier, aborted
compaction, and root/child compaction identity. Scripted responses are not
represented as real Luna inference.

## Adapter operation

`exo-codex-hook --config FILE` receives one JSON hook input on stdin. The config
has `runtime`, `scope_root`, `tenant`, `data_socket`, `codex_socket`, `attempts`,
and `bounds` fields. `bounds` has `max_attempts`, `max_ghost_bytes`,
`max_wrapper_bytes`, `interval_envelope_bytes`, `per_source`, and
`cooldown_events`. Every path is explicit and confined to the test run.

Configure every verified lifecycle hook as a synchronous invocation of that
command. Start Codex with `app-server --listen unix://ABSOLUTE_SOCKET_PATH`.
Only the isolated, reviewed test thread sets `bypass_hook_trust = true`.
The CLI always leaves approval/stop decisions to Codex by returning an empty
hook response. Operational errors go to stderr with the usual error exit code;
Codex can continue without the optional preview.

The adapter uses `agent_id` as the child context when present, while retaining
the parent `session_id` for ordering/cooldown. It scans event content, not
transport paths or IDs. It queries candidates through exod and uses the shared
transactional attempt selector. A bounded cross-process file lock orders one
send with the next reset; it is not a live-presence ledger. File-lock acquisition
and Codex RPCs are bounded to two seconds each; data service I/O retains its
five-second bound. A host hook may time out and drop a preview. No delivery is
retried, and failed/uncertain delivery never unmarks a key.

A fresh local WebSocket connection initializes the version-pinned App Server
interface and injects one developer message containing the trusted wrapper plus
escaped untrusted JSON. The public JSON contains `ghost_key`, `revision`, `blurb`,
`uses`, `topics`, `read_token_estimate`, and `trust`; source identity and
provenance remain internal and are available through explicit metadata output.
Full records use tool output. Final ghost bounds include
the extra JSON string escaping needed by this RPC. The wrapper limit includes
its escaped bytes and request framing; an invalid bound is rejected before any
key is marked. Count/byte bounds apply to new attempts between completed
compactions, not all information a compaction summary may retain.

## Limits and regression coverage

`cargo test --test codex_adapter` verifies all event mappings, parent/child
identity, scope rejection, failed sends, lost acknowledgments, no retries,
completed-only reset, the exact public ghost boundary, escaped data, and framing
limits. `tests/attempts.rs`
checks concurrent reservations, rollback, persistence, context-local reset,
logical-event replay, and cooldown. The host invokes each configured hook once;
the adapter does not replay hooks on reconnect. Repeating a PostCompact command
manually represents another completion invocation, so installations must not
add an external hook retry mechanism.

The live test depends on authentication, host availability, and model behavior,
so it is excluded from ordinary offline `cargo test --workspace`. Run it
explicitly when accepting host integration changes. Best-effort preview delivery
does not relax record publication, overwrite, scope, or feedback guarantees.
M3 still requires M2, failure recovery, and the final traceability gate.
