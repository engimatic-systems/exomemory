---
id: exo-jku1
status: open
deps: [exo-u1mp, exo-reie]
links: []
created: 2026-09-19T03:49:33Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-sqlite, sqlite, episodic, use-case]
---
# Build the episodic SQLite backend and timeline reconstruction workflow

## Background in Plain English — WHY

The spec names episodic-SQLite for date- and event-based retrieval and illustrates incrementally building a timeline of remembered events. Ordinary row reads do not provide the required temporal neighborhood operations. Sources: spec/data service.md, Initial backends; spec/spec - use cases to support.md, Timeline reconstruction.

## Outcomes / Acceptance Criteria — WHAT

1. A separate episodic-SQLite plugin implemented in Rust with rusqlite stores events with stable memory keys, complete revisions, timestamp or interval fields, and provenance.
2. Date or event triggers return deterministic, scoped candidates using exo-reie and stable ties for simultaneous events.
3. exoctl can retrieve the referenced event and events before, after, or during it through a documented grammar and machine-readable result format.
4. Temporal reads define reference-event identity, interval overlap rules, stable ordering, result limits, and continuation behavior without hiding unreturned events.
5. Create, conditional overwrite, and feedback reuse the canonical contracts; all canonical state in one SQLite store publishes transactionally without unnecessary external outbox work.
6. A test-folder timeline scenario adds events across sessions, revises one event, and reconstructs before, after, and overlapping portions with correct revisions and no out-of-scope events.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use the same SQLite transaction boundary for the event body, metadata, and temporal indexes. Define narrow query operators and explicit continuation fields when bounding full tool reads.

## Future work

exo-1izl verifies this backend alongside the other named initial backends. No further timeline work is required by the current spec.

## Definitions

Episodic memory: a record of an event and when it occurred. Temporal neighborhood: events before, after, or overlapping a reference event. Interval overlap: sharing part of a time range. Continuation: data needed to retrieve the next page of a bounded result.
