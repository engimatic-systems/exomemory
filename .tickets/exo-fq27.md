---
id: exo-fq27
status: open
deps: [exo-a81g]
links: []
created: 2026-09-19T03:37:24Z
type: epic
priority: 2
assignee: coopwilliams
tags: [roadmap, m3, sqlite, codex]
---
# M3: Accept the SQLite slice in the real Codex harness

## Background in Plain English — WHY

A controlled harness proves the application logic, but real use also requires verified preview delivery and completed-compaction reset hooks. This milestone joins real integration, recovery checks, and the finalized verification contracts into a SQLite acceptance gate. Sources: spec/App server spec.md and verification/RESULTS.md.

## Outcomes / Acceptance Criteria — WHAT

1. A supported Codex integration demonstrates live ghost delivery, separate full-record reads, and completed-compaction reset inside the test folder.
2. Storage and delivery remain correct under the failure and concurrency cases in exo-fr7t.
3. The acceptance checks in exo-a81g pass and identify the tested harness version and remaining product limitations.
4. The milestone remains open until real-host delivery and completed-compaction reset are verified. Missed previews and repetition across compactions are accepted; exact live presence is not required.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Start the capability investigation exo-a6l7 early. Use its evidence to implement exo-dcwx; complete the SQLite feedback milestone before final acceptance.

## Future work

exo-fsde is the explicitly deferred expansion backlog after this SQLite acceptance gate.

## Definitions

Harness: the program that hosts the model, tools, and conversation context. Compaction: a harness operation that replaces or removes earlier context. Acceptance gate: the checks required before calling the increment complete.
