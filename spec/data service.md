The Data Service is a persistent daemon that accepts normalized read and write requests for various storage backends and brokers connections to those backends.

The data service responds to requests from the app server with ghosts, and it responds to `xo` with the results of read and write queries.

In order to be as fast as possible, it should probably be written in Rust. It will need to handle multiple concurrent connections without a hiccup.

Supported backends each have an integration which defines rules for writing data to the backend which has been supplied by the agent, doing so in an intuitive way such that the agent barely has to worry about anything. Similarly, reading data from the backend is dead simple. Once the trigger has fired, the ghost is retrieved deterministically. Once the agent has requested metadata or the full data, that query is fulfilled deterministically. In-flight transformations on the data are trivial and deterministic, if there are any at all. If a backend is not writable, for example, a backend which searches local files using ripgrep-all, then writes fail.

The first backends supported should be:
- `SQLite` (triggers run SQL queries for ghost, `xo` queries return a row)
- `zvec` (triggers run a vector similarity search to return a ghost, `xo` queries return the data, `xo annoy` raises similarity threshold making the memory quieter)
- `redis` (triggers run a key-value query that returns a ghost, or the full data if it fits inside the ghost.)
- `episodic SQL` (triggers run SQL queries for ghost pertaining to a date or event, `xo` queries return related events before, after, or during that event)
