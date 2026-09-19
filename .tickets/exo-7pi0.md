---
id: exo-7pi0
status: in_progress
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
2. A synthetic preview reaches actual model input through a documented host path.
3. Completed-compaction notification or hook resets only its context; start or
   failed compaction does not reset. No exact survivor list is required.
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
