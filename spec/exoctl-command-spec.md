# `exoctl` command

The `exoctl` command is the model's POSIX-compatible data control plane. It queries the Data Service daemon and provides stable exit statuses, machine-readable errors, and the following operations.

## Discovery

- `exoctl help` describes the installed command grammar and result formats.
- `exoctl list-trigger-types` returns the currently available trigger types and their parameters. Types may include ordinary regex triggers, designated structured triggers such as dates, vector similarity, and backend-specific triggers.

## Creation

`exoctl create [<data>] <description> <triggers-and-params> <backend-and-params>` creates a memory with a non-empty trigger set. The caller specifies the backend when storage is required and may mark the record overwritable across sessions in the same tenancy. Retrieval-only backends do not require stored data.

Triggers are explicitly scoped along filesystem boundaries:

- `global` applies to sessions in every directory permitted by the tenancy.
- `dir` applies recursively within an explicitly recorded directory root.

Creation returns the stable `ghost_key` and initial revision. It succeeds only after the complete revision is atomically visible under the Data Service publication contract. A pending or partially failed external-backend write is not reported as a successfully created memory and cannot fire a trigger.

## Conditional overwrite

`exoctl overwrite <ghost_key> --if-revision <revision> ...` proposes a complete replacement revision, including any changed body, ghost metadata, trigger definitions, scope, or backend parameters.

The write succeeds only if `revision` is still current. A stale write returns a conflict, a distinct nonzero exit status, and the current revision and metadata; it never silently overwrites newer work. The caller may retrieve the new state, reconcile, and retry deliberately.

## Reads

- `exoctl <ghost_key>` returns the full record as tool data through stdout.
- `exoctl meta <ghost_key>` returns all metadata, including revision, source, provenance, trust, scope, trigger definitions, ranking penalty, and fields excluded from the rendered ghost.

The ghost revision tells the agent which version its preview described. Reads identify the revision actually returned.

## Negative feedback

`exoctl annoy <ghost_key>` atomically increments the memory's bounded ranking penalty and returns the new value. It does not change whether the trigger fires, does not directly edit a vector threshold, and never introduces a random surface/no-surface decision. The agent does not choose the numeric penalty directly.

On later events, the App Server uses the penalty as one field of its deterministic lexicographic ranking. The operation is attributable and versioned so that a selection decision can be reproduced from recorded state.
