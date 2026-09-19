---
id: exo-6g4f
status: closed
deps: [exo-5iw6]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-190g
tags: [m2, sqlite, ranking, cli]
---
# Persist attributable annoy feedback and deterministic ranking changes

## Background in Plain English — WHY

Agents need a simple way to reduce unwanted previews without deleting their triggers or relying on random suppression. Feedback also needs a recorded state version so an earlier selection can be explained. Sources: spec/exoctl-command-spec.md and spec/data service.md.

## Outcomes / Acceptance Criteria — WHAT

1. exoctl annoy <ghost_key> atomically increments a bounded non-negative penalty and returns the new value; the caller cannot choose an arbitrary penalty.
2. Feedback enforces tenancy and directory access and records the actor or session, attribution, and ordering needed to identify the change.
3. The design specifies how feedback versions relate to record revisions, and candidate snapshots identify both without allowing a later overwrite to silently reset feedback.
4. Concurrent feedback preserves accepted increments up to saturation; overflow cannot wrap or make the penalty negative.
5. Trigger matching is unchanged by feedback, while selection uses the penalty only in its specified lexicographic position; persistence survives restart.
6. Tests cover saturation, concurrent increments, unchanged matching, reproducible selection before and after feedback, and precedence of relevance and scope over penalty.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use a transactional update with a fixed increment and upper bound. A separate monotonic feedback version may preserve body revision semantics. Record enough ordered state for replay without making full record bodies part of routine logs.

## Future work

exo-bios demonstrates feedback with overwrites. exo-a81g verifies that the implementation follows the prose rather than copying the selection model's simplified additive rank.

## Definitions

Attribution: identifying who or which session performed an operation. Saturation: staying at the maximum instead of overflowing. Replay: reproducing a decision from saved inputs and state. Monotonic version: a counter that never decreases.

## Notes

**2026-09-19T15:14:14Z**

Implemented fixed saturating annoy increments with independent feedback versions, ordered session/cwd attribution, transactional migration and candidate snapshots. Overwrite preserves feedback. Four tests cover migration, concurrency, saturation, rollback, CLI/scope and deterministic replay; existing lexicographic tests prove relevance/scope priority. All 30 offline tests and fmt/clippy pass.
