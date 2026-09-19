# Data Service

The Data Service is a persistent daemon that accepts normalized read and write requests, brokers connections to storage backends, and supplies ghost candidates to the App Server. It responds to the App Server with candidate records and to `xo` with read, creation, feedback, and conditional-overwrite results.

The implementation should probably be written in Rust for low latency. It must handle multiple concurrent connections, but concurrency must not expose partial records or silently lose accepted writes.

## Record and revision contract

Every writable memory has a stable `ghost_key` and a monotonically increasing revision. A revision contains the body, ghost metadata, trigger definitions, backend reference, scope, provenance, and trust classification needed to interpret it.

A revision becomes readable and trigger-visible atomically. Readers see either the complete previous revision or the complete next revision, never a trigger without data or a ghost describing a different body revision.

When all canonical state is in one transactional store, creation and overwrite commit in one transaction. When an external backend participates, the canonical store uses a pending revision plus a durable outbox:

1. Write the complete pending revision and outbox operation.
2. Perform and confirm the external backend operation idempotently.
3. Atomically mark that revision ready and make its triggers visible.

Reads and trigger evaluation ignore pending revisions. Failed pending work may be retried or cleaned up, but it is never advertised as a ghost.

Overwrite is compare-and-swap. The caller supplies the revision it read. The Data Service commits the replacement only if that revision is still current; otherwise it returns a conflict with the current revision and metadata. Per-key revision transitions are linearizable.

## Deterministic candidate contract

After a trigger fires, its backend integration returns an ordered candidate list. Each candidate includes:

- a stable `ghost_key` and record revision;
- a source identifier consisting of `(backend instance, trigger id)`;
- a documented relevance band;
- a deterministic rank within that backend result;
- scope, provenance, and trust metadata; and
- the fields needed to render the bounded ghost.

An integration must define deterministic normalization, filtering, ordering, and tie-breaking. Approximate search is acceptable only when the same indexed snapshot and query parameters produce the same ordered result. Raw scores from different backend types are not assumed to be numerically comparable.

`annoy` feedback does not randomly include or exclude a result and does not change whether its trigger fires. The Data Service atomically increments a bounded, non-negative ranking penalty for the memory. The App Server incorporates that penalty into its deterministic ordering.

If a backend is retrieval-only—for example, one that searches local files using ripgrep-all—writes to that backend fail explicitly.

## Initial backends

- `SQLite`: triggers run SQL queries for candidates; `xo` reads return a row.
- `zvec`: triggers run deterministic, snapshot-bound vector similarity searches; `xo` reads return the referenced data.
- `redis`: triggers run key-value queries that return a candidate; `xo` reads return the full value.
- `episodic-SQLite`: triggers query by date or event; `xo` reads can return events before, after, or during the referenced event.
