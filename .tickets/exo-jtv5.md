---
id: exo-jtv5
status: closed
deps: [exo-mkyx, exo-aqd9, exo-h2bg]
links: []
created: 2026-09-19T03:37:24Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-5iw6
tags: [m1, app, ranking]
---
# Select ghosts with the finalized deterministic order and source quotas

## Background in Plain English — WHY

A broad trigger must not fill every slot, and feedback must never introduce random surfacing. The App Server needs the exact ordering from the active prose spec. Sources: Candidate selection in spec/App server spec.md and verification/DECISIONS.md.

## Outcomes / Acceptance Criteria — WHAT

1. Selection reads authoritative remaining live slots, then filters access, scope, trust schema, final size, cooldown, and live-presence eligibility before duplicate collapse.
2. Occurrences sharing a ghost key collapse to the highest-ranked eligible occurrence, with source_id as the stable tie-break.
3. Distinct keys sort lexicographically by relevance band, closest applicable directory before ancestors and global, ranking penalty, backend-local rank, least-recently-shown logical event, and stable ghost key.
4. After duplicate collapse, a source is exactly (backend instance, trigger id); candidates over that source's response quota are skipped while other sources can fill remaining slots.
5. The selected response cannot exceed remaining live count or rendered bounds, and acceptance follows the ledger concurrency contract.
6. Tests vary candidate input order and independently exercise each ranking field, duplicate sources, quotas, zero remaining slots, never-shown values, and rejected high-ranked candidates.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Implement selection as a mostly pure function over explicit event, snapshot, configuration, and ledger inputs. Fix the ordering directions and the never-shown sentinel in code-level documentation. Test the prose's lexicographic ordering directly: GhostSelection.tla uses a narrower effective-rank abstraction.

## Future work

exo-lm9g wires selection into the end-to-end first slice. exo-6g4f exercises live persisted penalty changes. exo-a81g traces the abstract model properties to concrete checks.

## Definitions

Lexicographic order: compare fields in sequence, using the next only when earlier fields tie. Duplicate collapse: keep one occurrence per memory key. Quota: a maximum count per source. Sentinel: a defined special value, here for a memory never shown.

## Notes

**2026-09-19T04:30:51Z**

Implemented exact ascending lexicographic selection: relevance, canonical scope distance, penalty, backend-local rank, least-recently-shown event (never=0), stable key. Eligibility and final rendering precede duplicate collapse; source pairs break duplicate ties and enforce response quotas. Presence version and logical event checks coordinate with harness acceptance. Tests independently oppose later fields for every ranking field, vary input order, and cover quotas, source ties, zero slots, cooldown/live filtering, malformed/oversized candidates, scope and trust schema. Formatting, clippy and selection tests pass.
