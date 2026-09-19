---
id: exo-aqd9
status: open
deps: [exo-sjlw]
links: []
created: 2026-09-19T03:37:24Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, app, authority]
---
# Render escaped ghost data with hard serialized size limits

## Background in Plain English — WHY

Ghost previews may contain hostile or malformed text. The App Server must preserve the distinction between its trusted format explanation and retrieved data, and bound the final output after escaping. Sources: spec/App server spec.md; NoUntrustedDeveloperContext and WithinRenderedByteBound in verification/.

## Outcomes / Acceptance Criteria — WHAT

1. The renderer accepts typed ghost fields with key, revision, blurb, optional uses and topics, read estimate, source, provenance, and trust; full record bodies are never inserted automatically.
2. The trusted interpretation wrapper describes optional retrieval suggestions, and all candidate-controlled fields are escaped data inside an unambiguous envelope.
3. Malformed and oversized candidates are rejected before selection; no serialized ghost is truncated.
4. The configured hard per-ghost byte limit includes field names, delimiters, escaping, and all attributable overhead; fixed wrapper overhead has a separate maximum.
5. Configuration validates that maximum live count times maximum ghost bytes plus maximum wrapper bytes fits the declared supported envelope without runtime token budgeting.
6. Boundary tests cover quotes, control characters, multibyte text, fake role delimiters, instruction-like blurbs, exact limits, and escaping expansion.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use a structured serializer and inspect its final bytes. Keep wrapper strings static and collect a small set of malicious preview fixtures. Size checks and final rendering should share one implementation.

## Future work

exo-jtv5 uses render eligibility before ranking. exo-dcwx maps the data envelope to the actual Codex delivery schema.

## Definitions

Envelope: the structure that separates retrieved fields from the surrounding explanation. Serialized representation: the exact encoded output sent to the harness. Authority boundary: the distinction between trusted instructions and retrieved data. Token budget: an allowance measured in model text tokens; this design instead uses fixed conservative bounds.
