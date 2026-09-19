---
id: exo-190g
status: closed
deps: [exo-bios]
links: []
created: 2026-09-19T03:37:24Z
type: epic
priority: 2
assignee: coopwilliams
tags: [roadmap, m2, sqlite]
---
# M2: Complete the SQLite revision and feedback loop

## Background in Plain English — WHY

Once the first memory can be created, surfaced, and read, the next increment lets agents update records without discarding another writer's work and make unwanted memories rank lower. Sources: spec/exoctl-command-spec.md and verification/DECISIONS.md.

## Outcomes / Acceptance Criteria — WHAT

1. A stale overwrite returns a machine-readable conflict while the winning complete revision remains readable and trigger-visible.
2. Negative feedback changes a persisted, bounded ranking penalty deterministically without changing trigger matching.
3. The repeatable multi-session demonstration in exo-bios passes with all state and memory scopes confined to tests/.
4. The milestone closes only when exo-bios and its prerequisites are closed.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Implement exo-hnil and exo-6g4f independently after M1, then demonstrate them together. Include each operation's regression coverage in its implementation change.

## Future work

exo-fq27 adds real harness acceptance and failure recovery evidence. exo-fsde covers backend expansion.

## Definitions

Revision: an identifiable complete version of a memory. Ranking penalty: a non-negative number that moves a memory later within otherwise equal ranking fields. Machine-readable conflict: a structured error that a program can recognize and reconcile.

## Notes

**2026-09-19T15:16:01Z**

M2 criteria and dependencies satisfied: exo-hnil, exo-6g4f and exo-bios closed with implementation and actual process-level demo evidence. See verification/SQLITE-UPDATES.md.
