---
id: exo-t3ru
status: open
deps: [exo-5iw6]
links: []
created: 2026-09-19T03:49:34Z
type: task
priority: 3
assignee: coopwilliams
parent: exo-fsde
tags: [post-m1, sqlite, use-case, acceptance]
---
# Demonstrate long-running tool recall and question-note workflows

## Background in Plain English — WHY

The existing demo covers measured test duration, but the spec also promises continuity for reusable tools and open questions across context windows. A question memory points the agent to a notes file; the agent uses ordinary file tools to read or append notes. Exomemory does not provide an append operation. Source: spec/spec - use cases to support.md, Long-running task coherence and Long-running questions, clarified by the user on 2026-09-18.

## Outcomes / Acceptance Criteria — WHAT

1. A small tool-library scenario records several tools with scoped triggers, then uses a later task event to surface relevant previews and retrieve the chosen tool record.
2. A question-note scenario records multiple open questions and surfaces the relevant question only when its trigger matches a later event.
3. The question scenario demonstrates three choices: ignore the ghost, retrieve the memory's file reference and read the notes with file tools, or use that reference to append a snippet with ordinary file tools without reading the existing notes. The notes file remains inside the test run's permitted directory.
4. Session changes and compaction preserve durable memories while live-context presence, count limits, cooldown, and bounded previews continue to apply.
5. Unrelated records and full note bodies do not enter context merely because they exist; provenance and revisions remain visible on explicit reads.
6. A reproducible command runs the scenarios with small fixtures, all memories scoped under tests/, and no required benchmark downloads or growing AGENTS.md tool catalog.
7. A memory revision describes the file reference and its metadata, not an immutable snapshot of the notes file; ordinary file edits do not become Exomemory revision changes or require an exoctl append command.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Extend the controlled harness demonstrations with a few concrete tools, questions, and notes-file references. Retrieve references with exoctl and use ordinary agent file tools for the notes workflow. Record tool calls to show that append does not require a preceding notes-body read. Use explicit event sequences to prove when each memory surfaces.

## Future work

exo-1izl includes these use cases in the full-spec acceptance evidence. Large-scale relevance evaluation is outside the current spec's correctness requirements.

## Definitions

Tool library: saved descriptions and references to reusable tools. Context window: the model's currently available conversation input. Compaction: replacement or removal of earlier context. Fixture: a small reproducible test input. File reference: stored information identifying the notes file. Append: an ordinary file operation that adds text at the end of a file.
