---
id: exo-reie
status: open
deps: [exo-5iw6]
links: []
created: 2026-09-19T03:49:33Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-m1, sqlite, triggers]
---
# Add discoverable structured date and event triggers

## Background in Plain English — WHY

The first slice supplies regex triggers, while the command and backend specs also name structured dates and event-oriented retrieval. Those triggers require explicit interpretation of time so the same normalized event yields the same result. Sources: spec/exoctl-command-spec.md, Discovery; spec/data service.md, episodic-SQLite.

## Outcomes / Acceptance Criteria — WHAT

1. A documented structured trigger type recognizes designated date or event fields and is listed with its parameter schema by exoctl list-trigger-types.
2. Trigger definitions specify accepted timestamp formats, timezone handling, interval boundaries, and how event identity relates to temporal matching.
3. Relative-time interpretation, if supported, uses an explicit normalized reference time rather than an unrecorded wall-clock read.
4. Invalid or ambiguous structured values return defined validation errors; create and overwrite cannot publish malformed trigger definitions.
5. SQLite candidate queries preserve scope, provenance, revision identity, stable ordering, and normal coexistence with regex triggers.
6. Focused tests cover equal timestamps, timezone boundaries, inclusive or exclusive range endpoints, invalid values, and repeated normalized events, with test-only memory scopes.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Begin with a small explicit timestamp and event-identifier grammar. General natural-language date understanding is not required. Use the existing normalized event types and SQLite query templates.

## Future work

exo-jku1 uses these triggers for before, after, and during reads; exo-1izl includes trigger discovery and mixed-trigger evidence.

## Definitions

Structured trigger: a rule matching typed fields rather than arbitrary prose. Timestamp: a recorded date and time. Timezone: the rules mapping local time to an absolute instant. Interval boundary: the start or end of a time range.
