---
id: exo-52xr
status: open
deps: [exo-u1mp]
links: []
created: 2026-09-19T03:49:34Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-sqlite, retrieval-only, use-case]
---
# Support retrieval-only local-file memories and the corpus workflow

## Background in Plain English — WHY

The spec permits creating a retrieval-only memory without stored body data and requires writes to its backend to fail explicitly. The anthropology corpus example adds a concrete workflow: advertise a document collection, then let the agent choose to read or search it. Sources: spec/data service.md, retrieval-only paragraph; spec/exoctl-command-spec.md, Creation; spec/spec - use cases to support.md, Anthropology corpus.

## Outcomes / Acceptance Criteria — WHAT

1. A local-file retrieval plugin registers an existing test corpus using exoctl create without requiring stored body data, while atomically publishing canonical trigger, scope, provenance, and backend-reference metadata.
2. The contract distinguishes registration or permitted canonical metadata changes from external body writes; unsupported body creation and replacement return explicit capability errors.
3. Matching queries have documented normalization, file filtering, a reproducible corpus snapshot identity, deterministic result order, and stable references.
4. Full reads return referenced file data as tool output with source identity and the version actually read; changed or missing source data has a defined response that cannot mislabel content as the advertised revision.
5. Scope enforcement checks resolved referenced files as well as the session directory, including symlink escapes and access changes.
6. A corpus scenario shows a bounded preview, an explicit agent-selected read or search, and per-source diversity when a broad corpus trigger matches many records.
7. All corpus fixtures and generated extracts remain under tests/; the plugin never searches the user's real document collection during verification.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Use a small text corpus first. A controlled rg or ripgrep-all integration is one possible query engine; choose and document the supported formats. Preserve file versions or detect changes explicitly instead of assuming a path denotes immutable content.

## Future work

exo-1izl combines retrieval-only results with writable backend results. Format expansion beyond the documented local-file capability is not required by this ticket.

## Definitions

Retrieval-only backend: a source that can be queried and read but does not accept body writes. Corpus: a collection of documents. Canonical metadata: the authoritative stored description and trigger information. Snapshot: an identified fixed corpus view. rg and ripgrep-all: file-search tools; format support depends on the chosen integration.
