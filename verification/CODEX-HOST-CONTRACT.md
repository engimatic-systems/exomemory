# Codex host contract gap — 0.155.1

## Decision and repeatable evidence

Exact-presence integration remains unsupported by the demonstrated Codex
0.155.1 App Server interface. Do not enable exo-dcwx or close M3 on the strength
of injection acknowledgments. This is a negative capability investigation,
not a real-host acceptance test.

Run from the repository root:

```sh
cargo run --manifest-path tests/probes/codex/Cargo.toml --target-dir target/probes
```

The executable pins the inspected version and writes `version.txt`, generated
schemas, `rpc.jsonl`, `stderr.log`, and `capabilities.json` in its printed unique
`tests/.runtime/codex-*` directory. All inputs are synthetic. The child has a
separate Codex home, an ephemeral thread, and an offline provider; no model
turn is started. Each RPC has a total 30-second deadline even if notifications
keep arriving, and the child is killed and reaped when the probe ends.

Observed on 2026-09-19:

| Check | Observed result | Meaning |
| --- | --- | --- |
| Injection request schema | Only `threadId` and `items` | No declared conditional presence version or delivery identity |
| Injection response | `{}` | No occurrence receipt or resulting presence version |
| Injection with proposed `expectedPresenceVersion = u64::MAX` and `deliveryId` | `{}` | Extra fields do not establish a conditional-insertion contract |
| Repeat with the same proposed delivery identity | `{}` again | No receipt proving retry deduplication; the probe does not claim that duplicate live items were observed |
| Current `contextCompaction` item schema | Only `id` and `type` | No retained/removed occurrence snapshot |
| Deprecated compaction notification schema | Only thread and turn IDs | Also insufficient for exact presence |

The invented proposal fields intentionally test whether extending a client
request alone supplies the required semantics. They are **not supported RPC
fields**. Successful acknowledgments cannot be used to implement compare and
swap or idempotency. Schema assertions fail on a changed interface so that a
future version requires a fresh investigation.

The [official hook documentation](https://learn.chatgpt.com/docs/hooks)
describes synchronous hooks and lists the required event families. Its
compaction inputs describe the triggering event, not the actual surviving
occurrences. The [App Server documentation](https://learn.chatgpt.com/docs/app-server)
describes injection and lifecycle notifications. These documents do not supply
the missing contract. Installed schemas and executable observations above are
the version-specific evidence.

## Concrete host extension required

These are proposed semantics, not existing Codex API names. exo-adfe owns their
implementation and real-host verification.

1. **Host-owned identity.** Associate each accepted ghost's occurrence ID,
   key, revision, and delivery ID with typed model-context items. User text
   containing an occurrence-like string must never create ledger membership.
   Keep tenant, session, and context identity explicit.
2. **Atomic snapshot.** Expose a context generation and monotonically changing
   presence version with the exact live occurrence set. Derive the set from
   the actual model input after insertion, compaction, eviction, fork, and
   resume. Snapshot publication and context mutation share one critical
   section. Compaction summaries do not inherit occurrence identity just
   because their prose mentions a ghost. Preserved typed ghost items do.
3. **Conditional insertion.** Require context generation, expected presence
   version, and a stable delivery identity with a payload digest. Check them
   and the key/count/size limits atomically; either accept the complete batch
   with occurrence receipts and a new version or reject without insertion.
   An identical retry returns its original receipt; reusing an identity with
   different data fails. A retry after compaction must not resurrect removed
   occurrences. Recover receipts consistently across supported resumes.
4. **Event barriers.** Pause the relevant context before it continues after
   every specified event. Supply a stable event identity and session logical
   clock, normalized event data, and an authoritative snapshot to the adapter.
   Selection and conditional insertion must finish before releasing the
   barrier. Child contexts need separate presence and the specified shared
   session clock/cooldown semantics. Hooks running concurrently cannot race
   around the insertion transaction or deadlock waiting on its lock.
5. **Recovery.** Disconnects, failed delivery, and restarts must require
   authoritative resynchronization before acceptance resumes. Return explicit
   unsupported mode if the negotiated host lacks any mandatory capability.
   Never reconstruct presence from transcript history or cooldown.
6. **Authority and isolation.** Preserve the trusted wrapper and escaped
   untrusted ghost envelope; return full reads through tool output. Bind the
   adapter to the isolated tenant and test scope on every operation.

## Acceptance of a host build

A host patch must pin a source commit and reproducible build command. Before
calling it supported, run real host tests for every required event, including
child start/stop and permission requests. Hold each barrier and verify that
the host cannot advance until it is released. Exercise retained and removed
ghosts after actual compaction, eviction, fork, and resume. Check the snapshot
against the actual model input, including stale-version rejection, racing
insertions, retries before/after compaction, receipt recovery, and bounds.
Then run exo-dcwx's create/delivery/read/dedup/compaction/resurfacing scenario.

None of those positive tests has been performed by this negative probe.
exo-7pi0 remains incomplete: enumeration of synchronous hooks does not prove
that all event barriers work. exo-dcwx and exo-fq27 remain open.

## Human decision

Choose the supported host distribution before implementing exo-adfe: maintain
a pinned patched Codex build for this project, or require an upstream Codex
interface and leave real-host M3 pending that dependency. This determines
whether Exomemory takes responsibility for distributing and maintaining a
host fork. The existing unmodified 0.155.1 adapter path cannot satisfy M3;
loosening the live-presence contract is not an implementation option under
the finalized decisions. M2 and SQLite failure work remain independent work
that can continue if requested while the host decision is pending.
