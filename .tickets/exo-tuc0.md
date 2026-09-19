---
id: exo-tuc0
status: closed
deps: [exo-dcwx]
links: []
created: 2026-09-19T15:41:23Z
type: task
priority: 2
assignee: coopwilliams
tags: [install, codex, local]
---
# Install a reversible personal Codex terminal trial

## Background in Plain English — WHY

The user explicitly requested installation for normal Codex terminal sessions across projects. The verified implementation currently requires an isolated test runtime. A personal trial needs an explicit local profile and a reversible installation that preserves existing Codex configuration.

## Outcomes / Acceptance Criteria — WHAT

1. Existing tests remain confined to their explicit test profile; a separately selected local profile permits a private persistent store and directory-scoped memories under an explicit permitted root.
2. Normal interactive codex launches use the verified Unix-socket host and all lifecycle hooks. Existing user models, approval settings, unrelated hooks and project configuration are preserved.
3. The user can create/read/update memories through an installed exoctl and inspect/stop/uninstall the integration without deleting stored memories.
4. Hook trust applies only to the exact installed definitions. No general approval, sandbox or hook-trust bypass is installed.
5. Isolated regression tests cover installation/configuration preservation and local-scope boundaries; real inference validation uses GPT 5.6 Luna inside tests/.
6. Install and start the reviewed build for this user and report how to try it and any concrete limitation.

## Methodological Suggestions — HOW

Use a Rust installer and macOS user LaunchAgents for the data service and Codex host. A small launcher may connect the standard terminal UI to the local host. Keep runtime configuration explicit, back up changed files and retain the store on uninstall. Another sound reversible method is acceptable.

## Future work

exo-hlw6 owns broader global-scope behavior; this trial keeps directory scopes. exo-a81g tracks the separate GitHub workflow permission blocker. Neither requires changing the user's preferred normal model.

## Definitions

Local profile: explicitly authorized use of a persistent store outside the test folder. LaunchAgent: a macOS per-user background service. Hook trust: approval of the exact installed hook definition. Unix socket: a machine-local connection endpoint.

## Notes

**2026-09-19T15:52:51Z**

Installed the release build for the user: private personal store, two healthy macOS LaunchAgents, normal interactive codex launcher, configured exoctl, and 11/11 exact hook definitions enabled/trusted. No broad bypass or sample data installed. All 40 offline entries, formatting/clippy and the separate real GPT 5.6 Luna installed-path test pass; evidence tests/.runtime/rc7197bc7335c/harness/installed-evidence.json, temporary auth removed. Installer is reversible and preserves stored records. See verification/LOCAL-INSTALL.md.
