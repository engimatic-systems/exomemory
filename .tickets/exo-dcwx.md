---
id: exo-dcwx
status: open
deps: [exo-a6l7, exo-5iw6, exo-7pi0]
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 2
assignee: coopwilliams
parent: exo-fq27
tags: [m3, codex, harness]
---
# Integrate the SQLite ghost loop with Codex and best-effort attempts

## Background in Plain English — WHY

The controlled harness must be replaced by the real host for practical agent use. The adapter must attempt each preview once per context between completed compactions and tolerate dropped delivery without retry. Sources: spec/App server spec.md and the decision produced by exo-a6l7.

## Outcomes / Acceptance Criteria — WHAT

1. The adapter uses the version-specific integration path established by exo-a6l7 and normalizes each event class named in the spec: user prompts, tool calls and responses, permission requests, session and sub-agent boundaries, compaction, and turn completion. Events are scanned before the harness continues; unavailable required interfaces remain explicit prerequisites rather than silently omitted event classes.
2. Codex receives the trusted interpretation wrapper plus escaped ghost data; exoctl full reads remain tool output.
3. Selection atomically marks attempts before sending once. Failed sends remain suppressed; completed compaction clears only that context, with sends and reset handling serialized.
4. Session and sub-agent context boundaries do not leak attempt state or bypass test-directory access restrictions.
5. A real Codex test run inside tests/ demonstrates create, unsolicited ghost delivery, explicit read, repeated-attempt suppression, compaction, and eligible resurfacing.
6. If the host lacks a required interface, the adapter reports the unsupported mode and remains incomplete; any host work is tracked as a concrete dependency.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Keep the adapter thin and reuse selection, rendering, and scope checks. Activate integration only for the explicit test session and folder. Do not retry uncertain preview delivery or infer acceptance from transcript history.

## Future work

exo-a81g includes the real-harness evidence in SQLite acceptance. exo-1izl reuses that evidence for full-spec acceptance; the spec does not name another required harness.

## Definitions

Adapter: a thin translation layer between host events and the application's shared protocol. Unsolicited ghost: a preview surfaced by a matching event without an explicit memory read request.

## Version-specific implementation evidence

The earlier 0.155.1 stdio probe verifies item injection only. exo-7pi0 verifies
ordinary hooks and completed-compaction reporting under the revised contract in
verification/ATTEMPT-SUPPRESSION.md. No host fork or exact-presence API is required.
