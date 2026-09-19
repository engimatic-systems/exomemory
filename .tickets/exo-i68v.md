---
id: exo-i68v
status: open
deps: [exo-tuc0]
links: []
created: 2026-09-19T20:07:26Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-9tjp
tags: [cli, install]
---
# Install exoctl as a self-configuring Rust binary

## Background in Plain English — WHY

The current personal installation places a shell script at `~/.local/bin/exoctl`. That script appends socket, tenant, scope, and runtime arguments before invoking the real Rust binary. The binary's own help therefore describes those arguments as required and mentions the isolated test layout. In the first live use, an agent followed that help and supplied the same options again, producing a confusing duplicate-option error and an unnecessary `tests/.runtime/` directory.

The shipped command should contain the behavior it needs. Normal users should run one compiled `exoctl` executable, while tests and administrators retain explicit ways to select an isolated profile.

## Outcomes / Acceptance Criteria — WHAT

1. A personal install puts the actual Rust `exoctl` executable at the command path; normal operation does not invoke or depend on a generated shell script.
2. Configuration precedence is explicit command-line overrides, then `EXOMEMORY_CONFIG`, then the installed user configuration. Loading installed configuration never makes an explicit override count as a duplicate option.
3. The installed configuration contains persistent connection and policy values. The command obtains the working directory from the process and derives the session from the supported Codex environment, with a documented safe fallback for non-Codex terminal use.
4. Test and controlled-harness commands continue to require an explicit existing runtime directory, tenant, socket, and permitted scope. They cannot fall back to the user's installed store.
5. Install, upgrade, repair, status, and uninstall migrate an existing wrapper installation in place, preserve the memory database, and remove only files owned by Exomemory.
6. Diagnostics identify which configuration source was selected without printing secrets, and failures name a missing or invalid configuration path directly.
7. Regression coverage executes the installed binary from a clean shell environment and proves that no shell interpreter or wrapper-supplied arguments are required.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Put profile resolution in the Rust CLI before command dispatch and model the resolved settings as one typed configuration. Keep the test profile explicit at the call site. Record owned configuration and executable paths in the existing install manifest so upgrades and uninstall remain reversible.

## Future work

`exo-v7n1` revises help and parser diagnostics for the new configuration behavior. `exo-uctk` applies the same compiled-program packaging principle to the Codex launcher.

## Definitions

Entry point: the executable selected when a shell resolves a command name. Configuration precedence: the order used when the same setting is available from more than one source. Installed user configuration: Exomemory's persistent per-user connection and scope settings. Controlled profile: the isolated configuration used by tests and manual experiments under `tests/.runtime/`.
