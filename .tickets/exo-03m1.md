---
id: exo-03m1
status: open
deps: [exo-3m9v]
links: []
created: 2026-09-19T03:49:33Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-sqlite, redis, backend]
---
# Deliver a Redis create-to-ghost-to-read backend slice

## Background in Plain English — WHY

Redis is an initial backend in the Data Service spec: triggers run key-value queries, and exoctl retrieves the full value. The backend needs a usable path through creation, matching, reading, and updates while preserving the shared contracts. Source: spec/data service.md, Initial backends.

## Outcomes / Acceptance Criteria — WHAT

1. A Rust Redis integration registers named instances and documented key-value trigger parameters discoverable through exoctl.
2. Creation and conditional overwrite publish complete revision-bound values through exo-3m9v; retries cannot replace a ready value with unconfirmed content.
3. Matching produces a deterministic ordered candidate list with stable keys, source pairs, relevance bands, snapshot or revision-set identity, provenance, trust, and feedback state.
4. exoctl retrieves the full referenced value and actual revision, with missing data, unavailable Redis, and unsupported operations reported explicitly.
5. Concurrent overwrite and annoy behavior preserve the canonical revision and feedback contracts; an external query cannot bypass tenancy or directory scope.
6. An isolated local Redis scenario demonstrates create, match, bounded ghost, full read, overwrite conflict, restart recovery, and feedback using fixture data; persistent service output stays under tests/.runtime/.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Choose a maintained Rust client during implementation and use bound command arguments. Prefer immutable versioned Redis keys and a bounded documented query vocabulary. Do not depend on a user's existing Redis instance.

## Future work

exo-1izl checks Redis together with SQLite and zvec. No additional Redis-specific work is identified in the current spec.

## Definitions

Redis: the key-value backend named in the spec. Revision-bound value: content tied to one immutable memory revision. Query vocabulary: the supported set of query forms. Snapshot identity: the recorded identity of the state used to produce a result.
