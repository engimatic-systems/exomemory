---
id: exo-5iw6
status: open
deps: [exo-lm9g]
links: []
created: 2026-09-19T03:37:24Z
type: epic
priority: 1
assignee: coopwilliams
tags: [roadmap, m1, sqlite]
---
# M1: Prove the SQLite create-to-ghost-to-read vertical slice

## Background in Plain English — WHY

The first usable increment must connect storage, a running Data Service, exoctl, and ghost delivery. It uses Rust and only SQLite through rusqlite. A controlled test harness can establish the behavior while real Codex integration is investigated. Sources: spec/overview.md, spec/data service.md, spec/App server spec.md, spec/exoctl-command-spec.md, and verification/DECISIONS.md.

## Outcomes / Acceptance Criteria — WHAT

1. A repeatable test creates a directory-scoped memory through exoctl, matches an event, delivers its ghost, and retrieves its full record through exoctl.
2. All test memory scopes and generated runtime state remain beneath tests/; an event outside the allowed test directory receives no memory.
3. The first slice preserves atomic publication, matching record revisions, deterministic selection, bounded rendering, and authoritative live-context presence.
4. The milestone closes only when exo-lm9g and its prerequisites are closed.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Deliver the child tickets in dependency order and review the executable demonstration before expanding scope. Keep SQLite as the only registered storage plugin; a compiled Rust implementation is sufficient.

## Future work

exo-190g adds conditional updates and feedback. exo-fq27 completes real Codex integration and release evidence. exo-fsde records expansion after SQLite is accepted.

## Definitions

Vertical slice: a small working path through every required layer. Ghost: a short preview that advertises a separately retrieved memory. exoctl: the command-line client. rusqlite: the Rust SQLite library at https://github.com/rusqlite/rusqlite. Plugin: a storage implementation behind an internal interface.
