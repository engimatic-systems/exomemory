# zvec snapshot decision

Tested 2026-09-19 on macOS arm64. Intended project: [alibaba/zvec](https://github.com/alibaba/zvec).
Use [zvec-rust 0.7.1](https://docs.rs/zvec-rust/0.7.1/zvec_rust/), the safe Rust
wrapper over the C API linked by the upstream project. No Python or external
embedding service is required. The optional probe is separate from the SQLite
application and never registers zvec in M1.

Run `cargo run --manifest-path tests/probes/zvec/Cargo.toml --target-dir target/probes`.
Formatting and linting use the same manifest (`cargo fmt`, `cargo clippy -- -D warnings`).
The pinned Cargo.lock includes zvec-rust and zvec-rust-sys 0.7.1. The bundled
build downloaded the SDK release's native library; its reported native version
is `v0.0.0-g8321c13`, not a claim of native version 0.7.1. The tested dylib SHA-256
is `e6f73866d619621abbed35a5a6c25982e439211cfa605144963271006630043f`.
The [SDK build script](https://docs.rs/crate/zvec-rust-sys/0.7.1/source/build.rs)
resolves optional external libraries before downloading its release binary.
Production integration must pin and verify the native artifact as well as the
Rust lockfile, and provide the native loader path for installed executables;
`cargo run` supplies the loader environment in this probe.

## Bounded experiment

The fixture has three supplied two-dimensional FP32 vectors, including a tied
pair. A Flat L2 collection stores full fixture bodies with revision-addressed
primary keys. It is flushed, closed, then opened read-only three times. Each
open executes 100 identical queries and full reads. Every ordered result is
`a@1: 0, b@1: 2, c@2: 2`. A separate pending generation contains `a@2` while
all queries and reads of the ready generation still return `a@1` and never
expose `a@2`. The run records manifest, snapshot hash, native version and results
under its unique tests/.runtime/ directory. Build/native downloads stay ignored
in target/probes. This is evidence for a small exact-index snapshot, not proof
of arbitrary approximate-index repeatability or crash-safe publication.

## Query and snapshot contract

- Start with supplied vectors and an explicit producer identifier/version;
  the fixture producer is `tests/fixtures/zvec-snapshot.json`, schema 1.
  No semantic text-to-vector conversion is implicit. Validate finite FP32
  components and exact dimensions. Do not normalize, round, or lowercase
  numeric inputs; include the exact query FP32 bits in query identity.
- Use exhaustive Flat squared Euclidean distance. Smaller distance ranks
  first. The probe verifies distances 0 and 2. A match has distance less than
  or equal to the explicit non-negative finite maximum; equality matches.
  All matches use a documented vector relevance band, then backend-local rank;
  distances are not compared to SQL or other backends' scores.
- Fetch every eligible row before sorting by `(distance, ghost_key, revision)`
  and applying the final result limit. Requesting only K results before fixing
  ties can omit tied keys nondeterministically. The first implementation must
  enforce a supported collection-size bound and reject oversized collections
  rather than silently claim exhaustive results. Scaling is not this probe.
- Identify a generation by SHA-256 of a canonical manifest containing native
  artifact hash, SDK version, schema/index/metric settings, producer provenance,
  and ordered `(key, revision, body hash, vector bits)` entries. The small probe
  hashes its fixed fixture bytes; production must include the native artifact
  hash and canonical encoding above. Query identity includes generation, vector,
  threshold, limit, scope filter, and search settings.
- Publish immutable generation directories. Hold the chosen generation open
  read-only for the complete query/read operation. Never mutate a published
  directory; readonly handles alone do not prevent a separate writer. Access
  control and the generation manager must enforce this ownership rule.

## Publication and full reads

exo-3m9v records the pending revision and operation identity in SQLite, builds
and verifies a new immutable external generation, then conditionally publishes
the ready pointer in a canonical transaction. Old pointers continue to identify
old full bodies while work is pending. Full reads use `(generation, key, revision)`;
missing objects are explicit errors. Failed or stale work cannot change an old
generation. Garbage collection must retain generations referenced by readers or
ready revisions. exo-upbn implements this with fault and concurrency tests;
this probe does not replace the required outbox.

The tested Rust route supports the bounded exact-search approach. No unresolved
capability gap requires another discovery ticket; native pinning, collection
bounds, generation ownership and publication remain explicit implementation
requirements of exo-upbn and exo-3m9v. Approximate indexes need new repeatability
evidence before being selected.
