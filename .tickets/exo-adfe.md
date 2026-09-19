---
id: exo-adfe
status: open
deps: []
links: []
created: 2026-09-19T14:19:40Z
type: task
priority: 0
assignee: coopwilliams
parent: exo-fq27
tags: [codex, host-gap, m3]
---
# Implement and verify a Codex host occurrence contract

## Background in Plain English — WHY

The installed Codex 0.155.1 host accepts injected items without a presence
version or occurrence receipt. Adding conditional fields to the request still
returns an empty success response. Its compaction item does not identify the
surviving ghosts. verification/CODEX-HOST-CONTRACT.md records the executable
evidence and proposed host change. A human must choose a pinned project host
fork or an upstream implementation as the supported distribution.

## Outcomes / Acceptance Criteria — WHAT

1. The supported distribution and owner are recorded, with a pinned source
   commit, reproducible Rust build, and isolated invocation command.
2. The real host owns typed occurrence identity and exposes versioned snapshots
   of actual model input after insertion, compaction, eviction, fork, and resume.
3. Insertion checks the expected generation and presence version atomically,
   returns occurrence receipts or explicit rejection, and handles retry and
   payload mismatch without duplicate insertion, including after compaction.
4. Every event required by spec/App server spec.md has an executable barrier
   check proving the relevant context waits for selection and insertion,
   including child contexts and permission events.
5. Real-host tests compare actual model input to snapshots and exercise retained
   and removed ghosts, concurrency, restart, isolation, authority, and bounds.
6. Unsupported host builds fail capability negotiation explicitly. No transcript
   or cooldown approximation can enable exact-presence mode.

## Methodological Suggestions — HOW

The proposed semantics in verification/CODEX-HOST-CONTRACT.md are a starting
point; another sound implementation is acceptable. Reuse the controlled host's
contract tests where possible, but inspect real host context to validate the
adapter. Do not change the installed global Codex executable or global hooks.

## Future work

exo-7pi0 completes the capability investigation against the verified build.
exo-dcwx integrates selection and delivery. exo-a81g records acceptance evidence
before exo-fq27 can close.

## Definitions

Occurrence: one typed ghost item in a particular live model context. Receipt:
the host's record that a delivery was accepted. Generation: an identity that
prevents stale requests from an earlier context instance being accepted.
Barrier: a point where the host waits before continuing. Fork: either a
maintained source variant or a child conversation, as specified by context.
Capability negotiation: checking that the host supplies every required interface.
