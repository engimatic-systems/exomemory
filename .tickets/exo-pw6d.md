---
id: exo-pw6d
status: open
deps: []
links: []
created: 2026-09-19T03:49:33Z
type: task
priority: 0
assignee: coopwilliams
parent: exo-fsde
tags: [discovery, zvec, post-sqlite]
---
# Discover a Rust zvec integration that can reproduce snapshot-bound results

## Background in Plain English — WHY

The Data Service spec names zvec but does not define a Rust binding, embedding source, or indexed-snapshot lifecycle. The finalized candidate contract requires identical ordered results for identical indexed state and query parameters. Resolve these questions before the later backend implementation starts. Sources: spec/data service.md, Deterministic candidate contract and Initial backends.

## Outcomes / Acceptance Criteria — WHAT

1. A short decision identifies the intended zvec project, tested version, supported Rust integration route, and authoritative documentation or source evidence.
2. A bounded probe with fixture vectors establishes how an indexed snapshot is identified, held stable, queried repeatedly, and tied to the full data revision.
3. The decision defines query-vector provenance, embedding or supplied-vector input, similarity metric, threshold semantics, normalization, and deterministic tie-breaking; no unselected external embedding service is assumed.
4. The decision explains how committed data can be read while pending external revisions remain invisible, including the immutable-reference or equivalent strategy needed by exo-3m9v.
5. Any capability gap becomes a concrete prerequisite of exo-upbn, with evidence and a proposed resolution; unavailable repeatability is not reported as supported.
6. The investigation has no implementation dependencies and uses only synthetic data and disposable artifacts under tests/; any Python probe uses uv.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use primary documentation and a tiny repeated-query experiment. Record the chosen route in this ticket or a focused decision note; an in-process Rust library or a small explicit boundary may be suitable if evidence supports it.

## Future work

exo-upbn implements the chosen route. exo-3m9v uses the external revision requirements, and exo-1izl checks reproducibility across backend results.

## Definitions

zvec: the vector-search backend named in the spec, whose precise integration must be established here. Embedding: a numeric representation used for similarity search. Snapshot: a fixed indexed view. Metric: the rule for measuring similarity. Provenance: the source and version of data or an embedding. Immutable reference: a reference to data that is not changed in place.
