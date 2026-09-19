---
id: exo-1izl
status: open
deps: [exo-03m1, exo-upbn, exo-jku1, exo-52xr, exo-hlw6, exo-t3ru]
links: []
created: 2026-09-19T03:49:34Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-sqlite, acceptance, verification]
---
# Verify all named backends and use cases against the complete spec

## Background in Plain English — WHY

The SQLite acceptance gate covers the first slice only. The remaining spec includes external publication, several backend types, global policy, and additional user workflows. A separate gate makes full coverage reviewable without expanding the first milestone. Sources: every document under spec/ and verification/DECISIONS.md.

## Outcomes / Acceptance Criteria — WHAT

1. The coverage map links every specified operation, backend, selection and publication contract, and seven use cases to implementation tickets and named verification evidence.
2. Combined SQLite, Redis, zvec, episodic-SQLite, and retrieval-only fixtures preserve deterministic ordering, scope specificity, duplicate collapse, source-pair quotas, feedback behavior, and final rendering bounds.
3. Mixed-backend reproduction records each relevant snapshot, revision, feedback state, and event input; concurrent backend changes do not expose pending or mixed-version content.
4. External failure and restart checks demonstrate that unconfirmed revisions never become ghosts or successful reads and that stale writers cannot publish.
5. Command help and list-trigger-types match actual registered capabilities; unsupported operations, body-free registration, and temporal reads follow their documented contracts. Question-note workflows retrieve a file reference and use ordinary file tools without introducing an exoctl append operation.
6. All seven use cases have repeatable evidence, including the existing test-duration scenario exo-bios and the new workflow tickets; global semantics use synthetic policy fixtures while live memories stay test-directory scoped.
7. The Rust checks and existing model expectations pass; the evidence distinguishes finite-model safety, actual integration behavior, and any manual product evaluation.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Extend the SQLite verification matrix and reuse each backend's focused tests. Use deterministic fixture snapshots for exact comparisons and isolated real backend services for integration checks. Keep large data and traces ignored.

## Future work

No further work is required to cover the currently named spec features. Additional harnesses, operational packaging, and production rollout require separately chosen scope because the spec does not identify their concrete targets.

## Definitions

Coverage map: a mapping from each requirement to the ticket and check that establish it. Mixed-backend reproduction: repeating selection using the same recorded state from several backend instances. Publication: making a complete confirmed revision visible. Capability: an explicitly supported operation.
