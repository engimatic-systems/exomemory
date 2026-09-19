---
id: exo-a6l7
status: closed
deps: []
links: []
created: 2026-09-19T03:37:25Z
type: task
priority: 0
assignee: coopwilliams
parent: exo-fq27
tags: [discovery, codex, m3]
---
# Establish the real Codex event and live-context integration path

## Background in Plain English — WHY

The App Server spec names Codex as the first host, but satisfying it requires evidence about the installed host's event and compaction interfaces. This investigation runs alongside the SQLite slice so integration uncertainty is resolved early. Source: spec/App server spec.md. Run this first, with no implementation prerequisites; any newly discovered investigation must be scheduled as early as its actual prerequisites allow.

## Outcomes / Acceptance Criteria — WHAT

1. A short version-specific integration decision records the installed Codex interface evidence and relevant official documentation links.
2. A capability table covers prompt, tool call and response, permission, session and sub-agent boundaries, compaction, turn completion, ghost delivery, and authoritative live occurrence reporting.
3. A minimal probe confined to tests/ demonstrates supported event and delivery interfaces without installing global hooks or storing global memories.
4. The decision identifies how accepted delivery, live presence, and compaction are observed; if unsupported, it records the exact gap and a concrete adapter or host-change proposal.
5. exo-dcwx is updated with the evidence-backed implementation path and any actual prerequisite tickets; unsupported presence reporting is never declared compliant.
6. Any additional discovery work is filed at the earliest feasible priority and dependency position, with a bounded question and a concrete decision or probe as its outcome.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Inspect the installed harness and use official documentation for the tested version. Store small evidence in a design note or this ticket. The controlled harness remains useful if a host change is needed; mark the real adapter blocked with evidence rather than weaken its contract.

## Future work

exo-dcwx builds the established integration. exo-fq27 cannot close without the real live-context contract.

## Definitions

Capability table: a mapping from each needed behavior to the interface that supplies it. Probe: a minimal experiment checking a specific interface. Host change: a modification to the harness itself when an adapter cannot supply missing behavior.

## Notes

**2026-09-19T04:11:03Z**

Completed version-specific schema inspection and isolated Rust RPC probe on Codex 0.155.1. See verification/CODEX-INTEGRATION.md. cargo fmt and clippy -D warnings passed; repeat probe acknowledged initialization, thread creation, and item injection. exo-7pi0 tracks the unverified host barriers and exact presence interface and is an explicit exo-dcwx prerequisite.
