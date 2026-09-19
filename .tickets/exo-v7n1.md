---
id: exo-v7n1
status: open
deps: [exo-dn0t, exo-i68v]
links: []
created: 2026-09-19T20:07:26Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-9tjp
tags: [cli, ux]
---
# Make exoctl usage and parser errors self-explanatory

## Background in Plain English — WHY

`exoctl` must stand on its own because no Exomemory skill is installed. Today its full help presents low-level test options without enough product context, bare invocation does not provide a compact command reminder, and several parser failures collapse into “unknown, duplicate or incomplete option.” That wording made a normal configuration mistake look like a parser incompatibility.

The concise product explanation already exists in `spec/overview.md` and was enough for a GPT 5.6 Luna agent to work out the intended workflow. The CLI should present that same explanation rather than require repository access.

## Outcomes / Acceptance Criteria — WHAT

1. Running `exoctl` with no arguments prints a short usage form with the available commands and compressed option grammar, then exits with the documented usage-error status.
2. `exoctl help` and `exoctl --help` print the normal options, followed by the contents of `spec/overview.md` word for word, followed by the existing “Non-empty validated triggers...” material.
3. The overview is included from one checked source or guarded by a drift test so command help cannot silently diverge from `spec/overview.md`.
4. Installed help explains normal configuration and does not tell users that a repository-local `tests/.runtime/` directory is required. Explicit test and administrator options remain discoverable in an accurately labeled section.
5. Unknown, duplicate, missing-value, and incomplete-command errors are distinct. Each error names the offending option or missing value and gives the relevant short usage or corrective next step.
6. A repeated explicit option reports that it was supplied more than once. Installed configuration and an explicit override are resolved by precedence and do not produce a duplicate-option error.
7. Tests exercise no-argument output, both help spellings, exact overview inclusion and placement, every parser error class, and the actual installed executable. The output is sufficient to create, inspect, read, and update a memory without a skill or repository file.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Keep a compact usage string separate from full help. Use typed parser errors rather than one catch-all branch. Rust's `include_str!` can make the repository overview the source for full help, with a placement test around the surrounding sections.

## Future work

`exo-0kao` runs the documented commands through the installed binary. Future command additions must update both the short grammar and the overview when they change the user workflow.

## Definitions

Short usage: the compact command grammar printed after a bare or malformed invocation. Full help: the longer reference printed by `help` or `--help`. Parser: the code that turns command-line words into a command and options. Drift: an unintended difference between two copies of documentation.
