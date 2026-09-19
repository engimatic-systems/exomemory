---
id: exo-uctk
status: open
deps: [exo-tuc0, exo-71ur]
links: [exo-qvzg]
created: 2026-09-19T20:07:26Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-9tjp
tags: [codex, install]
---
# Replace the Codex shell launcher with a compiled shim

## Background in Plain English — WHY

Codex CLI 0.155.1 has no persistent setting that makes every normal terminal session connect to Exomemory's hosted Codex process. The current installation therefore places a small shell script named `codex` earlier on `PATH`; it adds the host's Unix-socket address and current directory, then starts the official Codex executable. The official executable remains installed and unchanged.

That routing is currently necessary for recall, but shipping a generated shell script weakens portability and makes the installation harder to inspect. The same adapter can be a small compiled Rust program installed at the same command path.

## Outcomes / Acceptance Criteria — WHAT

1. Installation places a compiled Exomemory launcher at the owned `codex` path and records the separate official Codex executable it delegates to. No generated shell script is required at runtime.
2. A normal interactive invocation connects the official CLI to the configured Exomemory host and passes the caller's current directory without changing the user's remaining arguments.
3. Explicit remote invocations and supported non-interactive or maintenance commands preserve their documented routing behavior; the launcher never recursively selects itself as the official executable.
4. Status checks both inherited and fresh login-shell resolution, verifies that the selected file is the compiled launcher, and reports the official executable and host destination in actionable diagnostics.
5. Install, upgrade, repair, and uninstall are idempotent and preserve an executable that existed at the owned path before Exomemory installation, along with later user shell edits.
6. Regression tests reproduce Homebrew winning `PATH`, validate argument and exit-status forwarding without contacting a user store, and prove repair and uninstall remain reversible.
7. Installation and verification notes plainly state why this launcher is required for the tested Codex version and distinguish it from the official Codex CLI.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Build a small Rust launcher binary that resolves configuration, validates the recorded upstream path, and replaces itself with the official process where the platform supports it. Reuse the install manifest and login-shell checks from `exo-71ur`.

## Future work

`exo-qvzg` investigates whether Pi can host Exomemory directly without shadowing its normal CLI. A future verified Codex setting for persistent remote hosts could allow this launcher to be retired.

## Definitions

Shim: a small compatibility program placed in front of another executable. Official executable: the Codex CLI distributed by OpenAI and left unchanged by Exomemory. Hosted Codex process: the Codex app server process whose lifecycle hooks can inject Exomemory previews. Login shell: a shell that loads login configuration before resolving commands.
