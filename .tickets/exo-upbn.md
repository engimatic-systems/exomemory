---
id: exo-upbn
status: open
deps: [exo-3m9v, exo-pw6d]
links: []
created: 2026-09-19T03:49:33Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-sqlite, zvec, backend]
---
# Deliver a deterministic zvec similarity-search backend slice

## Background in Plain English — WHY

The spec requires vector similarity retrieval whose ordered output is reproducible for the same indexed snapshot and query parameters. The early discovery establishes an implementable Rust route; this ticket delivers the full memory loop. Source: spec/data service.md, zvec and Deterministic candidate contract.

## Outcomes / Acceptance Criteria — WHAT

1. The backend uses the version-specific Rust integration established by exo-pw6d and exposes validated vector-trigger parameters through exoctl discovery.
2. Create and conditional overwrite use exo-3m9v so indexed data, referenced full content, and ghost metadata agree on the ready revision.
3. Queries bind to an identified indexed snapshot, normalized query vector, metric, threshold, and relevant search settings; repeated requests return identical candidate ordering with stable ties.
4. Returned candidates map local similarity to documented relevance bands and backend-local rank without comparing raw scores against other backend types.
5. exoctl reads return the referenced full data and actual revision; missing snapshots, missing referenced data, and unavailable search capability produce explicit errors.
6. annoy changes only the canonical ranking penalty; it never changes the similarity threshold, trigger match, or random inclusion behavior.
7. Fixture-based tests demonstrate the complete loop, concurrent publication, restart, and repeatability using local test data; models, indexes, and large artifacts stay ignored under tests/.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Follow the discovery's supported approach and explicitly version any embedding inputs. Use small fixed vectors for correctness evidence; add semantic examples only where they clarify the user workflow.

## Future work

exo-1izl verifies selection across zvec and other backends. No additional vector backend is required by the current spec.

## Definitions

Vector: a list of numbers used to represent an item for similarity matching. Index: the search structure built from stored vectors. Similarity threshold: the boundary for considering a match. Backend-local rank: deterministic order within one backend's result.
