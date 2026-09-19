---
id: exo-mkyx
status: closed
deps: [exo-b12v]
links: []
created: 2026-09-19T03:37:24Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, sqlite, triggers]
---
# Match scoped triggers and return deterministic SQLite candidates

## Background in Plain English — WHY

Memories must surface from relevant events, and identical state must produce identical candidate order. The SQLite integration needs real SQL-backed candidate retrieval with controlled query behavior. Sources: spec/data service.md and spec/App server spec.md.

## Outcomes / Acceptance Criteria — WHAT

1. At least ordinary regex event triggers are executable and discoverable with validated parameters, normalization rules, and stable trigger identifiers.
2. A matching trigger performs a read-only, parameterized SQLite candidate query; event text and memory data cannot become SQL syntax or run writes.
3. Candidate generation uses one consistent committed snapshot and records enough state identity to reproduce the result.
4. Every candidate includes stable key, revision, source pair, relevance band, backend-local rank, scope, provenance, trust, current penalty, and bounded-preview fields.
5. Filtering applies tenancy and canonical directory scope before exposing results; duplicate matches remain identifiable for App Server collapse.
6. Repeated equivalent requests produce the same ordered candidates, including SQL ties; tests cover invalid patterns, nonmatches, out-of-scope events, and records that failed publication.
7. read_token_estimate is populated using a documented repeatable approximation of the referenced full record's retrieval cost, refreshed with its revision; it is not used as a runtime injection allowance.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Begin with a small built-in query template set and explicit ORDER BY ties, not a general user-authored query engine. Specify normalization and ordering next to the Rust types. Reuse the same supported-trigger registry in exoctl discovery.

## Future work

exo-jtv5 applies the cross-candidate ordering and source limits. exo-6g4f versions mutable penalties. exo-reie adds structured dates; exo-upbn and exo-03m1 supply the other named backend-specific trigger types.

## Definitions

Regex: a regular expression used to match text. SQL: the query language used by SQLite. Parameterized query: SQL with separate bound input values. Snapshot: a consistent view of committed state. Relevance band: an ordered relevance category, not a score compared across backends. Source pair: backend instance plus trigger identifier. read_token_estimate: an approximate count of model text tokens needed to retrieve the full record.

## Notes

**2026-09-19T04:24:07Z**

Implemented regex normalization and validated parameters, stable source IDs, read-only parameterized candidate SQL and one committed snapshot/version per query. Scope filtering precedes result exposure; duplicate matches remain explicit for App Server collapse. Full-record UTF-8 JSON bytes/4 is the documented deterministic read-cost approximation. Tests cover repeatability, duplicate triggers, invalid regex, SQL-like event text, scope boundaries, normalization and failed publication. Formatting, clippy -D warnings and candidate tests pass.
