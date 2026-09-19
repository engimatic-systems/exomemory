# Best-effort preview attempts

The user approved this replacement for exact live presence on 2026-09-19.
A SQLite set records attempted keys per session/context. `AttemptStore::process`
reads that set, selects candidates, and commits selected keys plus last-attempt
ordering in one immediate transaction. Only then does it return bytes to send
once. No acknowledgment changes this state. There is no delivery replay queue,
occurrence ledger, accepted-delivery receipt, or host presence version.

Failed or abandoned sends remain suppressed until completed compaction. A
retained preview may be sent again after the reset. These are intentional
tradeoffs, not invariant violations. No failure frequency is assumed or promised.
Creation and overwrite transactions keep their stronger guarantees.

`CompactionCompleted` deletes only the affected context's set. `CompactionStarted`
and restart do not. Positive session event numbers prevent an old repeated
completion from clearing newer attempts. Child contexts use separate sets and
share session cooldown and event order. Host adapters must serialize a context's
send with its reset handling and must discard abandoned batches rather than
replaying them after reconnect. The store does not hold a database lock across
network I/O, and cannot itself establish when a host event took place.

Count limits apply to attempts between completed compactions. For byte bounds,
assume every ghost is sent in its own batch with a fresh wrapper. The conservative
interval limit is `max_attempts * (max_ghost_bytes + max_wrapper_bytes)`.
This does not bound all old previews that might remain in live model input.
Rendering still rejects oversize or malformed ghosts without truncation.

Run `cargo test --test attempts`, `cargo test --test selection`,
`cargo test --test rendering`, and `cargo test --test m1 -- --nocapture`.
The attempt tests cover abandoned sends, reopen, duplicate events, completed-only
reset, context isolation, concurrent selection, count limits, transaction
rollback, and rejected scope/settings/schema changes. M1 exercises the actual
daemon and exoctl with a controlled receiver. Real Codex evidence remains the
responsibility of exo-7pi0 and exo-dcwx.

Attempt databases use schema 2. Old schema-1 controlled-harness databases are
rejected without rewriting their records; they cannot reveal previously failed
attempts. Start a fresh attempt database with a new test context. The canonical
memory database schema is unchanged. The historical migration remains for
explicit rejection testing and interpreting earlier artifacts.

`AttemptSuppression.tla` is a finite model of marking before an optional send,
completed-only reset, and context isolation. It checks at most one send per key
per interval and attempt capacity. The previous exact-presence models remain
historical checks under their original configurations. Their old live-presence
properties are not requirements of this new implementation. `GhostSelection.tla`
still abstracts ranking with arithmetic; Rust uses the prose lexicographic order.
