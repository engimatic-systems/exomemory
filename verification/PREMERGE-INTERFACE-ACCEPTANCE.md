# Pre-merge standalone interface acceptance

Ticket `exo-0kao` was accepted on 2026-09-21. The reviewed branch tip was
`e0a83abfe1758d1fe68050d8ef3c85f34c45f13e`; the real-inference run used its
immediate predecessor `e3d303d`, whose only subsequent change was bounded
LaunchAgent shutdown waiting in the installer.

## Automated gate

`./verification/check-sqlite.sh` passed from the repository root after the
installer restart fix. It ran the formatting check, workspace clippy with
warnings denied, all workspace tests (including `tests/process.rs` and seven
installer tests), and the model suite. All eleven model expectations matched:
eight historical configurations produced their intended counterexamples and
the three finalized configurations were safe.

The final installed-interface fixture retained
`tests/.runtime/r5f7e42dd72fa/harness/interface-evidence.json`. It proves that:

- `codex` and `exoctl` are compiled executables;
- bare invocation, both help spellings, parser failures, installed config,
  `EXOMEMORY_CONFIG`, and explicit override precedence behave as documented;
- create, full read, metadata read, compare-and-swap overwrite, and feedback
  work through the installed client;
- the exact delivered ghost contains only `ghost_key`, `revision`, `blurb`,
  `uses`, `topics`, `read_token_estimate`, and `trust`;
- metadata retains backend/source identity, provenance, and triggers; and
- the named selection regression continues to enforce per-source quotas.

The legacy-wrapper fixture upgraded the old shell `exoctl` and `codex` layout,
preserved configuration and SQLite data, repaired owned files idempotently,
and restored pre-existing commands on uninstall. Login-shell fixtures cover a
Homebrew-first path, native-command bypass, wrong or recursive upstream paths,
and an unavailable hosted endpoint.

## Real Codex acceptance

The following opt-in test passed with real inference:

```sh
cargo test --test codex_live installed_hooks -- --ignored --nocapture
```

It used `codex-cli 0.155.1` and `gpt-5.6-luna`. The isolated run is
`tests/.runtime/rb6489ba567aa`; its review record is
`harness/installed-evidence.json`, with protocol events in
`logs/installed-rpc.jsonl`. The installed launcher resolved in a fresh login
shell, all eleven hook definitions were trusted without a bypass, and the
model performed a full read through installed `exoctl`. Attempt event 2 was
unchanged on the second matching turn, completed PostCompact reset the context,
and the later turn resurfaced the key at event 13. The evidence records origin
metadata and reports `auth_removed: true`. All runtime, scope, database, socket,
log, and temporary authentication paths stayed within that run directory.

This demonstrates best-effort preview delivery; it does not claim that Codex
must retain or reproduce exact preview text.

## Applied personal installation

The reviewed release build was applied with:

```sh
target/release/exo-install install --home /Users/ravd --codex /opt/homebrew/bin/codex
```

The first upgrade exposed a LaunchAgent shutdown race: `bootout` could return
while the old job was still stopping, causing the immediate bootstrap to be
skipped. The installer now waits, bounded to ten seconds, for the job to leave
launchd. Targeted installer tests and the complete gate passed after the fix;
reapplying the installer completed normally.

The installed runtime binaries and commands compared byte-for-byte with the
reviewed release outputs. `~/.local/bin/exoctl` and `~/.local/bin/codex` are
arm64 Mach-O executables. Their SHA-256 values were respectively
`a4b30839d1fc81b8dec47828e2d5bb31f9b819b77aece60befc261841b385657`
and `8ae3dc4181e7896968b2118072992767424e6878ac01ee9553f7ad875c142b1e`;
the installed help SHA-256 was
`dae85a5bde9c629fc386e1415d10fc49f50824f64cc2fae34ab1168f1230d879`.
Both private JSON configuration files had mode `0600`.

A fresh login shell resolved both commands from `~/.local/bin`.
`exomemory status` reported the separately recorded official executable at
`/opt/homebrew/bin/codex`, reachable data and Codex services, Codex 0.155.1,
and 11/11 enabled and trusted hooks. The personal database contained one
memory at revision one before and after the upgrade; the acceptance procedure
did not create or modify a personal record.

Tested tool versions were Rust and Cargo 1.98.0, SQLite 3.43.2, and Codex CLI
0.155.1 on macOS arm64.

## Remaining boundary

Codex 0.155.1 has no verified persistent configuration that routes every
ordinary terminal session through the local App Server. The compiled launcher
therefore remains required and deliberately leaves explicit remote and
maintenance invocations on the official executable. `exo-qvzg` owns the Pi
adapter investigation; `exo-a81g` separately owns CI publication.
