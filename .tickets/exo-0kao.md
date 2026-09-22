---
id: exo-0kao
status: closed
deps: [exo-v7n1, exo-uctk]
links: []
created: 2026-09-19T20:07:26Z
type: task
priority: 1
assignee: coopwilliams
parent: exo-9tjp
tags: [acceptance, verification]
---
# Accept the pre-merge standalone interface revisions

## Background in Plain English — WHY

The individual changes alter a shared installed workflow: command resolution, configuration, help, ghost serialization, and Codex host routing. Component tests alone could miss a mismatch between them. This gate proves that an upgraded personal installation still stores and recalls a memory while presenting the simpler interface agreed for the merge.

## Outcomes / Acceptance Criteria — WHAT

1. Formatting, workspace linting, workspace tests, the process test, affected installer regressions, and `verification/run-models.sh` pass with their recorded expectations.
2. An upgrade fixture starts from the existing shell-wrapper installation, installs the compiled `exoctl` and Codex launcher, retains its database and configuration, and can be uninstalled back to the prior state.
3. The installed `exoctl` is a compiled executable. Bare invocation, full help, configuration discovery, malformed options, and the documented create/read/meta/update workflow match `exo-v7n1` and `exo-i68v`.
4. Captured delivered context proves that ghosts omit `provenance` and `source_id`, while `exoctl meta` still returns the stored origin and trigger information and selection source quotas remain enforced.
5. Fresh login and interactive shells resolve the compiled Codex launcher ahead of Homebrew, and `exomemory status` fails clearly for native-Codex bypass, wrong upstream selection, or an unreachable host.
6. A real Codex 0.155.1 session using GPT 5.6 Luna stores a directory-scoped memory, triggers it from a normal terminal session, suppresses the attempted preview until completed compaction, and can read its full record explicitly.
7. Live tests use a unique directory under `tests/.runtime/`, remove temporary authentication material, retain reviewable evidence, and do not create or alter unrelated memories.
8. The reviewed build is applied to the existing personal installation without deleting its store, and the exact installed versions, commands, results, and remaining Codex launcher limitation are recorded in a named verification note.

## Methodological Suggestions — HOW

These are suggestions; another sound method is acceptable. Extend the installed-path harness so one evidence file records executable identities, help hashes, delivered ghost JSON, metadata, hook events, and compaction behavior. Use GPT 5.6 Luna only for the live Codex inference step; keep deterministic checks outside the model session.

## Future work

`exo-a81g` remains the M3 CI publication gate. `exo-qvzg` investigates a Pi adapter that could offer recall without the Codex launcher.

## Definitions

Upgrade fixture: an isolated reproduction of the previously installed layout. Delivered context: the exact preview text supplied to the model. Compiled executable: a native program rather than a shell script. Acceptance evidence: retained output sufficient for another reviewer to repeat and assess the result.

## Notes

**2026-09-22T03:39:03Z**

Accepted 2026-09-21 on e0a83ab. ./verification/check-sqlite.sh passed after the bounded LaunchAgent restart fix: formatting, clippy, all workspace/process/installer tests, and all 11 model expectations. Installed-interface evidence: tests/.runtime/r5f7e42dd72fa/harness/interface-evidence.json. Real Codex 0.155.1 + GPT 5.6 Luna installed-path test passed with suppression, completed-compaction reset, resurfacing, full exoctl read, 11/11 trusted hooks, and authentication cleanup; evidence: tests/.runtime/rb6489ba567aa/harness/installed-evidence.json. The reviewed release was applied to the personal installation; installed binaries match it byte-for-byte, status is healthy, and the personal store remained 1 record at revision 1. Full record: verification/PREMERGE-INTERFACE-ACCEPTANCE.md.
