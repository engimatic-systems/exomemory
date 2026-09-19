---
id: exo-3m9v
status: open
deps: [exo-u1mp]
links: []
created: 2026-09-19T03:49:33Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-sqlite, storage, concurrency]
---
# Publish external backend revisions through a durable outbox

## Background in Plain English — WHY

A SQLite transaction cannot atomically commit an independent backend operation. The spec requires pending revisions and a durable outbox so failed or slow external writes never surface as ready memories. Sources: spec/data service.md, Record and revision contract; verification/DECISIONS.md, Atomic publication.

## Outcomes / Acceptance Criteria — WHAT

1. The canonical SQLite store atomically records the complete pending revision and a durable external operation before dispatching backend work.
2. Backend work has a stable operation identity and is safe to retry after service failure, including failure after external success but before canonical confirmation.
3. Only confirmed work becomes the ready current revision in one canonical transaction; reads and trigger queries ignore pending revisions and continue to see the complete prior ready revision.
4. External objects or equivalent version bindings preserve the prior ready body during pending overwrite; writing a mutable external key in place cannot expose a new body under old metadata.
5. Concurrent replacements honor the original compare-and-swap condition at publication; losing pending work cannot become current or destroy another writer's external data.
6. Creation and overwrite do not report successful completion while pending; documented failure or pending responses, bounded retries, and safe cleanup make interruption outcomes recoverable.
7. Fault-injection tests cover each handoff, duplicate delivery, stale writers, permanently failed work, and restart using a local test backend entirely under tests/.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use version-addressed external objects plus canonical publication, or an equally sound strategy. Separate acknowledgment of request receipt from publication success. Start with a deterministic external-store test double before integrating real services.

## Future work

exo-03m1 and exo-upbn supply concrete idempotent external operations. exo-1izl verifies their behavior under combined candidate queries.

## Definitions

Durable outbox: database records describing external work that survives a restart. Pending revision: a complete proposed version hidden until external work is confirmed. Idempotent: safe to repeat without changing the intended effect. Compare-and-swap: publish only if the expected current revision still matches. Test double: a controlled substitute used to exercise an interface.
