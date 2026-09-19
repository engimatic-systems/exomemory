The exomemory needs to expose a deterministic context middleware bus called the App Server. Every observable event can be normalized, scanned, and transformed into higher priority context before the model harness continues. 

There will be a set of hooks (and thus, an app server) for each harness that is supported. I will focus on codex here because I have the details hammered out, but this basic pattern can be extended to other harnesses that are supported. Codex exposes lifestyle hooks for user prompts, tool calls and responses, permission requests, session and sub-agent boundaries, compaction and intern completion. 

 Our middleware exposes a separate script for each hook targeted, which is a thin adapter for the daemon mediating media read/writes.  Each hook can receive the prompt from the user before it is sent and return additional context, which is inserted as extra developer context. 
 
 How is the additional context generated? What occurs is that one of multiple triggers (extendable) will occur based on the contents of the message. Hook events pass through an event normalizer, which will connect to the Data Service, a deterministic data access daemon.
 
  The Data Service will retrieve the "ghosts" of some data relevant to the trigger. A "ghost" is a line of text with a global max length, consisting of the following fields delimited by `|` symbols:

- `ghost_key`, a short hash which can be used to query the entire raw piece of data
- `blurb`, an ultra-terse description of the data (or the data itself, if it fits inside the ghost)
- `uses`, (optional) uses for the data. This can be a comma-separated list of verbs.
- `topics`, (optional). This can be a comma-separated list of nouns.
- approximate token count to read the data

All ghosts are injected into hook-specific output as additional context. A ghost is effectively a preview of some data which the model can take as an advertisement, and if the model decides it wants to "buy" that data, so to speak, it can freely do so at the expense of some of its context window.  the metadata attached to the Key is specifically designed to help the model make a decision about whether to retrieve the data in as few tokens as possible. If the metadata is implemented properly, in the majority of cases, the model will demur.

The set of ghosts that are returned will have to be rendered into an event-specific shape since all hooks have different accepted output schemas. 

Each ghost need only appear ONCE inside any given context window. In order to prevent recursive injection, the app server watches changes to the session history file for the active session. Using metadata about the context window size, it uses a fast filter to prevent the injection of any key that is already in the context window. Furthermore, a cooldown of a few messages is applied to each ghost key to ensure that delayed updates to the session history file do not trigger recursive injection.
