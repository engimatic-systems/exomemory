---
id: exo-u1mp
status: open
deps: [exo-fq27]
links: []
created: 2026-09-19T03:49:33Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-sqlite, service, backends]
---
# Route requests through multiple named backend instances

## Background in Plain English — WHY

The first slice registers only SQLite. The Data Service spec also requires brokering backend connections and source identifiers that distinguish instances of the same implementation. A concrete broker is needed before the other backends can participate. Source: spec/data service.md, service overview and Deterministic candidate contract.

## Outcomes / Acceptance Criteria — WHAT

1. Configuration registers multiple named backend instances with stable identifiers and explicit read, create, overwrite, and trigger capabilities.
2. The Data Service dispatches create, full read, metadata, conditional overwrite, and candidate queries to the correct instance while preserving canonical key and revision identity.
3. exoctl help and list-trigger-types describe only configured supported operations and their parameters, including backend-specific trigger types.
4. Duplicate instance names, unknown backend references, unavailable capabilities, and unavailable services produce documented structured errors without silently selecting another backend.
5. Candidate collection records each participating instance's snapshot identity and applies the existing common schema, access filters, ranking bands, and source pair; raw backend scores are not compared across implementations.
6. Tests cover two SQLite instances and a capability-limited test implementation, with all runtime state and scopes beneath tests/.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Extend the existing Rust plugin interface instead of introducing dynamic loading. Keep configuration small and pass explicit instance references. Define reproducible response behavior for a failed source rather than relying on arrival timing.

## Future work

exo-3m9v adds external publication. exo-03m1, exo-upbn, exo-52xr, and exo-jku1 register concrete integrations; exo-1izl tests combined results.

## Definitions

Backend instance: one configured storage connection, distinct from its implementation type. Broker: the service component that routes operations to those connections. Capability: an operation the backend explicitly supports. Ranking band: a documented relevance category shared by candidate records.
