# Exomemory development

## Scope and source of truth

Implement in Rust. The first storage plugin is SQLite using
[rusqlite](https://github.com/rusqlite/rusqlite); an internal Rust interface with
a compiled-in plugin is sufficient. Keep all canonical state in SQLite
transactions for this slice.

Read the relevant files in `spec/` before implementing a ticket. The finalized
contracts are recorded in `verification/DECISIONS.md` and `verification/RESULTS.md`.
The historical counterexamples describe behavior rejected by the original contract;
the approved best-effort attempt policy supersedes their exact-presence requirements. The finite models
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
3. [M3: real Codex acceptance](.tickets/exo-fq27.md), including best-effort delivery and
   completed-compaction reset, failure recovery, and verification evidence.

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
- Atomically mark preview attempts before sending once; suppress repeated keys
  until completed compaction clears that context. Do not retry failed delivery.
- Bound final serialized ghost size, attempts per compaction interval, and wrapper overhead.
- Keep escaped retrieved fields as untrusted data with provenance; return
  complete records through tool output.

The controlled harness exercises these contracts from M1. Codex 0.155.1 has
version-specific hook, delivery and completed-compaction evidence in
`verification/CODEX-HOOKS.md` and `verification/CODEX-ACCEPTANCE.md`. Other host
versions need fresh verification; exact live presence is not an acceptance claim.

Once the Rust workspace exists, run formatting, linting, and relevant tests
using its recorded commands. The existing model checks run with
`./verification/run-models.sh`; expected counterexamples must still fail for
their intended invariants and the finalized configurations must pass.
Add concrete Rust and demo commands here when their tickets deliver them.

## Rust commands and local boundaries

Run `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace` from the repository root. Run artifacts are retained
in unique `tests/.runtime/r*` directories for inspection. The test helper
creates scope, log, and harness directories there; no fallback is allowed.
Discovery probes have separate manifests under `tests/probes/` and are not
registered application backends. Their commands are in
`verification/CODEX-INTEGRATION.md` and `verification/ZVEC-SNAPSHOTS.md`.

Use GPT 5.6 Luna (`gpt-5.6-luna`) for live Codex tests. Clearly distinguish
scripted local-provider protocol probes from tests using actual model inference.

The first profile requires an explicit existing runtime directory, permitted
scope root and tenant. Paths must be absolute and contain no parent traversal;
resolved directory boundaries govern access. Tests own their runtime trees;
concurrent hostile filesystem mutation is outside this local controlled
profile. The daemon must bind requests to its configured tenant and apply
this same policy at each operation, including metadata and candidates.

The process-level service check is `cargo test --test process`. It launches
`exod`, uses `exoctl` for creation/read/metadata, restarts the daemon, and checks
concurrent clients and direct protocol rejection. `cargo run --bin exoctl --
help` and `cargo run --bin exod -- help` document explicit configuration,
JSON draft input, response envelopes and exit statuses. SQLite busy handling
is bounded to two seconds and local socket I/O to five seconds. A disconnected
write has an uncertain result; the client must not assume it rolled back.

The complete controlled-harness M1 demo is
`cargo test --test m1 -- --nocapture`. This single command starts an isolated
SQLite daemon, creates the test-duration fixture through exoctl, queries
candidates over its socket, accepts a bounded ghost, reads full data and
metadata through exoctl, suppresses another attempt after cooldown, resets
on completed compaction, and surfaces it again. It also sends direct outside-scope requests.
The printed run directory holds daemon logs, database/journals and
`harness/m1-evidence.json`; attempted keys and logical events are canonical in
`harness/context.sqlite`. This is a controlled receiver, not real Codex acceptance.
Events are positive, strictly increasing per session; child contexts share
that clock and cooldown history but have separate attempt sets. Failed sends
remain marked. Completion resets only the affected context, without predicting
which previews Codex retained. See `verification/ATTEMPT-SUPPRESSION.md`.

The opt-in real-host check is
`cargo test --test codex_live -- --ignored --nocapture`. It uses installed Codex
0.155.1 and GPT 5.6 Luna with existing authentication copied privately into the
isolated run and removed afterward. It verifies unsolicited preview delivery,
exoctl full reads, attempt suppression, PostCompact reset, and resurfacing.
`verification/CODEX-ACCEPTANCE.md` records configuration, evidence, and limits.
`cargo test --test codex_adapter` runs the offline adapter regression tests.

The M2 concurrent-session demo is `cargo test --test m2 -- --nocapture`.
It uses exod/exoctl to race two updates, return current metadata to the stale
writer, reconcile deliberately, apply feedback, and restart the daemon. Its
`harness/m2-evidence.json` includes before/after snapshots for deterministic
selection replay. Focused checks are `cargo test --test overwrite --test feedback`.

Recovery checks run with `cargo test --test recovery -- --nocapture`.
They kill test workers at publication/attempt commit boundaries, reopen through
the real daemon, drop an acknowledged write response, and check mixed clients,
bounded SQLite contention, version exhaustion and endpoint handling. See
`verification/SQLITE-RECOVERY.md` for fault schedules and the difference between
SQLite busy timeout and aggregate queued-request latency.

The SQLite acceptance gate is `./verification/check-sqlite.sh`, which runs
formatting, clippy, offline tests and eleven model-check expectations.
The proposed CI workflow is `verification/sqlite-workflow.yml`; activation is
pending GitHub workflow-write permission, as recorded in the acceptance note. The real
Codex and scripted-provider probes remain explicit host-dependent commands.
`verification/SQLITE-ACCEPTANCE.md` maps each contract to executable evidence and
records tested versions, artifact locations and the remaining product scope.

The user-authorized personal trial has a separate explicit `--profile local`;
all automated tests still use their own directories under `tests/.runtime/`.
Default command configuration remains the isolated test profile. Local mode
requires a private existing runtime and an explicit permitted directory root;
it enables directory scopes only. `verification/LOCAL-INSTALL.md` records the
macOS installer, installed commands, reversal and host-version limit. Run
`cargo test --test install` for isolated installation/scope regressions and
`cargo test --test codex_live installed_hooks -- --ignored --nocapture` for the
installed trust/CLI path with GPT 5.6 Luna. Never point these tests at the personal
memory store or install their hooks globally.
