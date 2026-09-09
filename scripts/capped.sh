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
#   capped cargo build --release                     # slice ceiling only
#   capped --limit 15g wat --grep ./wat-scripts/fixes/some-fix.wat   # + a per-run ceiling
#   capped -l 4g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000
#
# TWO CEILINGS, AND THEY NEST. The slice ceiling exists so a runaway cannot take the
# BOX down; it is shared by every capped run at once. `--limit` is a ceiling for THIS
# run — "this should not need more than 15g" — and cgroup limits compose along the
# path, so the effective bound is whichever is smaller. Use --limit when you know
# roughly what a job costs and want it killed rather than allowed to grow into the
# shared headroom that is protecting your shell.
#
# ⚠ `--limit` pins swap to ZERO for that run, deliberately: 15g means 15g, and the
# thing that made the 2026-09-09 outage take 30 minutes was swap thrash, not the kill.
#
# EXIT CODE is the child's own, unmodified — including 137 when the cap kills it.
# ⛔ Never pipe this into head/tail to decide pass/fail: a pipe returns the
# PAGER's status. That is not hypothetical — the probe that verified this script
# reported 0 through `tail` and 137 direct, for the same SIGKILL.

set -uo pipefail

SLICE="wat-heavy.slice"
LIMIT=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    -l|--limit)
      [ "$#" -ge 2 ] || { echo "capped: $1 needs a size, e.g. --limit 15g" >&2; exit 2; }
      LIMIT="$2"; shift 2 ;;
    --limit=*) LIMIT="${1#--limit=}"; shift ;;
    --) shift; break ;;
    *) break ;;
  esac
done

if [ "$#" -eq 0 ]; then
  echo "capped: no command given" >&2
  echo "usage: capped [--limit SIZE] COMMAND [ARGS...]" >&2
  exit 2
fi

# Per-run properties, empty unless --limit was given.
PROPS=()
if [ -n "$LIMIT" ]; then
  # Normalise 15g -> 15G. Reject anything systemd would not take, LOUDLY and with a
  # non-zero exit: a malformed limit must never silently become "no limit", which is
  # the same class of failure as a wrapper degrading to uncapped without saying so.
  norm="$(printf '%s' "$LIMIT" | tr 'kmgt' 'KMGT')"
  case "$norm" in
    *[0-9]) : ;;                      # bare bytes
    *[0-9]K|*[0-9]M|*[0-9]G|*[0-9]T) : ;;
    *) echo "capped: bad --limit '$LIMIT' — want a number with an optional K/M/G/T suffix (e.g. 15g, 512M)" >&2
       exit 2 ;;
  esac
  case "${norm%[KMGT]}" in
    ''|*[!0-9]*) echo "capped: bad --limit '$LIMIT' — the size must be a whole number" >&2; exit 2 ;;
  esac
  PROPS=(-p "MemoryMax=$norm" -p "MemorySwapMax=0")
  echo "[capped] per-run ceiling $norm (swap 0), inside $SLICE" >&2
fi

# ⛔ EVERY failure path below says WHICH check failed and runs the command anyway,
# UNCAPPED AND LOUDLY. A wrapper that silently degrades to uncapped is worse than
# no wrapper: it removes the protection while leaving the reassuring name in the
# command line. Do not collapse these arms into one.
# ⛔ If the caller asked for an explicit --limit and we cannot honour it, that is a
# REFUSAL, not a downgrade. Running a job uncapped that the caller bounded on purpose
# is precisely the silent-degradation failure this script exists to avoid.
refuse_or_run() {
  if [ -n "$LIMIT" ]; then
    echo "[capped] ⛔ REFUSING — you asked for --limit $LIMIT and it cannot be applied here." >&2
    exit 3
  fi
  echo "[capped] ⚠⚠ A memory spike can take the box down. Watch it." >&2
  exec "$@"
}

if ! command -v systemd-run >/dev/null 2>&1; then
  echo "[capped] ⚠⚠ UNCAPPED — systemd-run is not on PATH." >&2
  refuse_or_run "$@"
fi

if [ -z "${XDG_RUNTIME_DIR:-}" ] && [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
  echo "[capped] ⚠⚠ RUNNING UNCAPPED — no user D-Bus session (XDG_RUNTIME_DIR and" >&2
  echo "[capped]    DBUS_SESSION_BUS_ADDRESS both unset), so --user scopes cannot start." >&2
  refuse_or_run "$@"
fi

if ! systemctl --user show "$SLICE" >/dev/null 2>&1; then
  echo "[capped] ⚠⚠ RUNNING UNCAPPED — systemd could not resolve $SLICE." >&2
  echo "[capped]    Expected ~/.config/systemd/user/$SLICE (then: systemctl --user daemon-reload)." >&2
  refuse_or_run "$@"
fi

# --scope runs the command synchronously in the foreground and returns ITS status.
# Verified both directions: exit 0 propagates 0, a SIGKILL from the cap propagates 137.
# --collect: a scope whose process is KILLED (by the cap, by the OOM killer, by a
# signal) is left behind by systemd in `failed` state. Observed 2026-09-09: ten failed
# wat-capped-*.scope units accumulated over one afternoon of bisecting, each needing a
# manual `systemctl --user reset-failed`. --collect reaps them.
# ⚠ NOT claimed: that the pile caused any failure. It was briefly blamed for an exit-3
# that turned out to be a parse error in the probe being run. The nanosecond unit suffix
# below is likewise hygiene against same-second collisions, not a diagnosed fix.
exec systemd-run --user --scope -q --collect --slice="$SLICE" "${PROPS[@]}" \
     --unit="wat-capped-$$-$(date -u +%s%N).scope" -- "$@"
