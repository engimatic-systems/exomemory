# Exomemory development

## Scope and source of truth

Implement in Rust. The first storage plugin is SQLite using
[rusqlite](https://github.com/rusqlite/rusqlite); an internal Rust interface with
a compiled-in plugin is sufficient. Keep all canonical state in SQLite
transactions for this slice.

Read the relevant files in `spec/` before implementing a ticket. The finalized
contracts are recorded in `verification/DECISIONS.md` and `verification/RESULTS.md`.
The historical counterexamples describe rejected behavior. The finite models
are useful checks, not proof of the Rust implementation. In particular, use
the exact lexicographic ranking in `spec/App server spec.md`; the selection
model's effective-rank arithmetic is an abstraction of a narrower scenario.

Do not add or rewrite a README. Put developer guidance here, command grammar
in command help, and focused design or verification decisions in named notes
or tickets. If Python is ever needed, manage its environment, packages, and
execution with `uv`.

## Roadmap and tickets

Use `tk` and keep `.tickets/` in Git. Tickets must state background, numbered
acceptance criteria, optional methods, future work linked to actual tickets,
and definitions. Put meaningful regression coverage in the same change as
the implementation it validates.

Use task branches for planning and implementation commits. This origin does
not permit direct pushes to `main`; the initial roadmap branch is
`chore/roadmap-init`.

If `gh` is unauthenticated or the GitHub connector returns 404 while Git works,
use `git credential fill` for HTTPS `github.com`, path
`engimatic-systems/exomemory.git`, to obtain the configured Git credential.
Capture it in memory and pass its password as `GH_TOKEN` only to the `gh`
subprocess. Do not print or persist the credential; no new login is needed
when this existing credential works.

Start with `tk ready`. Run discovery as early as its real prerequisites allow;
the first investigation is [exo-a6l7](.tickets/exo-a6l7.md), priority 0 with no
dependencies. The Rust foundation [exo-sjlw](.tickets/exo-sjlw.md) is also ready.
The [zvec integration discovery](.tickets/exo-pw6d.md) is also priority 0 with
no dependencies, even though its implementation comes later.
New discoveries need a bounded question and a concrete decision or probe;
do not postpone them to the integration phase by default.

The reviewable increments are:

1. [M1: create, surface a ghost, and read](.tickets/exo-5iw6.md), using the
   running daemon, exoctl, SQLite, and a controlled Rust harness.
2. [M2: conditional updates and feedback](.tickets/exo-190g.md), demonstrated
   across concurrent test sessions.
3. [M3: real Codex acceptance](.tickets/exo-fq27.md), including exact live
   presence reporting, failure recovery, and verification evidence.

[Remaining specified backends and workflows](.tickets/exo-fsde.md) have
concrete dependent tickets. Non-SQLite implementation follows SQLite
acceptance; SQLite-only extensions can start after their actual M1 or M2
prerequisites. [The spec coverage map](verification/SPEC-COVERAGE.md) records
the owner of each requirement and all seven use cases. An epic's parent
relationship only groups work;
`tk dep <ticket> <prerequisite>` is the actual execution dependency. Milestone
epics depend on their acceptance tickets. Close them after their criteria and
dependencies are satisfied so downstream work becomes ready. Check
`tk dep cycle` after changing the dependency graph.

## Test isolation and generated files

Use `tests/` as the test folder. Every test or manual experiment gets a unique
run directory under `tests/.runtime/`. Store its database, SQLite journal
files, socket, logs, harness state, and memory scope roots there. A test
memory's directory scope must stay inside the run's permitted root. Reject
global scope in the test profile. Keep small reusable inputs in
`tests/fixtures/`; keep optional large outputs in ignored `tests/artifacts/`.

Pass explicit runtime and scope configuration to both client and daemon.
Never fall back to a user's normal memory store or install global hooks for
a test. Enforce scope on create, overwrite, read, metadata, feedback, and
candidate retrieval. Test canonical directory boundaries, parent traversal,
sibling-prefix confusion, symlink escape, and requests outside the scope.
Do not silently relocate a run outside `tests/` if setup fails.

Build output, coverage, disposable databases, large traces, and model-check
downloads/state stay ignored. Keep `.tickets/`, `Cargo.lock`, migrations,
small fixtures, and verification sources version-controlled. Inspect
`git status --short` before committing generated results.

## Contracts to preserve in each increment

- Publish complete revisions atomically; never advertise incomplete data.
- Use compare-and-swap for overwrite and report stale revisions explicitly.
- Select deterministically with the specified ordering and per-source limits.
- Consult authoritative live context for deduplication and compaction;
  cooldown is only rate limiting.
- Bound final serialized ghost size, live count, and wrapper overhead.
- Keep escaped retrieved fields as untrusted data with provenance; return
  complete records through tool output.

The controlled harness must exercise these contracts from M1. Real Codex
support remains unverified until the discovery and adapter tickets supply
version-specific evidence; unavailable context reporting is a tracked gap.

Once the Rust workspace exists, run formatting, linting, and relevant tests
using its recorded commands. The existing model checks run with
`./verification/run-models.sh`; expected counterexamples must still fail for
their intended invariants and the finalized configurations must pass.
Add concrete Rust and demo commands here when their tickets deliver them.
