# SQLite M3 acceptance

This gate covers the local SQLite slice under the user-approved best-effort
preview contract. Each context records attempted keys before sending once,
suppresses them until completed compaction, and does not retry uncertain
preview delivery. Exact live presence, survivor prediction and acknowledgment
receipts are not required. Compaction may retain a preview that is sent again.

## Repeatable checks

```sh
./verification/check-sqlite.sh
cargo test --test m1 -- --nocapture
cargo test --test m2 -- --nocapture
cargo test --test recovery -- --nocapture
cargo test --test codex_live -- --ignored --nocapture
cargo run --manifest-path tests/probes/codex/Cargo.toml --target-dir target/probes -- --hooks
```

The first command runs formatting, warnings-denied clippy, the locked workspace
tests, and every finite-model expectation. `.github/workflows/sqlite.yml`
runs that same command on pushes and pull requests; the reviewed source copy is
`verification/sqlite-workflow.yml`. The two demos and recovery suite
are already included in workspace tests; their separate commands print artifact
locations. The real-host test is opt-in and uses existing credentials and real
GPT 5.6 Luna calls. The final command uses a scripted local provider for host
protocol evidence; it is not model inference and requires installed Codex.
Neither host-dependent command is part of credential-free CI.

Tested locally on macOS arm64 with Rust/Cargo 1.98.0, bundled rusqlite 0.40.2,
Codex CLI 0.155.1, `gpt-5.6-luna`, and TLC 2.19 from TLA+ tools v1.7.4.
The CI workflow pins Rust 1.98.0 and Java 17 on Linux. Local and remote results
are recorded separately below.

## Executable traceability

Test names below belong to `tests/`; run a file with `cargo test --test NAME`.
The contract is the revised prose in `spec/` and `DECISIONS.md`. Historical
counterexamples establish rejected choices, not a proof about Rust executions.

| Contract | Concrete executable evidence |
| --- | --- |
| Complete atomic creation; no dangling trigger | `storage::failure_after_body_insert_rolls_back_every_canonical_table`; `candidates::normalization_and_rollback_never_advertise_partial_records`; recovery commit barriers before/after create |
| Same-version body and preview under replacement | `overwrite::concurrent_cas_has_one_winner_and_only_complete_revisions`; recovery commit barriers before/after overwrite; M1/M2 compare actual reads and previews |
| Stale writers cannot commit | The concurrent overwrite test; `overwrite::cli_requires_revision_and_returns_current_metadata_on_conflict`; M2 explicitly reads and reconciles the conflict |
| Overwrite authorization and complete replacement | `overwrite::invalid_unauthorized_and_failed_publication_preserve_the_old_revision`; validates immutable records, old/replacement scope, tenancy and rollback |
| Attempt-before-send and persistent suppression | `attempts::abandoned_send_stays_suppressed_across_restart_until_completed_compaction`; `recovery::kill_after_attempt_commit_never_retries_on_process_restart` |
| Completed-only reset, replay protection and child isolation | `attempts::reset_is_context_local_and_cooldown_and_session_order_survive`; abandoned-send test rejects old completion after a new attempt; adapter reset mapping; host root/child PostCompact probe |
| Count cap and simultaneous attempts | `attempts::concurrent_attempts_cannot_duplicate_or_exceed_the_interval_cap`; selection rejects overfull/stale state |
| Serialized ghost and wrapper byte bounds | `rendering::escaped_untrusted_fields_round_trip_and_exact_serialized_limit`; `rendering::interval_bound_includes_a_wrapper_for_every_possible_send`; `codex_adapter::host_framing_limit_is_checked_before_marking_an_attempt` |
| Retrieved text remains escaped untrusted data | All `rendering` tests; `codex_adapter::connection_loss_after_insertion_is_not_retried_and_ghost_stays_untrusted`; live test retrieves full body through exoctl tool output |
| Exact deterministic ranking and stable tie-breaks | `selection::every_lexicographic_field_and_never_shown_sentinel_have_specified_priority`; permuted candidate lists and source tie-break in the eligibility test; repeated snapshot queries in `candidates` |
| Eligibility, duplicate collapse and source quotas | `selection::eligibility_precedes_duplicate_collapse_and_source_quotas`; M2 checks one selected preview for two memories sharing a source |
| Canonical directory containment and tenancy | Both `isolation` tests cover traversal, sibling prefixes, symlinks and global rejection; `process::direct_socket_requests_cannot_bypass_scope_or_protocol_checks`; M1 direct read/meta/candidate rejection; overwrite/feedback authorization tests |
| Attributable, bounded feedback independent of body revisions | `feedback::concurrent_feedback_is_attributed_and_saturates_without_overflow`; `feedback::feedback_changes_rank_not_matching_and_survives_overwrite_and_restart`; schema migration and CLI arbitrary-penalty rejection |
| Snapshot-based selection replay across restart | The feedback ranking test and M2 save before/after snapshots and reproduce selections; candidate snapshots carry body revision, feedback version and global store version |
| Process interruption, uncertainty and contention | All `recovery` tests: killed workers, committed response loss, stale retry, mixed clients, bounded busy errors, exhausted snapshot counter, endpoint preservation and restart |
| Real unsolicited Codex preview, full read, suppression and resurfacing | `codex_live::real_luna_reads_unsolicited_preview_then_suppresses_and_resurfaces_after_compaction`; actual GPT 5.6 Luna calls, successful observed hooks and archived thread |
| All eleven host event classes, synchronous barrier, permission insertion and aborted/root/child compaction | `tests/probes/codex/src/hooks.rs --hooks` with local scripted provider; `codex_adapter::hook_mapping_keeps_children_separate_and_rejects_unknown_or_outside_context`; evidence in `CODEX-HOOKS.md` |

The stale-event check applies to normalized logical events. The host adapter
runs each configured hook once, allocates its number under the SQLite/OS lock,
and does not replay hooks on reconnect. Adding an external command retry layer
would change this contract, especially for PostCompact, and is unsupported.
The attempt set is exact key membership; it is not a probabilistic Bloom filter.

## Model scope

`run-models.sh` checks eleven configurations: eight historical counterexamples
must still violate their named invariants, and the hardened core, finalized
selection, and new attempt-suppression configurations must pass. Historical
exact-presence configurations retain their original meaning and are superseded
for current delivery acceptance. `AttemptSuppression.tla` covers reservation,
reset, crash/restart and stale events in a finite state space.

`GhostSelection.tla` uses effective-rank arithmetic for a narrower abstraction.
Rust must not copy that arithmetic. Its order is lexicographic: relevance band,
scope distance, penalty, backend-local rank, least-recent attempt, stable key;
source is a final duplicate tie-break. The named selection regression opposes
all later fields to prove each earlier field takes precedence. Model success
is supporting evidence, not a proof that all implementation schedules are safe.

## Evidence and boundaries

Each test owns a unique `tests/.runtime/` directory. M1 exports
`harness/m1-evidence.json`; M2 exports `harness/m2-evidence.json`; live Codex
exports `harness/codex-evidence.json` plus RPC/log traces. Commit-barrier recovery
exports `harness/recovery-evidence.json`. SQLite databases and journals remain
there. Live credentials are private temporary copies removed on completion;
they are never committed or uploaded as CI artifacts.

The supported slice is SQLite/local, regex triggers, directory scope in an
explicit test profile, local client-supplied session attribution, and Codex
0.155.1. The separate personal trial has a reversible local installer, but no
general production installer. This slice has no hostile-filesystem guarantee, general
global scope, remote authentication, external backend publication or other host
version claim. Broader specified backends and workflows remain in exo-fsde and
its linked tickets. Preview count/byte limits constrain new attempts between
compactions; they do not constrain what an unpredictable summary retains.

## Recorded local result — 2026-09-19

The gate passed formatting, warnings-denied clippy, all 38 offline test entries
(including the crash-worker helper), and all eleven model expectations. The
explicit real Luna test passed in `tests/.runtime/re7c4dec44f76` against the
completed update/feedback/recovery implementation. Its random memory key was
read through exoctl, its attempt stayed unchanged on the next turn, and its
attempt number advanced after completed compaction. All observed hook handlers
completed successfully and the thread was archived. The private authentication
copy was confirmed removed. Two earlier real Luna runs passed during adapter
implementation; scripted all-event evidence is recorded in `CODEX-HOOKS.md`.
A subsequent test-helper cleanup guard also covers failure during host startup;
formatting and clippy passed after that change.

`tk dep cycle` reported no dependency cycles. Git ignore checks confirmed that
runtime evidence, build output, TLC downloads/state and optional large artifacts
are ignored. Source, migrations, lockfiles, tickets and this verification map
remain versioned. These are bounded local results, not full-spec acceptance.
The broader exo-fsde expansion backlog remains open.

## CI activation — 2026-10-05

An earlier attempt to publish `.github/workflows/sqlite.yml` was rejected because
the configured Personal Access Token lacked `workflow` scope. A maintainer
published the file through GitHub on branch `test/premerge-interface-acceptance`
at commit `32b10bbb257ae2e3f61b02153bec57a4144a06f1`. It is byte-for-byte
identical to `verification/sqlite-workflow.yml`. The maintainer reported that the
resulting [SQLite acceptance run](https://github.com/engimatic-systems/exomemory/actions/runs/37335330379)
succeeded for this commit. The configured
command-line token still receives 403 for the Actions API, and the GitHub
connector receives 404, so this local record does not independently assert
individual remote job log contents.

The accepted M3 result combines this maintainer-confirmed CI run with the
recorded local gate, process recovery, and real Codex 0.155.1/GPT 5.6 Luna
inference. The workflow is presently on the unmerged acceptance branch and
will reach `main` when the review stack is merged.
