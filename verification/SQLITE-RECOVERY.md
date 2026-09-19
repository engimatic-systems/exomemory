# SQLite and attempt recovery

Run `cargo test --test recovery -- --nocapture`. All seven test entries passed
on Rust 1.98.0; one entry is the subprocess worker used by the fault tests.
The schedules are deterministic commit barriers, not random timing or a claim
of exhaustive fault coverage.

The crash worker compiles the same `src/sqlite.rs` with a test-only callback.
Production builds have no fault field or callback. A parent waits until all
publication writes have executed immediately before commit, or commit has
returned, then kills and reaps the worker. This covers create and overwrite on
both sides of the commit. The real daemon reopens each database; integrity,
canonical table counts, complete reads, and matching previews must agree with
the old or new revision. Each case saves the reached barrier, integrity result,
and candidate snapshot in `harness/recovery-evidence.json`, with worker logs.

A separate worker dies after reserving an attempt and before sending it. Reopen
must keep it suppressed, completed compaction must enable a later attempt, and
a stale completion must not clear that later attempt. `tests/attempts.rs` also
checks concurrent count limits, context isolation and rollback;
`tests/codex_adapter.rs` checks unavailable hosts and lost acknowledgments.

A local proxy forwards a real overwrite, observes the daemon's committed result,
and drops the response. The caller receives an uncertain-result error; repeating
the stale overwrite conflicts, and revision 2 remains readable. A lost create
response likewise does not prove rollback: the caller must not blindly recreate
it. This is separate from optional previews, which are never retried.

Four concurrent clients mix reads, creates, overwrites and feedback. Every
accepted increment and create survives; exactly one same-revision writer wins.
An externally held SQLite write lock returns `unavailable` after the two-second
busy timeout and leaves feedback unchanged. The daemon serializes database work
with one mutex; the busy bound applies to SQLite lock acquisition, not an
unbounded aggregate of queued requests. Client/socket I/O has a five-second
bound, so a queued request can outlive its caller and have an uncertain result.
There is no transparent write retry. Snapshot-version exhaustion returns a
conflict and rolls back create, overwrite or feedback rather than becoming a
floating-point SQLite value.

Endpoint checks preserve active sockets and ordinary files, set socket mode
0600, and recover a stale socket after process shutdown. All files, sockets,
child processes and memory scopes belong to the unique test run. Controlled
local filesystem ownership remains required; hostile concurrent path mutation
and production process supervision are outside this slice.
