---
id: exo-7pi0
status: in_progress
deps: [exo-adfe]
links: []
created: 2026-09-19T04:09:30Z
type: task
priority: 0
assignee: coopwilliams
parent: exo-fq27
tags: [codex, host-gap, discovery]
---
# Supply Codex event barriers and authoritative live occurrence snapshots

## Background in Plain English — WHY

The Codex 0.155.1 probe in verification/CODEX-INTEGRATION.md confirms item injection, but neither its response nor the compaction item supplies the exact surviving ghost occurrences. Notification delivery also does not itself pause the host before its next model request. Determine whether synchronous lifecycle hooks plus a host extension can supply both guarantees. This investigation can begin immediately and is a prerequisite of exo-dcwx, not M1.

## Outcomes / Acceptance Criteria — WHAT

1. A bounded isolated probe establishes whether synchronous hooks for every required event can pause the host until selection and insertion finish, including child contexts.
2. A concrete host change or existing supported interface reports a versioned snapshot of live occurrence IDs and keys after insertion, compaction, eviction, fork, and resume; transcript items alone do not qualify.
3. Conditional insertion takes an expected presence version and an idempotency identity, returning accepted occurrences or an explicit rejection before the model continues.
4. A version-specific decision and executable synthetic probe show retained and removed occurrences after compaction, or document the exact remaining host patch needed; exo-dcwx stays incomplete until that patch is verified.

## Methodological Suggestions — HOW

Optional: start from the generated HookEventName and ThreadInjectItems schemas. Put occurrence IDs in typed context items and expose a host-owned live-context snapshot and conditional insertion RPC. Use the same critical section as context mutation. No global hook installation or real memories are needed.

## Future work

exo-dcwx consumes the verified interface. exo-a81g includes its real-host acceptance evidence.

## Definitions

RPC: a request to another process. Event barrier: a point where the host waits before continuing. Eviction: removal from live model input. Idempotency identity: a stable request ID that prevents a retry from inserting twice.

## Notes

**2026-09-19T14:21:52Z**

Expanded the pinned 0.155.1 Rust probe: injection accepts unsupported condition and delivery identity fields, including a repeated identity, with empty acknowledgments. Current compaction items expose only id/type. See verification/CODEX-HOST-CONTRACT.md for evidence, limits, and the concrete extension. All-event barriers and compaction survivors remain unverified, so this ticket stays incomplete. exo-adfe tracks the host implementation; a human must choose a maintained project build or an upstream dependency.
