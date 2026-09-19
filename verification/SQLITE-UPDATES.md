# SQLite conditional updates and feedback

`exoctl OPTIONS overwrite KEY --if-revision REVISION --json FILE|-` takes the
same complete draft as create. Both the existing memory and replacement scope
must be accessible; the existing memory must be overwritable. An immediate
SQLite transaction checks the old revision, inserts the complete replacement
and triggers, conditionally moves the visible pointer, and increments the store
snapshot version. Old revisions remain immutable. Failed publication rolls back.

A stale caller receives exit 7 and `result.Err.current` metadata, including the
current revision. It must read and reconcile deliberately. There is no automatic
retry with a newer revision. Cross-session writes in the same tenant are allowed.
A response lost after commit still has an uncertain outcome.

`cargo test --test overwrite` checks concurrent writers, complete observations,
changed triggers, rollback after body insertion, immutable records, scope/tenant
rejection and the CLI conflict envelope. Full-workspace formatting, clippy and
26 offline tests passed when conditional updates were introduced.

## Feedback versions and attribution

`exoctl OPTIONS annoy KEY` increments the penalty by one, saturating at
`4294967295`. Each accepted request, including one at saturation, advances that
memory's `feedback_version` and the global `state_version`. `feedback_events`
records those versions, key, tenant, session, canonical cwd and resulting penalty
in the same transaction. No caller-selected penalty or randomness is supported.
Session identity is supplied by this local controlled client; it is attribution,
not authentication of a remote principal.

Body revision, feedback version and global snapshot version are distinct.
A body overwrite advances the revision and global version but preserves penalty
and feedback version. Current metadata overlays the separate feedback row on the
immutable body revision, then recalculates its read estimate. Candidate snapshots
contain each body's revision and feedback version plus the global version, with
all reads pinned to one SQLite snapshot. Feedback does not alter triggers or
backend rank. The exact ranking remains relevance, scope, penalty, backend rank,
last attempt, stable key, and source tie-break. Earlier relevance/scope fields
always take precedence over any penalty (`tests/selection.rs`).

Data schema 1 migrates transactionally to schema 2 using
`migrations/003_feedback.sql`; existing revisions and penalties are preserved.
The attempt database is separate and has its own schema number. New revision
JSON stores the feedback snapshot at publication; the feedback table is the
source of truth for current reads. `tests/feedback.rs` covers migration,
snapshot replay, restart, overwrite preservation, eight concurrent increments,
saturation, attribution, rollback and authorization. Thirty offline tests pass.

## M2 acceptance

`cargo test --test m2 -- --nocapture` runs real exod/exoctl clients in two named
sessions against one isolated run. Both read revision 1, race different complete
updates, and get one success and one explicit conflict. The losing session reads
the winner and deliberately combines its data with a later measurement in
revision 3. Two matching memories then demonstrate quota-respecting selection
before and after feedback. Restart preserves revisions and feedback; saved
snapshots replay both selections. The initial passing run was
`tests/.runtime/r27eb7b0dc9f8`; every run writes `harness/m2-evidence.json`.
