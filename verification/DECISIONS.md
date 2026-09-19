# Finalized design decisions

Recorded from the submitted review form on 2026-09-18.

1. **Atomic publication:** store a complete revision and expose it atomically;
   use an outbox when an external backend participates.
2. **Concurrent writers:** `xo overwrite` uses compare-and-swap revisions and
   returns an explicit conflict rather than silently losing an update.
3. **Context presence:** the harness owns an authoritative live-context ledger
   and reports compaction. Cooldown is rate limiting, not deduplication truth.
4. **Injection bounds and ordering:** use a maximum ghost count and a hard
   limit on each final rendered ghost. Do not maintain a runtime token budget.
   Select with deterministic lexicographic ranking, a stable final tie-break,
   deterministic `annoy` penalties, and per-source diversity limits.
5. **Authority:** an App Server developer message may provide the trusted
   interpretation wrapper, but retrieved ghost fields remain typed, escaped,
   untrusted data with provenance. Full records return as tool data.

For diversity accounting, a source is the pair `(backend instance, trigger
id)` after candidates with the same ghost key have been collapsed. This stops
one broad trigger from filling every slot without penalizing all retrievals
from the same backend implementation.
