---
id: exo-lm9g
status: closed
deps: [exo-ljge, exo-jtv5]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, demo, acceptance]
---
# Demonstrate create, ghost delivery, retrieval, and compaction end to end

## Background in Plain English — WHY

This is the first reviewable product increment: an agent can save a useful memory, see its preview on a later event, and pull the complete record. The implementation must exercise the running service and command-line client rather than only internal functions. Sources: spec/spec - use cases to support.md.

## Outcomes / Acceptance Criteria — WHAT

1. One documented command in AGENTS.md or test help starts an isolated Data Service and drives the controlled Rust harness through create, matching event, ghost acceptance, exoctl read, and metadata read.
2. The example uses a concrete repository memory such as test-suite duration and verifies key, provenance, and the returned revision.
3. The scenario suppresses a second live occurrence, compacts away the first, and surfaces the key again when eligible.
4. An outside-directory event and direct outside-scope read return no test memory data; all created scopes, databases, logs, and harness state stay beneath tests/.
5. Repeating the scenario from the same recorded inputs produces the same candidate selection and order; generated run artifacts are ignored by Git.
6. The first-slice demonstration and component checks pass, and their commands are recorded without adding a README.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use the compiled exoctl and daemon binaries from Rust integration tests. Keep small fixture events in tests/fixtures/ and disposable output in tests/.runtime/. Record concise review evidence in the ticket when closing it.

## Future work

exo-hnil and exo-6g4f extend this demonstrated slice. exo-dcwx substitutes a real Codex harness for controlled delivery.

## Definitions

End to end: using the actual client, service, storage, and delivery path together. Fixture: a small committed input used to reproduce a test.

## Notes

**2026-09-19T04:34:50Z**

All six criteria verified in verification/M1-ACCEPTANCE.md. cargo test --test m1 -- --nocapture starts the actual daemon and CLI, checks create/ghost/read/meta, suppresses live duplicates, compacts and resurfaces, rejects direct outside-scope requests, and replays the recorded event sequence identically in a fresh controlled context. Full workspace: 18 integration tests pass. Formatting and clippy -D warnings pass; all ten model-check configurations match their expected outcomes.
