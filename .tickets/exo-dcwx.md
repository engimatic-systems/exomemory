---
id: exo-dcwx
status: open
deps: [exo-a6l7, exo-5iw6]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-fq27
tags: [m3, codex, harness]
---
# Integrate the SQLite ghost loop with Codex and exact live presence

## Background in Plain English — WHY

The controlled harness must be replaced by the real host for practical agent use. Delivery is valid only when the host can confirm what is in the live model context. Sources: spec/App server spec.md and the decision produced by exo-a6l7.

## Outcomes / Acceptance Criteria — WHAT

1. The adapter uses the version-specific integration path established by exo-a6l7 and normalizes the supported event classes into the shared event contract.
2. Codex receives the trusted interpretation wrapper plus escaped ghost data; exoctl full reads remain tool output.
3. Accepted insertions, retries, rejected delivery, and compaction update authoritative occurrence presence using the same correctness contract as the controlled harness.
4. Session and sub-agent context boundaries do not leak presence state or bypass test-directory access restrictions.
5. A real Codex test run inside tests/ demonstrates create, unsolicited ghost delivery, explicit read, live duplicate suppression, compaction, and eligible resurfacing.
6. If the host lacks a required interface, the adapter reports the unsupported mode and remains incomplete; any host work is tracked as a concrete dependency.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Keep the adapter thin and reuse selection, rendering, and scope checks. Activate integration only for the explicit test session and folder. Do not substitute cooldown or transcript history for live presence.

## Future work

exo-a81g includes the real-harness evidence in acceptance. exo-fsde considers additional harnesses after this integration works.

## Definitions

Adapter: a thin translation layer between host events and the application's shared protocol. Unsolicited ghost: a preview surfaced by a matching event without an explicit memory read request.
