---
id: exo-fsde
status: open
deps: [exo-1izl]
links: []
created: 2026-09-19T03:37:25Z
type: epic
priority: 3
assignee: coopwilliams
tags: [roadmap, full-spec, post-sqlite]
---
# Complete the remaining specified backends and memory workflows

## Background in Plain English — WHY

The full spec names more backends and use cases than the first SQLite slice needs. This epic owns concrete implementation and acceptance tickets for that remaining scope. Discovery starts immediately, SQLite-only extensions follow their actual prerequisites, and non-SQLite implementation waits for exo-fq27. Sources: all spec/ documents and verification/SPEC-COVERAGE.md.

## Outcomes / Acceptance Criteria — WHAT

1. exo-pw6d resolves the zvec integration question without waiting for implementation milestones.
2. exo-u1mp and exo-3m9v provide backend routing and atomic external publication, with exo-03m1, exo-upbn, exo-jku1, and exo-52xr delivering Redis, zvec, episodic SQLite, and retrieval-only workflows.
3. exo-reie and exo-hlw6 complete structured date triggers and general scope policy; exo-t3ru demonstrates notes-file references and ordinary agent file operations without adding an Exomemory append command.
4. The seven specified use cases have executable evidence through existing exo-bios and the new backend, scope, and exo-t3ru scenarios.
5. All live test runs keep memories and runtime state under tests/; general global-scope semantics use synthetic policy fixtures without enabling real global stores.
6. The epic closes only after exo-1izl and its prerequisites pass; creating the tickets alone does not constitute implementation completion.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Follow tk dependencies and keep each backend's create-or-register, ghost, and read path reviewable as an increment. Refine methods using SQLite observations without silently dropping required outcomes. The parent relationship groups work and is not a dependency on this epic.

## Future work

exo-1izl is the final acceptance gate for the concrete scope here. exo-qvzg is a bounded, nonblocking discovery for Pi as an additional host. It does not add Pi implementation to this epic until that discovery creates a concrete implementation ticket and dependency. Operational packaging and production rollout remain unselected.

## Definitions

zvec: the vector-search backend named in the spec. Redis: the key-value backend named in the spec. Episodic SQLite: the time- and event-oriented SQLite integration. Retrieval-only: a backend that can read but does not accept body writes. Atomic publication: exposing every part of a complete revision together. Synthetic fixture: a constructed test value that does not create a live global memory. Acceptance gate: the checks required before claiming the work is complete.
