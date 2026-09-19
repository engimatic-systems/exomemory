# Codex integration decision — 0.155.1

Current best-effort integration path: [Codex hooks](CODEX-HOOKS.md).

> Historical exact-presence investigation. On 2026-09-19 the user approved
> best-effort attempts with reset after completed compaction. The missing
> presence interface below no longer blocks M3 and no Codex fork is required
> for that reason. Current contract: [attempt suppression](ATTEMPT-SUPPRESSION.md).

The follow-up [host contract investigation](CODEX-HOST-CONTRACT.md) adds an
executable negative capability check, specifies the required host extension,
and records the host-distribution decision needed for exo-adfe. Exact presence
remains unsupported; exo-7pi0, exo-dcwx, and M3 remain incomplete.

Tested 2026-09-19 with installed `codex-cli 0.155.1` on macOS arm64.
Run `cargo run --manifest-path tests/probes/codex/Cargo.toml --target-dir target/probes`.
The Rust probe creates a unique `tests/.runtime/codex-*` directory, generates
experimental schemas, and starts a separate stdio host with its own Codex
home and an ephemeral directory-scoped thread. An offline provider prevents
model calls. No global hooks, credentials, or memories are installed.
The transcript and stderr remain in that run directory; response waits are
bounded and the child is killed and reaped on success or failure.

## Evidence and decision

The actual RPC run acknowledged `initialize`, emitted `thread/started`, and
acknowledged `thread/inject_items` with `{}` for a synthetic escaped data
message. This establishes protocol acceptance of injection into a loaded
thread. It does **not** prove model receipt, survival through compaction,
or compatibility with every hook. No model turn was requested.

Use the versioned stdio App Server protocol as the adapter boundary. The
installed generated schemas are stronger version evidence than current
online documentation. The [official App Server documentation](https://learn.chatgpt.com/docs/app-server)
describes schema generation, item injection and lifecycle notifications.
Its [injection section](https://learn.chatgpt.com/docs/app-server#inject-items)
describes appending to model history, while the generated injection response
has no presence version or accepted occurrence list.

## Required capabilities

| Capability | Installed schema / observed interface | Evidence limit |
| --- | --- | --- |
| Prompt | `userPromptSubmit` hook; `turn/start` | Schema; pre-model scanning must be probed |
| Tool call and response | `preToolUse`, `postToolUse`; `item/started`, `item/completed` | Notifications alone are not barriers |
| Permission | `permissionRequest`; command/file/permissions approval RPCs | Schema; event barrier mapping remains to verify |
| Session | `sessionStart`, `sessionEnd`; `thread/started` | Thread start notification observed |
| Sub-agent | `subagentStart`, `subagentStop`; collaboration items | Schema; child context ownership remains to verify |
| Compaction | `preCompact`, `postCompact`; `contextCompaction` item | Compaction item contains an ID, not retained occurrences |
| Turn completion | `stop`; `turn/completed` | Schema only |
| Ghost delivery | experimental `thread/inject_items` | Empty successful acknowledgment observed |
| Authoritative live occurrences | No occurrence snapshot or conditional presence version in inspected request/notification schemas | Unsupported by the demonstrated adapter path |

`HookEventName` includes every event family above and `HookExecutionMode`
includes sync and async. This is promising, but enum membership does not
prove that every hook can insert and synchronize before the next model step.
Stored history (`thread/read`, item pagination, rollout files) cannot establish
which items survive in live model input. Raw-event output is explicitly
internal-use-only and is not evidence of a supported presence contract.

## Concrete missing interface

exo-7pi0 investigates synchronous barriers now, and specifies a host-owned
versioned occurrence snapshot plus conditional insertion. The host must update
this snapshot in the same critical section as accepted insertion and context
mutation. Compaction/fork/resume must report the actual retained occurrences;
a retry must return the same acceptance, not add another occurrence.

exo-dcwx depends on that work. Until verified, real Codex integration remains
incomplete and must report unsupported exact-presence mode. M1 continues with
its controlled Rust harness; the spec explicitly permits that increment.
