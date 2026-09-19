---
id: exo-ljge
status: open
deps: [exo-b12v]
links: []
created: 2026-09-19T03:37:24Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, service, cli]
---
# Create and retrieve test memories through the daemon and exoctl

## Background in Plain English — WHY

The first slice must exercise the actual client-to-service path so a model can store and retrieve records. Sources: spec/data service.md and spec/exoctl-command-spec.md.

## Outcomes / Acceptance Criteria — WHAT

1. A persistent Rust Data Service accepts multiple local client connections using explicit test-root, database, and endpoint configuration.
2. exoctl help, list-trigger-types, create, <ghost_key>, and meta <ghost_key> have a documented grammar in command help and stable structured output.
3. Creation requires a non-empty supported trigger set and explicit SQLite backend parameters, and returns success only after committed publication.
4. Full reads return body and actual revision through stdout as tool data; metadata reads include all metadata fields, including fields omitted from the ghost.
5. Invalid input, unavailable service, forbidden scope, unsupported backend or trigger, and missing key have documented machine-readable errors and stable nonzero exit statuses; stdout remains parseable.
6. Process-level integration tests start an isolated daemon, create and read a memory, restart it, and confirm persistence.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Choose a POSIX-compatible command grammar that resolves optional body input unambiguously, including stdin if useful. A local Unix socket and structured messages are reasonable options. Bind every request to an explicit trusted local session context and reuse the shared access policy.

## Future work

exo-mkyx connects trigger matching to the service. exo-hnil and exo-6g4f add the remaining commands after M1. exo-fr7t expands failure behavior and concurrent load coverage.

## Definitions

Daemon: a long-running service process. POSIX: conventions for Unix-like command-line tools. stdout: a process's normal output stream. Unix socket: a local endpoint for communication between processes. Structured output: data with a documented format that programs can parse.
