---
id: exo-sjlw
status: open
deps: []
links: []
created: 2026-09-19T03:37:24Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, rust, isolation]
---
# Establish the Rust workspace and test-folder isolation contract

## Background in Plain English — WHY

Every later experiment needs an isolated place to store memories and run the service. The user requires Rust, SQLite through rusqlite, and memories scoped only to the test folder. Isolation must apply to paths and scope checks, not just where a database happens to be stored.

## Outcomes / Acceptance Criteria — WHAT

1. A Cargo workspace builds the shared memory types, Data Service, App Server, exoctl, and SQLite plugin boundaries; Cargo.lock is tracked.
2. Each test run receives a unique directory under tests/.runtime/ for its database, socket, logs, and harness state, with no default fallback to a user memory store.
3. Test-mode creation and overwrite reject global scope and directory roots outside the run's allowed directory beneath tests/.
4. The shared path policy rejects parent traversal, sibling-prefix confusion, and symlink escapes for scopes and runtime paths; callers outside the allowed scope cannot read or surface test memories.
5. Shared versioned request, response, record, candidate, and event types identify tenancy, directory, provenance, trust, record revision, and logical event number.
6. Rust formatting, linting, and relevant tests have runnable commands; generated large outputs are ignored while source fixtures and Cargo.lock remain eligible for Git.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Prefer a small number of Rust crates or modules. Choose an explicit local transport and application-owned SQLite build; rusqlite documents its bundled feature at https://github.com/rusqlite/rusqlite. Use a shared test-directory helper and parameterized path checks. No Python is needed; use uv if Python tooling is later introduced.

## Future work

exo-b12v implements transactions; exo-ljge connects the daemon and client; exo-h2bg defines context events in detail. exo-fsde handles production configuration beyond the test profile.

## Definitions

Cargo: Rust's build and dependency manager. Tenancy: the ownership boundary used to permit memory access. Provenance: information about where a memory came from. Trust classification: a typed label describing how retrieved data must be treated. Logical event number: an ordered counter of harness events. Symlink: a filesystem link that can redirect a path.
