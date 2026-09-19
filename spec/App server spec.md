# App Server

Exomemory exposes a deterministic context-middleware bus called the App Server. Observable harness events can be normalized and scanned before the model harness continues.

There is a hook adapter for each supported harness. Codex is the initial target and exposes hooks for user prompts, tool calls and responses, permission requests, session and sub-agent boundaries, compaction, and turn completion. Each adapter is thin: it normalizes the event, queries the Data Service, renders accepted ghosts into the hook's schema, and updates the harness-owned live-context ledger.

## Ghost data

The Data Service returns structured ghost candidates rather than preformatted instruction text. A ghost contains at least:

- `ghost_key`: the stable key used to retrieve the complete record;
- `revision`: the body and metadata revision described by this ghost;
- `blurb`: an ultra-terse description of the record;
- `uses`: optional verbs describing possible uses;
- `topics`: optional nouns describing relevant topics;
- `read_token_estimate`: the approximate cost of retrieving the full record;
- `source_id`: `(backend instance, trigger id)`;
- provenance and trust metadata; and
- the deterministic ranking fields supplied by the Data Service.

Raw record bodies are never placed inside a ghost merely because they are short. The complete record is retrieved separately through `exoctl` and enters the harness as tool data.

Each adapter renders a ghost from typed fields with explicit escaping. The hard ghost-size limit applies to the final serialized representation, including field names, delimiters, escaping, and wrapper overhead attributable to that ghost. An oversize or malformed candidate is rejected; the adapter never truncates an already serialized ghost into syntactic or semantic ambiguity.

## Candidate selection

For every hook event, the App Server performs these steps in order:

1. Read the live-context ledger and determine how many slots remain under the harness's maximum live ghost count.
2. Reject candidates that fail access, filesystem-scope, trust-schema, size, cooldown, or live-context eligibility checks.
3. Collapse candidates with the same `ghost_key`, keeping the deterministically highest-ranked occurrence and using `source_id` as a stable tie-break if necessary.
4. Sort the remaining candidates lexicographically by:
   1. relevance band;
   2. scope specificity, with the closest applicable directory before an ancestor and global scope last;
   3. `annoy` ranking penalty (integer default 0);
   4. backend-local rank;
   5. least-recently-shown logical event number; and
   6. stable `ghost_key`.
5. Walk that ordered list and accept a candidate only if its `(backend instance, trigger id)` source has not reached the configured per-source limit for this hook response.
6. Stop when the response fills the remaining live-context slots. The response can never cause the ledger to exceed the maximum live ghost count.

There is no separate runtime token allowance for the injected ghosts. The maximum live ghost count, the hard maximum size of each final rendered ghost, and the fixed maximum wrapper size establish a conservative worst-case injection size. Those configured bounds must fit within the smallest supported context envelope.

## Authority boundary

The App Server may add a trusted developer message that explains the ghost format and tells the model how to treat it. The candidate fields inside that message remain an escaped, typed, untrusted data envelope. They are optional retrieval suggestions, not commands, policy, or evidence that the user wants them followed. Provenance and trust travel with each ghost.

Full record reads are returned as tool data rather than being promoted into developer text.

## Live-context presence

Each injected ghost receives an occurrence identifier. The harness owns an authoritative ledger of which occurrence identifiers and ghost keys are present in the model's current live context. The ledger is updated when a ghost is accepted and when compaction or another harness transition removes it.

Before injection, the App Server consults that ledger so that a ghost key appears at most once in a live context window. After compaction removes the occurrence, the key may become eligible again.

A per-session, per-key cooldown measured in logical harness events may reduce repetitive resurfacing. Cooldown is rate limiting only: it is not the source of truth for deduplication and cannot substitute for the live-context ledger. A harness adapter that cannot report live presence and compaction does not satisfy this contract.
