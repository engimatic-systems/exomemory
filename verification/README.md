# Exomemory TLA+ verification

This directory contains an executable state-machine model of the concurrency
and context-injection mechanics described by the active Exomemory specs.

The model is intentionally narrow. It covers behaviors with crisp safety
properties; it does not attempt to prove that an LLM will find a ghost useful,
interpret prose correctly, or make a good relevance judgment.

## Files

- `Exomemory.tla` — storage, overwrite, context-presence, authority, and
  injection-count state machine.
- `GhostSelection.tla` — fixed-size/count-cap selection, deterministic ranking,
  and `annoy` penalties.
- `configs/Current_*.cfg` — one isolated counterexample per underspecified
  current behavior.
- `configs/Hardened.cfg` and `configs/Finalized_Selection.cfg` — the finalized
  design choices split across the two focused models.
- `run-models.sh` — downloads the official TLA+ command-line tools and checks
  every expected result.
- `DECISIONS.md` — the choices and caveat submitted through the review form.
- `RESULTS.md` — findings and their shortest counterexample traces.

## Run it

```sh
./verification/run-models.sh
```

Java 11 or newer and `curl` are required. The script uses Homebrew's Java path
on this machine when `java` is not on `PATH`; set `JAVA_BIN` to override it.
The official `tla2tools.jar` v1.7.4 release is cached under `.tools/` and is not
committed.

## How the prose maps to the model

| Prose mechanism | Model state/action |
| --- | --- |
| data plus trigger creation | `bodyVersion`, `ghostVersion`, `triggerPublished` |
| overwrite from any session | `BeginOverwrite`, `CommitOverwrite` |
| concurrent connections | independently staged `Writers` |
| hook injection | `InjectGhost` |
| lagging history-file watcher | `historyView`, `SyncHistory` |
| cooldown of a few messages | `cooldown`, `AdvanceHook` |
| compaction removes active context | `CompactAway` |
| higher-priority context | `contextAuthority` |
| all returned ghosts are injected | injection without a budget guard |

The `AtomicCreate`, `AtomicOverwrite`, `CompareAndSwap`,
`ExactContextDedup`, `EnforceBudget`, and `SafeAuthority` constants are design
choices. The active specs do not currently settle them. The `Current_*.cfg`
files choose a behavior the prose permits; `Hardened.cfg` enables the proposed
safety contracts.

`GhostSelection.tla` additionally models the chosen decision-four refinement:
a maximum ghost count plus a hard bound on each final rendered ghost. The
configuration asserts that the resulting worst-case byte size fits the hook's
injection envelope, so runtime token accounting is not needed. Candidate order
is deterministic, and `annoy` adds a rank penalty rather than making a random
surface/no-surface choice.

## What a successful hardened run means

TLC exhaustively explores the configured finite core model: two memories, one
session, two concurrent writers, three record versions, a one-event cooldown,
and a one-ghost count cap. The selection model separately explores three ranked
candidates from two sources, bounded negative feedback, stable tie-breaking, a
per-source quota, and the fixed-size injection envelope. Passing means no
represented execution in those finite state spaces violates the listed
invariants.

It is not a proof about arbitrary scale, Rust code, backend implementation
details, path containment, trigger relevance, or LLM behavior. Those require
refinement models, implementation tests, and product evaluation respectively.
