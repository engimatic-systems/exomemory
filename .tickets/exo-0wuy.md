---
id: exo-0wuy
status: closed
deps: []
links: []
created: 2026-09-19T14:33:15Z
type: task
priority: 0
assignee: coopwilliams
parent: exo-fq27
tags: [m3, harness, contract]
---
# Replace live presence with best-effort preview attempt suppression

## Background in Plain English — WHY

The user approved a simpler policy on 2026-09-19: remember which previews we
attempted, suppress them until completed compaction, and accept rare drops or
repetition across compactions. The old exact-presence contract required host
interfaces that Codex 0.155.1 does not supply. That complexity is no longer a
product requirement. Sources: spec/App server spec.md and verification/DECISIONS.md.

## Outcomes / Acceptance Criteria — WHAT

1. SQLite selection and marking are one transaction, committed before any send.
2. Each key is attempted at most once per session/context between completed
   compactions, including concurrent hooks, failure, and process restart.
3. Only completed compaction clears that context's set; old repeated completions
   cannot clear newer attempts. Child contexts remain separate.
4. Failed or uncertain delivery is never retried or unmarked. Ranking and cooldown
   use last-attempt events; count and byte limits bound attempts per interval.
5. Occurrence receipts, host presence versions, retained-set tracking, and their
   runtime machinery are removed. Legacy controlled databases fail explicitly.
6. Specs, active tickets, model-check interpretation, and the M1 demo describe the
   revised contract. Meaningful regression tests and recorded checks pass.

## Methodological Suggestions — HOW

Use a SQLite immediate transaction to select and reserve the keys. Keep any
historical model or probe explicitly labeled so it cannot imply current
acceptance requirements. Another implementation preserving these outcomes is valid.

## Future work

exo-7pi0 verifies ordinary Codex hook delivery and completed-compaction behavior.
exo-dcwx integrates that evidence, and exo-a81g gates M3 acceptance.

## Definitions

Attempt set: the keys selected for one attempted delivery, regardless of receipt.
Interval: the time between completed compactions of one context. Best effort:
a failed or uncertain send may be dropped. Atomic: all state changes commit
together or none do. Cooldown: additional rate limiting measured in session events.

## Notes

**2026-09-19T14:40:09Z**

Implemented AttemptStore: selection and mark commit together before send; no receipts, presence versions, occurrence IDs, retained-set tracking, or delivery retries. Completed-only per-context reset preserves session cooldown and ordering; schema-1 harness files fail explicitly. Updated M1, active M3 tickets/specs, and superseded the host-fork ticket. Rust formatting and Clippy pass; 19 integration tests pass. All 11 model configurations match expectations; the new attempt model explores 196 distinct states. Real-host hooks remain exo-7pi0/exo-dcwx work.
