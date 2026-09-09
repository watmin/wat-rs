#!/usr/bin/env bash
# capped.sh — run a heavy command inside a BOUNDED, SHARED cgroup so a memory
# spike kills the work instead of the machine.
#
# WHY THIS EXISTS (2026-09-09). A run exhausted both RAM and swap; sshd went down
# and took ~30 minutes to come back. The user slice's own accounting recorded it:
#
#   user.slice memory.peak       31G   of 31.8G total   — RAM exhausted
#   user.slice memory.swap.peak  25G   of 25.4G swap    — swap exhausted, 100%
#
# Two things follow from that measurement, and both shaped this script:
#
#   1. THE HANG WAS SWAP, NOT THE KILL. 25G of swap thrash starves sshd of I/O
#      and CPU for minutes on either side of the OOM. So `MemorySwapMax` in
#      wat-heavy.slice is the load-bearing limit — `MemoryMax` alone would still
#      have let the box thrash itself unreachable.
#
#   2. THE CEILING MUST BE SHARED. memory.peak is an AGGREGATE over everything in
#      the user slice, and the realistic failure here is two heavy runs at once —
#      an orchestrator measuring while an executor runs its own floor. A
#      per-command cap does not bound that. The slice does.
#
# Limits live in the slice unit, sized there with the numbers above. This script
# only places work into it. The unit is tracked at scripts/wat-heavy.slice; it must
# be INSTALLED to take effect (it is user config, outside the repo):
#
#   cp scripts/wat-heavy.slice ~/.config/systemd/user/ && systemctl --user daemon-reload
#
# If it is not installed, this script says so and runs UNCAPPED rather than
# pretending — see the checks below.
#
# USAGE
#   scripts/capped.sh cargo build --release
#   scripts/capped.sh ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000
#
# EXIT CODE is the child's own, unmodified — including 137 when the cap kills it.
# ⛔ Never pipe this into head/tail to decide pass/fail: a pipe returns the
# PAGER's status. That is not hypothetical — the probe that verified this script
# reported 0 through `tail` and 137 direct, for the same SIGKILL.

set -uo pipefail

SLICE="wat-heavy.slice"

if [ "$#" -eq 0 ]; then
  echo "capped.sh: no command given" >&2
  exit 2
fi

# ⛔ EVERY failure path below says WHICH check failed and runs the command anyway,
# UNCAPPED AND LOUDLY. A wrapper that silently degrades to uncapped is worse than
# no wrapper: it removes the protection while leaving the reassuring name in the
# command line. Do not collapse these arms into one.
if ! command -v systemd-run >/dev/null 2>&1; then
  echo "[capped] ⚠⚠ RUNNING UNCAPPED — systemd-run is not on PATH." >&2
  exec "$@"
fi

if [ -z "${XDG_RUNTIME_DIR:-}" ] && [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
  echo "[capped] ⚠⚠ RUNNING UNCAPPED — no user D-Bus session (XDG_RUNTIME_DIR and" >&2
  echo "[capped]    DBUS_SESSION_BUS_ADDRESS both unset), so --user scopes cannot start." >&2
  exec "$@"
fi

if ! systemctl --user show "$SLICE" >/dev/null 2>&1; then
  echo "[capped] ⚠⚠ RUNNING UNCAPPED — systemd could not resolve $SLICE." >&2
  echo "[capped]    Expected ~/.config/systemd/user/$SLICE (then: systemctl --user daemon-reload)." >&2
  exec "$@"
fi

# --scope runs the command synchronously in the foreground and returns ITS status.
# Verified both directions: exit 0 propagates 0, a SIGKILL from the cap propagates 137.
exec systemd-run --user --scope -q --slice="$SLICE" \
     --unit="wat-capped-$$-$(date -u +%s).scope" -- "$@"
