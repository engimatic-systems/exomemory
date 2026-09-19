---
id: exo-bios
status: closed
deps: [exo-hnil, exo-6g4f]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-190g
tags: [m2, demo, acceptance]
---
# Demonstrate concurrent updates and feedback across test sessions

## Background in Plain English — WHY

A complete memory loop must work when sessions overlap and negative feedback arrives between reads and updates. This increment extends the first demonstration with the test-duration use case and later-session retrieval.

## Outcomes / Acceptance Criteria — WHAT

1. A reproducible scenario creates an overwritable test-duration memory, reads one revision in two sessions, accepts one update, and reports a conflict for the stale update.
2. The losing session reads current state and deliberately reconciles using the fresh revision, with body and preview staying consistent.
3. With multiple equally relevant in-scope memories, annoy changes later ordering without removing matching candidates or violating source quotas.
4. Restart preserves accepted record updates and feedback; the recorded state reproduces selection before and after changes.
5. All scenarios remain scoped to a unique run directory under tests/ and pass through the actual daemon and exoctl.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Extend exo-lm9g's fixtures with synchronized clients and known event sequences. Demonstrate that a worse relevance band does not beat a better one merely because of penalty arithmetic.

## Future work

exo-a81g uses this scenario in final SQLite acceptance. exo-fr7t adds failure injection beyond the demonstrated successful and conflicting operations.

## Definitions

Reconciliation: intentionally combining a proposed change with newly read current data. Equally relevant: candidates sharing the same relevance band and preceding ranking fields.

## Notes

**2026-09-19T15:16:01Z**

Actual daemon and exoctl demo passes across two named concurrent sessions: one winner, stale conflict, explicit reconciliation to revision 3, annoy changes ordering without changing matches, source quota preserved, restart and saved-snapshot replay agree. Command cargo test --test m2 -- --nocapture; evidence tests/.runtime/r27eb7b0dc9f8/harness/m2-evidence.json. Formatting and clippy pass.
