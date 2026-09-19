---
id: exo-fr7t
status: open
deps: [exo-190g]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-fq27
tags: [m3, sqlite, reliability]
---
# Verify SQLite and delivery recovery under failures and contention

## Background in Plain English — WHY

Transactions and context ledgers must remain correct when processes fail or requests overlap. The normal demonstrations do not cover all interruption points that can expose partial state or duplicate ghosts. Sources: verification/RESULTS.md and spec/data service.md.

## Outcomes / Acceptance Criteria — WHAT

1. Process-interruption tests around create and overwrite commits recover to either complete old or complete new state, with no dangling trigger or mixed revision.
2. Parallel reads, creates, overwrites, and feedback have documented bounded busy handling and preserve accepted state without silent lost updates.
3. Connection loss after a commit has explicit client behavior for an uncertain result, and retry behavior cannot be mistaken for proof that a previous write failed.
4. Delivery failure, stale ledger versions, duplicate events, and restart require authoritative presence synchronization before more ghosts can be accepted.
5. Service startup, shutdown, and stale endpoint handling are deterministic and confined to test runtime paths.
6. Focused tests exercise the identified failure boundaries, with reproducible seeds or schedules and compact failure artifacts.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use subprocesses and controlled fault points for actual crash evidence. Decide connection ownership and busy limits explicitly. Prefer focused concurrency tests over a broad benchmark project.

## Future work

exo-a81g makes the recovery checks part of acceptance. exo-fsde handles operational features beyond the local SQLite slice.

## Definitions

Contention: multiple operations competing for database access. Busy handling: how the service retries or rejects a temporarily locked database. Uncertain result: a disconnected caller cannot yet know whether its write committed. Fault point: a controlled place to interrupt execution.
