---
id: exo-qvzg
status: open
deps: []
links: [exo-uctk]
created: 2026-09-19T20:07:26Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-fsde
tags: [discovery, pi]
---
# Discover a shim-free Pi integration

## Background in Plain English — WHY

The tested Codex integration needs a local `codex` launcher because Codex CLI 0.155.1 does not persist the remote-host connection Exomemory requires. That requirement changes the value proposition for users who do not want another program in front of their normal CLI. Pi may expose lifecycle or extension interfaces that let Exomemory inject previews directly, but the exact Pi project, version, and supported hooks have not been established.

This ticket is a bounded investigation. It decides whether Pi can become a concrete host adapter without shadowing the Pi command or modifying a user's global setup during discovery.

## Outcomes / Acceptance Criteria — WHAT

1. Identify the specific Pi coding-agent implementation and tested version, with primary documentation or source links for its extension and context APIs.
2. Map Exomemory's required events to supported Pi behavior: session start, user prompt submission, stable session and child-context identity, working directory, preview injection, completed compaction, and shutdown or recovery.
3. Build an isolated probe under `tests/probes/` that demonstrates the available lifecycle and injection data, using a unique `tests/.runtime/` directory and no personal memory store. If an API is absent, retain a minimal reproducible counterexample instead.
4. Determine whether a Pi extension can connect to the existing data service and best-effort attempt store without replacing the normal `pi` executable. Record the required install files, permissions, version checks, and failure diagnostics.
5. Write the concrete decision and evidence in a named verification note. If the adapter is feasible, create and link an implementation ticket with explicit dependencies and acceptance commands; if it is not, name the missing capability and the earliest condition for revisiting it.
6. Do not install global Pi hooks, change a user's Pi configuration, or claim Pi support as part of M3 acceptance during this discovery.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Start by recording which project the name “Pi” refers to, then inspect its official extension contract before writing a probe. Reuse the event table in the Codex integration verification note so differences are easy to review.

## Future work

`exo-9tjp` completes the current Codex-based standalone interface. Any Pi implementation ticket created by this discovery belongs under `exo-fsde` and must depend on the capabilities proved here.

## Definitions

Pi: the coding-agent host to be identified precisely by this discovery. Host adapter: integration code that maps an agent's lifecycle and context APIs to Exomemory delivery. Shim-free: integration that does not put an Exomemory executable in front of the agent's normal command. Counterexample: a repeatable probe showing that a required event or API is unavailable.
