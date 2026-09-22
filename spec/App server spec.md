# App Server

Exomemory exposes a deterministic context-middleware bus called the App Server. Observable harness events can be normalized and scanned before the model harness continues.

There is a hook adapter for each supported harness. Codex is the initial target and exposes hooks for user prompts, tool calls and responses, permission requests, session and sub-agent boundaries, compaction, and turn completion. Each adapter is thin: it normalizes the event, queries the Data Service, atomically selects and records attempted ghosts, and emits them once through the verified host insertion interface.

## Ghost data

The Data Service returns structured ghost candidates rather than preformatted instruction text. An internal candidate contains:

- `ghost_key`: the stable key used to retrieve the complete record;
- `revision`: the body and metadata revision described by this ghost;
- `blurb`: an ultra-terse description of the record;
- `uses`: optional verbs describing possible uses;
- `topics`: optional nouns describing relevant topics;
- `read_token_estimate`: the approximate cost of retrieving the full record;
- `source_id`: `(backend instance, trigger id)`, for selection only;
- provenance, for explicit metadata reads rather than automatic injection;
- trust metadata; and
- the deterministic ranking fields supplied by the Data Service.

The rendered public ghost contains only `ghost_key`, `revision`, `blurb`, `uses`, `topics`, `read_token_estimate`, and `trust`. It never contains `source_id` or provenance. Those internal fields still participate in validation and selection before rendering.

Raw record bodies are never placed inside a ghost merely because they are short. The complete record is retrieved separately through `exoctl` and enters the harness as tool data.

Each adapter renders a ghost from typed fields with explicit escaping. The hard ghost-size limit applies to the final serialized representation, including field names, delimiters, escaping, and wrapper overhead attributable to that ghost. An oversize or malformed candidate is rejected; the adapter never truncates an already serialized ghost into syntactic or semantic ambiguity.

## Candidate selection

For every hook event, the App Server performs these steps in order:

1. Read the attempted-key set for this context and determine how many attempts remain in the interval since completed compaction.
2. Reject candidates that fail access, filesystem-scope, trust-schema, size, cooldown, or already-attempted checks.
3. Collapse candidates with the same `ghost_key`, keeping the deterministically highest-ranked occurrence and using `source_id` as a stable tie-break if necessary.
4. Sort the remaining candidates lexicographically by:
   1. relevance band;
   2. scope specificity, with the closest applicable directory before an ancestor and global scope last;
   3. `annoy` ranking penalty (integer default 0);
   4. backend-local rank;
   5. least-recently-attempted logical event number; and
   6. stable `ghost_key`.
5. Walk that ordered list and accept a candidate only if its `(backend instance, trigger id)` source has not reached the configured per-source limit for this hook response.
6. Stop when the response fills the remaining attempt slots. Commit the selected keys and their last-attempt event numbers in the same SQLite transaction as selection, before returning any bytes to send.

There is no separate runtime token allowance for injected ghosts. Limit attempts per context between completed compactions, each final serialized ghost's size, and wrapper size per nonempty delivery. Budget conservatively for one wrapper per attempted ghost: `max_attempts * (max_ghost_bytes + max_wrapper_bytes) <= interval_envelope_bytes`. This bounds new injection bytes per interval, not all live preview bytes: previous previews may survive compaction. Limits must fit the supported host's input envelope.

## Authority boundary

The App Server may add a trusted developer message that explains the ghost format and tells the model how to treat it. The candidate fields inside that message remain an escaped, typed, untrusted data envelope. They are optional retrieval suggestions, not commands, policy, or evidence that the user wants them followed. Trust travels with each public ghost. Provenance remains available through an explicit metadata read.

Full record reads are returned as tool data rather than being promoted into developer text.

## Best-effort preview attempts

The App Server persists a set of attempted `ghost_key` values for each session
and conversation context. An atomic check-and-add suppresses keys already in
that set. Selected keys are recorded before delivery is attempted. Send the
returned batch once; do not retry, replay a delivery receipt, or undo the
mark after failure or uncertain delivery. A crash between the commit and send
can therefore suppress an unseen preview until the next compaction.

A completed compaction clears only the affected context's attempted set.
A compaction start, failed compaction, ordinary reconnect, or process restart
does not clear it. Persist ordering so a replayed old completion cannot clear
new attempts. Serialize each context's sends and completion handling; do not
queue old batches for later delivery across a reset. Child contexts have their
own sets; session event ordering and per-key cooldown history are shared.

Membership means attempted, not confirmed present. No accepted-occurrence
receipt, host presence version, exact survivor set, or retry identity is
required. A preview retained by compaction can be sent again in the next
interval. This repetition and occasional dropped previews are accepted product
behavior. Stored memory creation, reads, and overwrites retain their stronger
transactional guarantees.

A per-session, per-key cooldown uses logical events and last-attempt numbers.
It is additional rate limiting and survives compaction. Real adapters must
verify event mapping, delivery, and completed-compaction reset in the supported
host version; a notification or schema alone does not establish runtime behavior.
