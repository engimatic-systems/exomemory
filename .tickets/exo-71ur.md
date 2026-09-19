---
id: exo-71ur
status: closed
deps: []
links: []
created: 2026-09-19T18:25:14Z
type: bug
priority: 1
assignee: coopwilliams
tags: [install, codex]
---
# Fix login-shell launcher routing and detect inactive recall

## Background in Plain English — WHY

Homebrew prepends its binaries after .zshenv, so a normal login shell selected native Codex instead of the installed launcher. Storing memories still worked through exoctl, but recall hooks silently did nothing outside the installed host. Existing status checks verified services without verifying the command users actually launch.

## Outcomes / Acceptance Criteria — WHAT

1. Installation and a repair command put the launcher ahead of Homebrew for new login and interactive zsh sessions while preserving existing shell setup.
2. Uninstall removes the owned PATH changes while preserving later user edits; repeating repair does not duplicate the block.
3. Status fails with an actionable diagnostic when either inherited PATH or fresh login-shell resolution selects something other than the installed launcher.
4. An unhosted SessionStart reports inactive recall on stderr while retaining an empty successful hook response.
5. Isolated tests reproduce the ordering failure and verify repair, diagnostics and reversal. Apply the repair and updated binaries to the existing installation without changing memories or restarting active sessions.

## Methodological Suggestions — HOW

Use marked shell blocks and the existing installation manifest. Check actual zsh command resolution rather than assuming the caller's PATH represents a fresh terminal. Another sound approach is acceptable.

## Future work

No additional future work is identified. The original installation contract remains tracked in exo-tuc0.

## Definitions

PATH: ordered directories used to locate executable commands. Login shell: a shell that reads .zprofile after .zshenv. Launcher: the wrapper that connects Codex to the host where recall hooks can inject previews.

## Notes

**2026-09-19T18:30:14Z**

Reproduced native Codex winning in a fresh zsh login shell. Added reversible .zprofile/.zshrc PATH blocks, idempotent repair-path, inherited and fresh-shell routing checks before healthy status, and a nonblocking unhosted startup diagnostic. Regression reproduces the legacy installation, checks actual --remote launcher arguments, status failures and uninstall preserving edits. All 42 offline tests plus fmt/clippy pass. Updated installed binaries and repaired the user shell; /bin/zsh -lic now resolves /Users/ravd/.local/bin/codex and status confirms both services plus 11/11 trusted hooks. Memories and active sessions untouched.
