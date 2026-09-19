---
id: exo-7pi0
status: closed
deps: [exo-0wuy]
links: []
created: 2026-09-19T04:09:30Z
type: task
priority: 0
assignee: coopwilliams
parent: exo-fq27
tags: [codex, host-gap, discovery]
---
# Verify Codex hook delivery and completed-compaction reset

## Background in Plain English — WHY

The user replaced exact live presence with best-effort attempt suppression on
2026-09-19. The missing snapshot and receipt APIs no longer block integration.
We still need executable evidence that the installed host runs our event hooks,
delivers their previews, and signals completed compaction. Sources:
verification/ATTEMPT-SUPPRESSION.md and spec/App server spec.md.

## Outcomes / Acceptance Criteria — WHAT

1. A bounded isolated probe verifies event mapping and synchronous hook behavior
   for prompt, tool call/response, permission, session, child, compaction, and
   turn-completion events, distinguishing runtime evidence from schema evidence.
2. Synthetic context reaches actual model input through a verified host path.
3. The completion hook identifies the affected root or child context and is
   absent after aborted compaction. exo-dcwx wires this signal to the tested
   per-context reset. No exact survivor list is required.
4. The version-specific decision records concurrency and restart limitations,
   commands, evidence artifacts, and any remaining integration prerequisite.

## Methodological Suggestions — HOW

Use ordinary hooks with an isolated Codex home and a controlled local model
endpoint if useful. Do not install global hooks or substitute an event enum
for a runtime check. Other executable probes meeting these outcomes are valid.

## Future work

exo-dcwx consumes the verified path. exo-a81g includes the real-host evidence.

## Definitions

Hook: code the host runs at a lifecycle event. Completed compaction: the host
finished shortening context. Model input: the actual request sent for inference.
Best effort: preview delivery can be dropped without retry.

## Historical investigation

The earlier probe demonstrated that 0.155.1 has no verified exact-presence or
conditional-insertion contract. That result is retained in
verification/CODEX-HOST-CONTRACT.md; the user explicitly removed that requirement.
exo-adfe is superseded, not implemented. This ticket now verifies ordinary hooks.

## Notes

**2026-09-19T14:53:22Z**

Verified Codex 0.155.1 with the isolated --hooks probe. All eleven event types execute synchronously; prompt gate prevents model advance; context markers and insertion during a waiting permission hook reach actual scripted-provider requests. Aborted compaction emits no PostCompact; successful root and child compactions do, with parent-session/child-context identity. Stop output must be {} rather than additionalContext. See verification/CODEX-HOOKS.md for commands, limits, and adapter mapping. Probe formatting, Clippy, and executable checks pass. This establishes the host path, not full application or real-model acceptance.

**2026-09-19T14:53:45Z**

The added child identity assertion exposed ordering in the test: contextCompaction item completion can precede PostCompact execution. Reopened pending a run that waits for hook/completed before inspecting child reset identity; earlier successful runs had observed the hook after waiting for archive.

**2026-09-19T14:54:53Z**

Final probe run tests/.runtime/hooks-71596-1789829625653 passed after waiting for the actual child PostCompact hook. Root/child identities, all hook statuses, model markers, prompt barrier, and aborted-compaction checks now pass together. Close the investigation; exo-dcwx owns application wiring and real Luna inference.
