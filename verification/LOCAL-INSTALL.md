# Personal Codex terminal trial

The personal installer supports macOS and verified Codex CLI 0.155.1. It enables
Exomemory in normal interactive `codex`, `codex resume` and `codex fork` sessions
under the user's home directory. This is a local trial, not a general production
installer. Noninteractive `codex exec`, desktop sessions and explicit remote
connections retain their native behavior.

Build and install:

```sh
cargo build --release --bins --locked
target/release/exo-install install --home /absolute/user/home --codex /absolute/native/codex
```

The native Codex path must point to the original executable, not the installed
launcher. The install puts launchers in `~/.local/bin` and appends marked PATH
blocks to `.zprofile` and `.zshrc`, after Homebrew and other existing shell setup.
Open a new terminal after installation; in an existing zsh, run
`source ~/.zshrc` followed by `rehash`. From any project beneath the home directory, run
`codex` as usual. The launcher passes the current directory explicitly to the
local host unless `--cd`/`-C` was supplied.

Ask Codex, for example, to remember a project fact and associate it with a topic:
“Remember that our integration tests take twelve minutes; surface this when we
plan tests.” In a later session in the same project, mention planning tests.
The SessionStart hook explains the installed `exoctl` command to the model.
The first request creates a real memory only when the user asks; installation
seeds no sample records. Individual previews remain best effort.

`exoctl help` shows the complete JSON draft grammar. The installed command is the
compiled Rust executable, not a shell wrapper. It reads persistent runtime,
scope-root, socket, tenant, and local-profile values from
`~/.config/exomemory/exoctl.json`; `EXOMEMORY_CONFIG` selects another file and
explicit command options take precedence. It derives the current directory from
the process, uses `CODEX_THREAD_ID` when supplied by Codex, and otherwise uses
the documented `terminal` session name.
Drafts still specify a directory scope, normally the current project. No global
scope is enabled. A memory rooted in project A is inaccessible from sibling B;
a deliberately chosen ancestor scope is applicable to its descendant projects.
The default permitted root is the home passed to the installer. Other filesystem
roots require separate configuration and are not enabled by this trial.

## Services, configuration and reversal

The installer creates two per-user LaunchAgents:

- `org.exomemory.data`: the persistent SQLite data service.
- `org.exomemory.codex`: the verified Codex host with a private Unix socket.

Binaries, SQLite data, attempt state, logs, hook configuration and installation
backups live under `~/.local/share/exomemory` with a private directory mode.
The installed client configuration lives at
`~/.config/exomemory/exoctl.json`. The installer records both it and the compiled
`~/.local/bin/exoctl` as owned files. `upgrade` migrates the earlier shell-wrapper
layout in place; `repair` reapplies unchanged owned files but refuses to replace
later user edits. Both retain the SQLite store.
No authentication files are copied for personal installation; the host uses the
user's existing Codex configuration and login. The normal model and approval
settings are preserved. Live validation uses GPT 5.6 Luna separately.

User-level hooks are merged into `~/.codex/hooks.json`. Their command is inert
unless the installed host supplies `EXOMEMORY_HOOK_CONFIG`. The installer asks
Codex for the exact current hashes and persists trust only for its eleven
matching definitions. It does not enable a blanket hook-trust, approval or
sandbox bypass. Existing unrelated hooks and project configuration remain active.
The host insertion path and hooks follow the official
[App Server](https://learn.chatgpt.com/docs/app-server) and
[hooks](https://learn.chatgpt.com/docs/hooks) interfaces.

```sh
exomemory status
exomemory repair
exomemory repair-path
exomemory stop
exomemory start
exomemory uninstall
```

Status checks the inherited PATH and a fresh interactive login shell before
checking services and hook trust. A native Codex path, alias override or missing
launcher makes it fail with an actionable diagnostic. `repair-path` repairs
older installations or moves the owned blocks after later shell setup; repeating
it does not duplicate the blocks. Custom ZDOTDIR setups are checked but need their
startup files configured explicitly. Unhosted SessionStart hooks now report
inactive recall on stderr, while remaining successful and injecting no context.

Stop unloads the two
LaunchAgents; start loads them again. Uninstall stops the services, removes only
the installed hook handlers, removes owned PATH blocks while preserving later
user edits, and restores preexisting
launcher/service files when
unchanged, and preserves user edits. Stored memories and backups remain in the
private runtime. Unreferenced hook hash entries may remain in Codex configuration;
they do not trust different future definitions. Reinstall after uninstall reuses
the stored records. While stopped, the `codex` launcher needs `exomemory start`;
uninstall restores the native command path.

## Verification

`cargo test --test install` checks private-runtime requirements, test/local
profile separation, directory boundaries, compiled-client identity, clean-shell
configuration discovery and precedence, session fallback, legacy-wrapper
upgrade/repair, launcher routing/current directory, existing configuration
preservation, inert hooks outside the managed host and uninstall without data
loss. Fixtures use unique `tests/.runtime/` homes and `--no-start`, which never
touches launchd.

`cargo test --test codex_live installed_hooks -- --ignored --nocapture` runs the
actual installer in an isolated home, starts its two binaries as owned test
processes, persists exact hook trust without bypass, and asks GPT 5.6 Luna to
retrieve an unsolicited preview through the installed CLI. The initial passing
run was `tests/.runtime/rc7197bc7335c`; its evidence is
`harness/installed-evidence.json`. The temporary authentication copy is removed
on completion. Existing app-server compaction tests cover the same adapter loop.

The login-shell regression in `tests/install.rs` reproduces Homebrew preceding
the launcher after `.zshenv`, checks status even when its inherited PATH looks
correct, verifies that `codex` actually receives the local `--remote` arguments
from a fresh shell, repairs a legacy manifest, and tests idempotence/uninstall.
