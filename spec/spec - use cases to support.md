# Use cases to support

These examples illustrate the eager-ghost workflow. A ghost advertises that potentially useful data exists; it does not become an instruction merely because it surfaced.

- **Long-running task coherence.** During a long-running program synthesis task like completing the ARC-AGI benchmark, the agent may build a library of purpose-built and composable tools. Instead of accumulating a ledger of these tools in an ever-expanding AGENTS.md file, the agent uses the Exomemory to track all its tools. When a new task appears, the agent notices which affordances might be needed and ghosts appear advertising the relevant tools. This saves valuable context for critical decision-making.

- **Long-running questions.** During a long-running analysis task, the agent can keep track of a very large set of open questions which may require the length of several context windows to actually answer. None of those questions needs to enter context until the moment they are relevant. When the agent reads something that triggers the memory of the question, the agent can ignore it, retrieve all its notes on the question, or simply append a small snippet to its notes without reading the existing contents.

- **Frontend preference.** The user says, “Remember to use TypeScript when making frontends.” The agent stores a directory- or global-scoped memory with triggers for frontend work. In a later session, the ghost advertises the preference as data. The current user request remains authoritative, and the agent can retrieve the full record when its scope or exceptions matter.

- **Test-suite duration.** The user asks Exomemory to remember how long the test suite took. The memory is overwritable, so a future agent can propose a new measured duration. The agent supplies the revision it read; if another run updated the record concurrently, compare-and-swap reports a conflict instead of discarding either measurement. A short blurb may contain the duration, but the underlying body and metadata share the same atomic revision.

- **Anthropology corpus.** The user asks to be reminded that an anthropology document collection exists whenever anthropology becomes relevant. A matching ghost advertises the corpus and its provenance; the agent can then decide to search it with `ls`, `rg`, or another retrieval tool. A broad corpus trigger cannot fill the hook response because count and per-source diversity limits apply.

- **Experiment location.** The user says that experiments for a particular repository live in its parent directory. The memory uses directory scope tied to that repository. When an experiment task occurs inside the scope, the ghost advertises the location; it does not apply as a global filesystem instruction.

- **Timeline reconstruction.** The agent could build up a timeline of episodic events incrementally over time, by remembering events that were important enough to recall and adding new events into that timeline. This likely would involve a timeline-oriented storage backend.
