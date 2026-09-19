---
id: exo-fsde
status: open
deps: [exo-fq27]
links: []
created: 2026-09-19T03:37:25Z
type: epic
priority: 4
assignee: coopwilliams
tags: [deferred, roadmap, post-sqlite]
---
# Deferred: Reassess additional backends and broader use after SQLite acceptance

## Background in Plain English — WHY

The full spec names more backends and use cases than the first slice needs. Keep them visible without making them prerequisites for the SQLite learning loop. Sources: spec/data service.md and spec/spec - use cases to support.md.

## Outcomes / Acceptance Criteria — WHAT

1. After exo-fq27, a review records what the SQLite test runs taught us and chooses the next user outcome before scheduling more implementation.
2. Selected work is split into concrete dependent tickets for zvec, Redis, episodic SQLite, or retrieval-only integrations as justified; unselected ideas remain explicitly deferred.
3. Any external writable backend includes atomic publication through pending revisions and a durable outbox, idempotent confirmation, and hidden pending triggers.
4. Any selected search backend documents reproducible snapshot-bound candidate ordering and capability-specific errors.
5. Global scope, use outside tests/, broader tenancy configuration, additional harnesses, richer trigger types, and operational packaging receive explicit follow-up decisions before rollout.
6. This planning epic does not claim that any additional backend or production rollout has shipped; closure requires linked next-step tickets or an explicit decision to defer them.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use the smallest next vertical slice supported by actual test observations. Keep the Rust backend interface extensible without requiring dynamic plugin loading or speculative frameworks in M1.

## Future work

Concrete expansion tickets are intentionally created only after the SQLite acceptance review; this ticket owns that follow-up and dependency assignment.

## Definitions

zvec: the vector-search backend named in the spec. Redis: the key-value backend named in the spec. Episodic SQLite: the proposed time- and event-oriented SQLite integration. Retrieval-only: a backend that can read but does not accept memory writes. Idempotent: repeating an operation has the same effect as performing it once. Pending revision: a stored version withheld from readers until external publication completes.
