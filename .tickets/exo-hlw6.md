---
id: exo-hlw6
status: open
deps: [exo-190g]
links: []
created: 2026-09-19T03:49:34Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-m2, scope, use-case]
---
# Complete global and directory scope policy with preference and location scenarios

## Background in Plain English — WHY

The first slice deliberately restricts test runs to directory-scoped memories. The full command spec also defines global scope within a tenancy, and the preference and experiment-location examples need precise cross-session and directory behavior. Sources: spec/exoctl-command-spec.md, Creation; spec/spec - use cases to support.md, Frontend preference and Experiment location.

## Outcomes / Acceptance Criteria — WHAT

1. The general policy represents global scope as every directory allowed by the tenancy and directory scope as the recorded canonical root plus permitted descendants.
2. Outside the enforced test profile, command validation and storage representations support global registration and permitted revision changes under the general policy; verify these paths with synthetic or mocked persistence, without enabling real global stores.
3. All operations enforce the same tenancy and scope rules; an untrusted memory field or caller-supplied directory cannot grant access outside the caller's established permissions.
4. Selection orders a closest applicable directory before its ancestors and global scope, with deterministic behavior across sessions.
5. Global policy and serialization are exercised with synthetic fixtures; live daemon and model test runs still reject global memory creation and use only directory scopes inside tests/.
6. A preference scenario surfaces a frontend preference as optional data in a later session and verifies the envelope preserves the current user's authority.
7. An experiment-location scenario stores a parent-directory location as body data while the memory remains scoped to its repository; unrelated sibling sessions receive no ghost or record.
8. Tests cover cross-tenancy denial, nested scopes, canonical path boundaries, and shared overwritable records within one tenancy.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use pure policy tests for global semantics and a fake directory hierarchy entirely beneath the test root for live scenarios. Keep real-user global rollout and installation outside the implementation acceptance for this ticket.

## Future work

exo-1izl includes the global policy evidence and both use-case scenarios. Deployment to real memory stores requires a separate rollout task if later requested.

## Definitions

Global scope: visibility across every directory permitted by the memory's tenancy. Tenancy: the ownership and permission boundary. Synthetic fixture: a constructed test value that does not create a live global memory. Canonical path: a normalized resolved filesystem path. Authority: the right of an instruction to govern the model's behavior.
