---
id: exo-h2bg
status: closed
deps: [exo-sjlw]
links: []
created: 2026-09-19T03:37:24Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, harness, context]
---
# Model authoritative live ghost presence in a controlled Rust harness

## Background in Plain English — WHY

Cooldown and append-only history cannot say whether a ghost is still in live context. The first slice needs a controlled harness that owns this truth and exposes the same contract required of a real adapter. Sources: spec/App server spec.md; AtMostOneGhostPerWindow and NoStaleHistorySuppression in verification/Exomemory.tla.

## Outcomes / Acceptance Criteria — WHAT

1. Normalized events identify session, live context, logical event number, working directory, event type, and authoritative presence version.
2. The harness ledger records occurrence identifiers and ghost keys only when delivery is accepted into live context, and compaction supplies the actual retained occurrences.
3. One key cannot appear twice in a live context even after cooldown expires; a removed key can become eligible after its cooldown.
4. Selection and acceptance coordinate against a presence version so overlapping, retried, or rejected deliveries cannot overfill slots or create duplicate occurrences.
5. Last-shown event state changes only after accepted delivery, and cooldown is measured in logical events per session and key.
6. Tests cover accepted and failed delivery, concurrent events, event retry, compaction that retains or removes occurrences, and independent sessions or child contexts.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Build a deterministic in-process test harness with explicit delivery acknowledgments and compaction events. Use a session lock or conditional ledger update. Do not infer presence from old transcript files.

## Future work

exo-jtv5 consumes the ledger and remaining slots. exo-lm9g exercises the full controlled-harness loop. exo-dcwx implements this contract for Codex using exo-a6l7 evidence.

## Definitions

Ledger: the authoritative set of ghosts currently present in context. Occurrence identifier: the identity of one accepted insertion. Cooldown: a minimum event interval before resurfacing. Presence version: a change counter that detects a stale context view. Acknowledgment: confirmation that the harness accepted a delivery.

## Notes

**2026-09-19T04:28:53Z**

Implemented the controlled harness authoritative live context as transactional SQLite live items, with per-context presence versions, per-session logical event/cooldown state and exact idempotent receipts. Only accepted delivery changes last-shown state. Compaction validates actual retained occurrence IDs. Tests cover rejection, accepted retry, replay after compaction, retained/removed occurrences, cooldown, simultaneous stale views, independent sessions/child contexts, atomic batch failure, reopen and tenant binding. Formatting, clippy -D warnings and four presence tests pass.
