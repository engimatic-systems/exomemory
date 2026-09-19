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
