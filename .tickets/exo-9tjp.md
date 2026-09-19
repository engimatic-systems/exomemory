---
id: exo-9tjp
status: open
deps: [exo-0kao]
links: []
created: 2026-09-19T20:07:17Z
type: epic
priority: 1
assignee: coopwilliams
parent: exo-fq27
tags: [handoff, pre-merge]
---
# Polish Exomemory's standalone interface before M3

## Background in Plain English — WHY

The first personal installation proved that Exomemory can store and recall memories in normal Codex terminal sessions. That trial also exposed product-facing rough edges: injected previews reveal internal origin fields, the installed `exoctl` is a shell wrapper around a binary whose help describes the test profile, parser failures hide the offending option, and Codex routing depends on a shell script. These issues make the tool harder to understand and package even though the underlying memory path works.

Codex 0.155.1 still needs a local launcher to connect its terminal UI to the hosted session where Exomemory hooks can inject previews. This epic keeps that requirement explicit while making the shipped commands self-contained compiled programs.

## Outcomes / Acceptance Criteria — WHAT

1. `exo-dn0t` removes `provenance` and `source_id` from rendered ghosts without removing the internal source identity used for deterministic selection and quotas.
2. `exo-i68v` installs `exoctl` as a self-configuring Rust binary with no shell wrapper and no skill required for normal use.
3. `exo-v7n1` makes bare usage, full help, and option errors sufficient to discover and correct ordinary commands.
4. `exo-uctk` replaces the generated Codex shell launcher with a compiled, reversible launcher while leaving the official Codex executable untouched.
5. `exo-0kao` verifies migration, command behavior, ghost output, launcher routing, and real recall with GPT 5.6 Luna before this epic closes.
6. Specifications, command help, installation notes, and verification evidence describe the same final behavior and do not direct personal installations into `tests/.runtime/`.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Implement the ghost and configuration changes first, then generate help from the revised overview. The compiled Codex launcher can proceed independently. Keep each implementation in its own reviewable change and use the acceptance ticket to validate the combined installed result.

## Future work

`exo-qvzg` investigates a direct Pi integration that may avoid the Codex-specific launcher. `exo-fsde` owns broader post-SQLite backends and workflows.

## Definitions

Rendered ghost: the bounded memory preview injected into model context. Self-configuring binary: an executable that reads its installed configuration itself instead of relying on a shell script to append arguments. Launcher: the local `codex` executable that starts the official Codex CLI with the remote host connection required by this integration. M3: the real-Codex SQLite acceptance milestone.
