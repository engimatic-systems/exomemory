---
id: exo-a81g
status: open
deps: [exo-bios, exo-dcwx, exo-fr7t]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-fq27
tags: [m3, verification, ci]
---
# Gate SQLite acceptance with invariant traceability and repeatable checks

## Background in Plain English — WHY

The finite models found useful failure cases, but they do not prove Rust code, filesystem isolation, or actual Codex behavior. Acceptance needs a clear mapping from each contract to executable evidence. Sources: verification/RESULTS.md, verification/DECISIONS.md, and active spec/ documents.

## Outcomes / Acceptance Criteria — WHAT

1. A compact verification matrix maps atomic create, same-version body and ghost, stale-write rejection, attempt suppression, post-compaction eligibility, authority boundaries, count and byte bounds, deterministic selection, and source diversity to named Rust tests.
2. The matrix separately maps directory containment, feedback attribution, daemon recovery, and actual Codex delivery and completed-compaction reset to implementation or integration evidence.
3. Automated checks run Rust formatting, linting, component and integration tests, plus verification/run-models.sh with its expected counterexamples and passing finalized models.
4. The actual prose lexicographic order is explicitly tested; the narrower effective-rank abstraction in GhostSelection.tla is documented in a non-README note or the matrix.
5. A reviewer can repeat both milestone demos and the real-harness test using recorded commands and versions; any host-dependent manual check is distinguished from automated coverage.
6. Generated databases, journals, build output, model-check state, large traces, and run artifacts stay ignored, while tickets, small fixtures, and verification source remain versionable.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use existing tests first and add only missing meaningful checks. Record successful model-check expectations, not a claim of proof about all Rust executions. Put commands in AGENTS.md, command help, or a dedicated verification note; add no README.

## Future work

exo-fsde opens expansion only after this acceptance gate. No other future work is identified for the initial SQLite contract.

## Definitions

Invariant: a property required to remain true throughout execution. TLA+: the specification language used by the models. TLC: the program that checks the finite models. CI: automated continuous integration checks. Traceability matrix: a table linking each requirement to its verification evidence.

## Notes

**2026-09-19T15:24:36Z**

Acceptance matrix and reusable check script added, with GitHub Actions running the same offline gate. Formatting, clippy, 38 offline entries and all eleven TLC expectations passed. Final real GPT 5.6 Luna run tests/.runtime/re7c4dec44f76 passed on Codex 0.155.1 with full read, suppression, PostCompact reset, resurfacing and successful hooks; auth copy removed. All dependencies closed, no cycles, generated artifacts ignored. See verification/SQLITE-ACCEPTANCE.md for exact coverage, host-dependent commands and limits.

**2026-09-19T15:25:49Z**

Reopened: GitHub rejected the acceptance branch because the configured PAT lacks workflow scope. The tested gate and exact proposed workflow are reviewable at verification/check-sqlite.sh and verification/sqlite-workflow.yml; the latter is not active. All local checks and real Luna evidence pass, but a maintainer must enable workflow permission or publish the prepared workflow, then inspect CI before closure.
