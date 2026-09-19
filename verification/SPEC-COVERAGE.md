# Spec-to-ticket coverage

This is a planning map, not evidence that the implementation passes these
requirements. All tickets are open when this map is introduced. The SQLite
gate is [exo-a81g](../.tickets/exo-a81g.md); full-spec acceptance is
[exo-1izl](../.tickets/exo-1izl.md). Those tickets will attach executable evidence.

## Service, command, and storage requirements

Sources: `spec/overview.md`, `spec/data service.md`, and
`spec/exoctl-command-spec.md`.

| Spec requirement | Owning tickets |
| --- | --- |
| Rust components and test-directory isolation | [exo-sjlw](../.tickets/exo-sjlw.md) |
| Persistent daemon, concurrent clients, normalized requests | [exo-ljge](../.tickets/exo-ljge.md), [exo-fr7t](../.tickets/exo-fr7t.md) |
| Stable keys, complete revisions, atomic SQLite publication | [exo-b12v](../.tickets/exo-b12v.md) |
| Conditional overwrite, permission checks, explicit stale-write conflicts | [exo-hnil](../.tickets/exo-hnil.md) |
| Attributable versioned feedback and bounded deterministic penalties | [exo-6g4f](../.tickets/exo-6g4f.md) |
| Help, stable grammar and exit statuses, structured errors | [exo-ljge](../.tickets/exo-ljge.md), [exo-hnil](../.tickets/exo-hnil.md), [exo-6g4f](../.tickets/exo-6g4f.md) |
| Full reads and complete metadata with actual revision | [exo-ljge](../.tickets/exo-ljge.md) and each backend slice below |
| Trigger discovery and parameter validation | [exo-ljge](../.tickets/exo-ljge.md), [exo-mkyx](../.tickets/exo-mkyx.md), [exo-reie](../.tickets/exo-reie.md), [exo-u1mp](../.tickets/exo-u1mp.md) |
| Regex matching and SQL candidate queries | [exo-mkyx](../.tickets/exo-mkyx.md) |
| Global and recursive directory scope within tenancy | [exo-sjlw](../.tickets/exo-sjlw.md), [exo-hlw6](../.tickets/exo-hlw6.md) |
| Multiple backend instances and capability-aware routing | [exo-u1mp](../.tickets/exo-u1mp.md) |
| Pending external revisions, durable outbox, idempotent confirmation and recovery | [exo-3m9v](../.tickets/exo-3m9v.md) |
| SQLite backend through rusqlite | [exo-b12v](../.tickets/exo-b12v.md), [exo-mkyx](../.tickets/exo-mkyx.md) |
| Redis key-value queries and full-value reads | [exo-03m1](../.tickets/exo-03m1.md) |
| zvec snapshot-bound similarity queries and referenced-data reads | [exo-pw6d](../.tickets/exo-pw6d.md) discovery, [exo-upbn](../.tickets/exo-upbn.md) implementation |
| Structured dates and event triggers | [exo-reie](../.tickets/exo-reie.md) |
| Episodic SQLite, including before/after/during reads | [exo-jku1](../.tickets/exo-jku1.md) |
| Body-free retrieval-only registration and explicit unsupported-write errors | [exo-52xr](../.tickets/exo-52xr.md) |
| Backend normalization, filtering, stable rank and tie-breaking | [exo-mkyx](../.tickets/exo-mkyx.md), each backend slice, [exo-1izl](../.tickets/exo-1izl.md) |

## App Server and harness requirements

Source: `spec/App server spec.md`; finalized decisions:
`verification/DECISIONS.md` and `verification/RESULTS.md`.

| Spec requirement | Owning tickets |
| --- | --- |
| Typed candidates, provenance, trust, source pair, and revision | [exo-sjlw](../.tickets/exo-sjlw.md), [exo-mkyx](../.tickets/exo-mkyx.md), [exo-aqd9](../.tickets/exo-aqd9.md) |
| Reproducible read-cost estimate without runtime token budgeting | [exo-mkyx](../.tickets/exo-mkyx.md), [exo-aqd9](../.tickets/exo-aqd9.md) |
| Escaping, trusted wrapper, untrusted fields, separate full tool reads | [exo-aqd9](../.tickets/exo-aqd9.md), [exo-ljge](../.tickets/exo-ljge.md), [exo-dcwx](../.tickets/exo-dcwx.md) |
| Final serialized size limits, rejection instead of truncation, fixed wrapper bound | [exo-aqd9](../.tickets/exo-aqd9.md) |
| Eligibility before duplicate collapse, exact lexicographic order | [exo-jtv5](../.tickets/exo-jtv5.md) |
| Per-source limits after duplicate collapse and remaining live-slot cap | [exo-jtv5](../.tickets/exo-jtv5.md), [exo-h2bg](../.tickets/exo-h2bg.md) |
| Authoritative occurrences, accepted delivery, compaction and resurfacing | [exo-h2bg](../.tickets/exo-h2bg.md), [exo-dcwx](../.tickets/exo-dcwx.md) |
| Logical-event cooldown and last-shown bookkeeping | [exo-h2bg](../.tickets/exo-h2bg.md), [exo-jtv5](../.tickets/exo-jtv5.md) |
| All named Codex event classes scanned before continuation | [exo-a6l7](../.tickets/exo-a6l7.md) discovery, [exo-dcwx](../.tickets/exo-dcwx.md) implementation |
| Repeatable decisions from versioned state across backends | [exo-u1mp](../.tickets/exo-u1mp.md), [exo-1izl](../.tickets/exo-1izl.md) |

## All seven use cases

Source: `spec/spec - use cases to support.md`.

| Use case | Owning demonstration |
| --- | --- |
| Long-running task coherence and reusable tool recall | [exo-t3ru](../.tickets/exo-t3ru.md) |
| Long-running questions with references to notes files | [exo-t3ru](../.tickets/exo-t3ru.md) |
| Frontend preference across sessions | [exo-hlw6](../.tickets/exo-hlw6.md) |
| Test-suite duration with concurrent measurement updates | [exo-lm9g](../.tickets/exo-lm9g.md), [exo-bios](../.tickets/exo-bios.md) |
| Anthropology corpus and source diversity | [exo-52xr](../.tickets/exo-52xr.md) |
| Repository-scoped experiment location | [exo-hlw6](../.tickets/exo-hlw6.md) |
| Incremental timeline reconstruction | [exo-jku1](../.tickets/exo-jku1.md) |

## Sequencing and scope boundaries

Codex and zvec discovery have priority 0 and no dependencies. The original
SQLite milestones remain the implementation starting point. Date triggers
and file-reference workflows follow M1; broader scope policy follows M2.
Backend brokering and non-SQLite integration follow SQLite acceptance M3.
The expansion epic depends on full-spec acceptance, not the other way around.

The user clarified that adding question notes is an ordinary file-tool
workflow. It does not require an Exomemory append command or a discovery
ticket. Memory revisions describe the file reference, not its changing notes.

All live test memories remain directory-scoped inside `tests/`. General global
scope is implemented and checked with synthetic policy and persistence
fixtures, without enabling real global memory stores during tests.

Codex is the only named harness. The generic adapter contract does not specify
additional hosts. The spec also does not require production installation,
packaging, a dynamic plugin loader, or arbitrary user-authored SQL. Those are
not hidden prerequisites of this roadmap. Model checks retain their existing
finite scope; implementation and workflow evidence must establish the rest.
