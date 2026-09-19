# Codex hook path — 0.155.1

The best-effort attempt policy uses ordinary synchronous hooks and the existing
stdio App Server. It requires no exact live-context snapshot or host fork.
The version-specific executable is:

```sh
cargo run --manifest-path tests/probes/codex/Cargo.toml --target-dir target/probes -- --hooks
```

This runs the installed Codex host with a **scripted local HTTP provider**, not
actual model inference. Live inference tests must use `gpt-5.6-luna` as requested
by the user. The probe creates a unique `tests/.runtime/hooks-*` tree containing
an isolated Codex home, scope, temporary files, hook inputs, model requests,
RPC transcript, and evidence summary. No global hook is installed and no model
API key is loaded. Hook trust is bypassed only for this generated test setup.

## Observations and checks

The probe drives prompt, local command call/response, permission, session start
and end, child start and stop, pre/post compaction, and turn completion. It
requires every hook to be observed and every completed handler to report
successful synchronous execution. It holds the first prompt hook and checks
that the local provider receives nothing until release. Prompt, session-start,
tool, and child-start output markers must occur in actual model request bodies.

`PreCompact` is first made to stop the operation. No `PostCompact` may appear.
A subsequent successful compaction must run `PostCompact`; the root session
then runs `SessionStart` with `source: compact` before continuing. A child
context is also compacted through its thread ID.

The permission hook waits while the probe sends `thread/inject_items` over
stdio. The injection acknowledgment releases the hook, and its marker must
appear in the next model request. This establishes a way to add context at a
hook whose output schema does not accept `additionalContext`. The probe does
not retry insertion or treat its acknowledgment as a live-presence receipt.

## Adapter implications

- Thread creation needs `config: {"bypass_hook_trust": true}` for this isolated
  reviewed fixture. In the observed build, passing only the top-level CLI hook
  trust flag loaded definitions but did not execute them.
- `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, and
  `SubagentStart` accept `hookSpecificOutput.additionalContext`.
- `Stop` and `SubagentStop` reject that output shape. Their fixture returns `{}`;
  permission returns its own decision schema. Use App Server insertion for
  preview delivery at hooks without a context-output field, then release the
  waiting hook. The permission probe verifies this path, not every such event.
- Child hook input provides the parent `session_id` and separate `agent_id`.
  The adapter must keep their attempt sets separate while sharing session
  ordering/cooldown. Do not derive presence from the transcript path.
- Only `PostCompact` resets attempts. `SessionStart` with source `compact`
  must not clear them a second time. Ordinary startup/resume must reopen the
  same persisted set for the same context.
- Keep context output bounded to avoid host truncation. Configuration can disable
  output spilling for a handler that already strictly bounds its own bytes.

The [official hook documentation](https://learn.chatgpt.com/docs/hooks) describes
synchronous execution, event inputs, and event-specific output. Installed
schemas and these runtime checks establish the tested version. Hook output
shapes must not be assumed interchangeable merely because an event enum exists.

This is integration-path evidence, not the complete M3 application demo.
exo-dcwx still owns memory creation, unsolicited ghost delivery, explicit full
reads, attempt suppression/reset, isolation, and real inference evidence.
