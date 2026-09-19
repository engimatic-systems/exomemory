---
id: exo-b12v
status: closed
deps: [exo-sjlw]
links: []
created: 2026-09-19T03:37:24Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, sqlite, storage]
---
# Publish and read complete SQLite memory revisions atomically

## Background in Plain English — WHY

A ghost must never advertise a missing body or metadata from a different version. SQLite can hold all canonical memory state in one transaction, so the first plugin needs no external-store publication machinery. Sources: spec/data service.md; NoDanglingGhost and GhostBodySameVersion in verification/Exomemory.tla.

## Outcomes / Acceptance Criteria — WHAT

1. The SQLite plugin uses rusqlite and a versioned schema for stable keys, complete revisions, body, metadata, non-empty triggers, backend reference, scope, tenancy, provenance, trust, overwrite permission, and initial ranking state.
2. Creation returns a stable key and initial revision only after all canonical state commits together.
3. Body and metadata reads return the complete committed revision and enforce access and directory scope.
4. Failed creation exposes neither a partial record nor active triggers; reopening the database preserves successfully committed records.
5. The plugin capability contract distinguishes supported reads and writes and returns an explicit unsupported-operation error for unavailable capabilities.
6. Focused transaction and reopen tests exercise the publication invariants under tests/.runtime/.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Keep the plugin behind a Rust interface and register SQLite statically. Use transactions, constraints, and explicit schema migration versions. Do not enable arbitrary SQLite extensions. Add failure injection at meaningful transaction boundaries.

## Future work

exo-ljge exposes create and read to exoctl. exo-mkyx adds trigger queries. exo-hnil extends complete revision publication to conditional replacement. exo-3m9v implements the durable outbox for external backends.

## Definitions

Canonical state: the authoritative stored record. Transaction: a group of database changes that commits entirely or has no effect. Atomic publication: making every part of a revision visible at once. Schema migration: a versioned change to database structure. Durable outbox: stored pending work used to coordinate an external storage system.

## Notes

**2026-09-19T04:18:41Z**

Implemented bundled rusqlite schema migration 1 and complete revision publication in one immediate transaction, including trigger rows and a canonical state version. Reads enforce shared tenancy/scope policy. An injected SQLite trigger failure after body insertion rolls back all canonical tables; reopening preserves committed full records. Capability flags identify unsupported operations. Formatting, clippy with warnings denied, and all four workspace integration tests pass.
