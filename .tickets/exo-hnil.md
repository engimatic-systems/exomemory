---
id: exo-hnil
status: open
deps: [exo-5iw6]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-190g
tags: [m2, sqlite, cli, concurrency]
---
# Replace SQLite revisions with compare-and-swap through exoctl

## Background in Plain English — WHY

Two agents may read the same revision and later propose different updates. Silent last-write-wins would discard one update without warning. Sources: spec/data service.md, spec/exoctl-command-spec.md, and NoStaleWriterCommit in verification/Exomemory.tla.

## Outcomes / Acceptance Criteria — WHAT

1. exoctl overwrite requires --if-revision and a complete valid replacement, including any changes to body, metadata, triggers, scope, and backend parameters.
2. Only an authorized overwritable memory can change; the test-scope policy applies to the replacement as well as the existing memory.
3. A transaction commits a strictly newer complete revision only if the supplied revision is still current.
4. A stale attempt returns a distinct nonzero exit status plus current revision and metadata without changing any stored fields or active triggers.
5. Concurrent writers from the same base revision produce exactly one winner and explicit conflicts for stale losers; readers and matchers see complete old or new state.
6. Regression tests cover changed triggers, failed replacement, immutable records, cross-session updates within the same tenancy, and scope violations.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use a conditional update and affected-row check inside the same transaction as all revision changes. Keep conflict reconciliation in the caller; do not silently retry a stale proposal against a fresh revision.

## Future work

exo-bios demonstrates deliberate reconciliation. exo-fr7t checks process interruption and database contention around replacement.

## Definitions

Compare-and-swap: update only if the stored version still equals the version the caller read. Linearizable transition: concurrent changes behave as if each took effect at one instant. Immutable record: a memory that was not marked overwritable.
