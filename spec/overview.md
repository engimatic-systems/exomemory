# Exomemory

In humans, memory storage and retrieval is a grab bag of different highly effective methods. It feels different to remember an episode, a half-formed habit, an adage, a social security number one looked at 10 seconds ago, a memorized verse of scripture, the words to a song, the notes to a piano tune, the location of one's keys, or someone's face. What is common across all of these experiences is that, once seeded, thoughts leap unbidden into consciousness. What we do with those thoughts is up to us in that moment.

This insight should help us give LLMs useful memory. An LLM should be able to receive a ghost of a thought arising from triggers in its current context. The seed of that thought can be far in the past, creating a chain of will across context windows. The agent remains free to decide whether the ghost is useful and can provide feedback that changes how prominently the memory ranks in the future.

Exomemory is a middleware layer that provides that feedback loop. It has three broad affordances:

1. It gives an LLM coding agent small, unsolicited previews called **ghosts**.
2. It lets the agent pull the thread of a ghost to retrieve the complete record.
3. It lets the agent create memories and revise their content, triggers, and future ranking behavior.

A ghost is an optional advertisement for data, not an instruction and not the complete memory. Exomemory preserves that distinction even when its own trusted wrapper is delivered as high-priority context. Retrieved fields remain escaped, typed data with provenance; the full record is only returned if the agent runs `xo` and gets back a tool read.

For the same normalized event and the same versioned Exomemory state, candidate selection and ordering are deterministic. Negative feedback changes a deterministic ranking penalty.

The system's correctness depends on five cross-cutting contracts:

- a memory becomes visible only as a complete, atomically published revision;
- concurrent overwrite uses compare-and-swap rather than silent last-write-wins;
- the harness reports which ghost occurrences are actually present in its live context, including after compaction;
- every live context has a maximum ghost count and every final rendered ghost has a hard size limit; and
- untrusted memory content never acquires developer-message authority merely because Exomemory retrieved it.

The executable state-machine models and checked invariants live in [`verification/`](../verification/README.md).
