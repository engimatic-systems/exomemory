# M1 controlled SQLite acceptance

Updated delivery contract: `cargo test --test m1 -- --nocapture` now checks
attempt-before-send, suppression, completed-compaction reset, and resurfacing.
Older occurrence/receipt descriptions below record the original M1 run and
are superseded by [attempt suppression](ATTEMPT-SUPPRESSION.md).

Verified 2026-09-19 on macOS arm64, Rust 1.98.0, bundled rusqlite 0.40.2.
This establishes exo-5iw6 and exo-lm9g with the controlled Rust host. It does
not establish M2 overwrite/feedback or real Codex M3 acceptance.

## Reproduce

From the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
./verification/run-models.sh
```

The standalone demonstration is `cargo test --test m1 -- --nocapture`.
It uses compiled exod/exoctl binaries and bundled SQLite, with the committed
`tests/fixtures/m1.json` test-duration input. The test prints a unique run
beneath tests/.runtime/ containing database/journal files, socket, daemon logs,
and the canonical controlled context database. Its human-inspectable export
is `harness/m1-evidence.json`.

The final checked demo run was `tests/.runtime/rd18eacdf461d`.
Disposable paths are evidence from this checkout, not required committed
fixtures. The executable test regenerates them without personal stores.

## Milestone criteria

| Criterion | Inspected executable evidence |
| --- | --- |
| exo-5iw6.1: create, matching event, accepted ghost, full read | `tests/m1.rs` creates via exoctl, queries the daemon socket, selects/accepts into the actual controlled live context, and retrieves through exoctl |
| exo-5iw6.2: directory isolation and outside event | `tests/isolation.rs`, `tests/process.rs`, `tests/m1.rs`; absolute canonical roots, rejected global scope/traversal/sibling prefixes/symlink escape, and direct socket read/meta/event rejection |
| exo-5iw6.3: atomic complete revisions | `tests/storage.rs` injects a failure after body insertion and asserts all canonical tables rolled back; committed records survive reopening |
| exo-5iw6.3: revision agreement and deterministic candidates | `tests/candidates.rs` checks repeated snapshot queries, explicit duplicates, scope, invalid triggers, normalization and failed publication; the demo compares candidate key/revision/provenance with the complete returned record while asserting that delivered previews omit internal provenance and source identity |
| exo-5iw6.3: exact ordering and source quotas | `tests/selection.rs` independently exercises every lexicographic field against opposing later fields, input order changes, duplicate tie-breaking, quota skip, zero slots and eligibility filtering |
| exo-5iw6.3: serialized bounds and authority | `tests/rendering.rs` checks final byte boundaries, escaping expansion, controls/multibyte/hostile delimiters, malformed input and envelope arithmetic; the demo checks the actual live byte envelope and separate full-body tool read |
| exo-5iw6.3: authoritative presence | `tests/presence.rs` checks accepted/rejected delivery, concurrent stale views, exact retries, retained/removed compaction, cooldown, child contexts, independent sessions, rollback and reopen |
| exo-5iw6.4: completed dependency graph | `tk dep tree exo-5iw6` and closed tickets exo-sjlw, exo-b12v, exo-ljge, exo-mkyx, exo-aqd9, exo-h2bg, exo-jtv5, exo-lm9g |

## Demonstration criteria

| exo-lm9g criterion | Result |
| --- | --- |
| 1: one documented executable command | AGENTS.md records the standalone demo command; `tests/m1.rs` launches the actual daemon and CLI |
| 2: concrete memory and metadata | Synthetic 12-second test-suite measurement; exact body, stable key, provenance and returned revision asserted |
| 3: suppression, compaction, resurfacing | Event 1 accepts, event 10 suppresses despite elapsed cooldown, event 11 compacts away, event 12 accepts a new occurrence |
| 4: outside access and isolated artifacts | Direct socket reads, metadata and candidate events outside the permitted root return forbidden; CLI rejection is separately tested |
| 5: reproducibility and ignored artifacts | The recorded event sequence replays against the same committed memory state in a fresh controlled context; every selected item and final live byte sequence is identical. Git ignore checks cover runtime evidence and build output |
| 6: component and model checks | All 18 Rust integration tests pass; formatting and clippy with warnings denied pass; all 10 TLC configurations match expectations |

The eight historical configurations still fail for their intended invariants;
the finalized core and selection configurations pass. Those are finite model
results, not a proof of the Rust implementation. The concrete checks above
supply the implementation evidence.

## Review chain and remaining scope

The chain starts from chore/roadmap-init, not main. PRs #2–#10 deliver Codex
and zvec discovery, foundation, SQLite transactions, service/CLI, candidate
queries, rendering, controlled presence and deterministic selection. The final
M1 acceptance PR follows #10.

The daemon registers only SQLite/local. zvec remains an isolated discovery
probe for exo-upbn. exo-hnil and exo-6g4f own conditional overwrite and feedback.
The real Codex adapter remains incomplete: verification/CODEX-INTEGRATION.md
and exo-7pi0 record the missing verified live occurrence and event-barrier
contract required by exo-dcwx. The controlled harness does not substitute for
that acceptance. No user decision was needed to complete the scoped M1 work.
