# Finalized design decisions

Initially recorded from the review form on 2026-09-18; context delivery revised by explicit user approval on 2026-09-19.

1. **Atomic publication:** store a complete revision and expose it atomically;
   use an outbox when an external backend participates.
2. **Concurrent writers:** `exoctl overwrite` uses compare-and-swap revisions and
   returns an explicit conflict rather than silently losing an update.
3. **Best-effort attempts:** atomically record selected preview keys in SQLite
   before sending once. Suppress further attempts in that context until completed
   compaction clears its set. Preserve marks across failure and restart; no
   delivery retries or authoritative live presence are required. Dropped previews
   and repeats across compactions are accepted. Cooldown uses last-attempt events.
4. **Injection bounds and ordering:** use a maximum attempt count per context between completed compactions and a hard
   limit on each final rendered ghost; include one bounded wrapper per possible send.
   These bound new injection per interval, not all previews remaining in context. Do not maintain a runtime token budget.
   Select with deterministic lexicographic ranking, a stable final tie-break,
   deterministic `annoy` penalties, and per-source diversity limits.
5. **Authority:** an App Server developer message may provide the trusted
   interpretation wrapper, but rendered ghost fields remain typed, escaped,
   untrusted data. Source identity and provenance stay internal until an explicit
   metadata read. Full records return as tool data.

For diversity accounting, a source is the pair `(backend instance, trigger
id)` after candidates with the same ghost key have been collapsed. This stops
one broad trigger from filling every slot without penalizing all retrievals
from the same backend implementation.

The earlier exact-presence decision and host-fork requirement are superseded.
See `verification/ATTEMPT-SUPPRESSION.md` for implementation and migration limits.
