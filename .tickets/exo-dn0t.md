---
id: exo-dn0t
status: open
deps: [exo-aqd9, exo-jtv5]
links: []
created: 2026-09-19T20:07:26Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-9tjp
tags: [ghost, wire-format]
---
# Remove provenance and source identifiers from rendered ghosts

## Background in Plain English — WHY

Rendered ghosts currently expose `provenance` and `source_id` to the model on every recall. They describe where a candidate came from, not the memory preview itself, and consume context on the common path. An agent that needs origin details can request metadata through `exoctl meta`.

The app still needs source identity internally. Candidate selection uses `(backend instance, trigger id)` for per-source limits and as a stable tie-break, so this ticket changes the injected representation rather than deleting source tracking from storage or selection.

## Outcomes / Acceptance Criteria — WHAT

1. Every rendered or injected ghost omits the `provenance` and `source_id` keys, including ghosts containing hostile text that resembles those keys.
2. Stored records and `exoctl meta` retain the provenance, backend, and trigger definitions needed to inspect a memory's origin.
3. Candidate selection retains its internal source identity, deterministic duplicate tie-break, and per-source response quota exactly as specified.
4. The final serialized-size checks and fixed wrapper description are updated for the smaller ghost shape; full record bodies remain excluded from automatic injection.
5. `spec/overview.md`, `spec/App server spec.md`, affected verification notes, and command examples describe the same public ghost fields.
6. Meaningful renderer, selection, delivery, and installed-path regressions assert both the missing public keys and the unchanged internal source behavior.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Separate the internal candidate type from the serialized ghost type so a future renderer cannot expose source fields accidentally. Test the serialized value rather than relying only on Rust field visibility.

## Future work

`exo-v7n1` embeds the revised overview in full help. `exo-0kao` verifies the final ghost shape in an installed GPT 5.6 Luna session.

## Definitions

Provenance: stored information about where a memory came from. Source identity: the backend instance and trigger that produced a candidate. Rendered ghost: the bounded preview sent to the model. Metadata: the complete origin and trigger information returned on explicit request.
