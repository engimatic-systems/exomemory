# Model-checking results

Checked on 2026-09-18 with TLC 2.19 from the official TLA+ tools v1.7.4.

## Outcome

The seven original current-spec configurations each produce the intended
counterexample. A new configuration also shows that unconstrained/random
surfacing can select a lower-ranked candidate. The finalized core completes
with no violations after generating 64,716 states and visiting 11,977 distinct
states. The finalized selection model completes with no violations after
generating 431 states and visiting 132 distinct states.

These findings identify choices missing from the prose spec. They are not
implementation bug reports.

| Safety contract | Shortest counterexample TLC found | Required spec decision |
| --- | --- | --- |
| `NoDanglingGhost` | Publish a trigger before its body write completes. | Make a memory invisible until body, ghost, and trigger are committed. |
| `GhostBodySameVersion` | A writer commits body v2 while the visible ghost remains at v1. | Publish body and ghost as one versioned snapshot. |
| `AtMostOneGhostPerWindow` | Inject; let the cooldown expire before history sync; inject the same key again. | Deduplicate against an authoritative live-window ledger, not only a lagging file. |
| `NoStaleHistorySuppression` | Inject; sync append-only history; compact the ghost away; suppress it because history still contains the key. | Track what is live after compaction, not what appeared at any earlier time. |
| `WithinGhostBudget` | Two memories match and both are injected into a context whose count cap is one. | Set a hard per-hook/per-window count and final-rendered ghost-size limit before injection. |
| `NoUntrustedDeveloperContext` | An untrusted memory is injected with developer authority. | Preserve retrieved text as quoted data with provenance; never promote its instructions. |
| `NoStaleWriterCommit` | Two writers read v1; writer A commits v2; writer B commits work based on v1 as v3. | Require compare-and-swap or explicit conflict resolution. |
| `AlwaysSelectedTopRanked` | Two candidates are eligible; unconstrained surfacing selects the lower-ranked one. | Use a total deterministic order and make `annoy` a rank penalty. |

## Counterexample action traces

The logs under `results/` include full state values. The action-only traces are:

```text
NoDanglingGhost:
  Init -> PublishTrigger

GhostBodySameVersion:
  Init -> WriteInitialBody -> PublishTrigger
       -> BeginOverwrite -> CommitOverwrite

AtMostOneGhostPerWindow:
  Init -> WriteInitialBody -> PublishTrigger -> InjectGhost
       -> AdvanceHook -> InjectGhost

NoStaleHistorySuppression:
  Init -> WriteInitialBody -> PublishTrigger -> InjectGhost
       -> SyncHistory -> AdvanceHook -> CompactAway
       -> ObserveStaleSuppression

WithinGhostBudget:
  Init -> WriteInitialBody(m1) -> WriteInitialBody(m2)
       -> PublishTrigger(m1) -> PublishTrigger(m2)
       -> InjectGhost(m1) -> InjectGhost(m2)

NoUntrustedDeveloperContext:
  Init -> WriteInitialBody(untrusted) -> PublishTrigger(untrusted)
       -> InjectGhost(untrusted)

NoStaleWriterCommit:
  Init -> WriteInitialBody -> BeginOverwrite(A) -> BeginOverwrite(B)
       -> CommitOverwrite(A) -> CommitOverwrite(B)

AlwaysSelectedTopRanked:
  Init -> SelectGhost(lower-ranked candidate)
```

## Finalized contracts represented by the clean configurations

1. A record becomes trigger-visible atomically with its first complete version.
2. Body and ghost metadata expose the same immutable revision.
3. Overwrite is conditional on the revision the writer read.
4. Deduplication uses the actual live context window and understands compaction.
5. Injection is rejected before it exceeds the configured budget.
6. Untrusted retrieved material is data, never developer-authority instruction.
7. A maximum ghost count and maximum final-rendered ghost size imply a fixed
   worst-case byte envelope; no runtime token budget is required.
8. Candidate selection uses a total deterministic order. `annoy` monotonically
   adds a ranking penalty and never performs a random draw.
9. A source is `(backend instance, trigger id)` after duplicate ghost keys are
   collapsed. No source may contribute more than its configured quota to one
   hook response; a lower-ranked candidate from another source can therefore
   outrank an otherwise-preferred candidate whose source quota is full.

The cooldown remains useful for rate limiting, but it is not treated as a
correctness mechanism.

No additional unresolved design choice appeared in the finalized finite
models.
