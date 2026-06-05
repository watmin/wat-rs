# DESIGN — Stone 245.7: the leak-contained integration runner (task #151)

> Status: STRIKE-DRAWING.
> The 245-REOPEN continues (user directive; cites the arc-245 INSCRIPTION — this
> stone does not amend it). Predecessor slice: the wat-corpus green (`94261f45`,
> 217/0/53). This stone builds the TOOLING that unlocks the ~190-failure
> Rust-integration-tier triage — the foundation-trust campaign's gate-first move.

## Why

The integration tier cannot currently be RUN safely or completely:

- `scripts/green-gate.sh` deliberately excludes the integration RUN ("the full
  workspace RUN leaks processes"), gating only build-the-tests + run-the-lib. The
  ~190 clojure-ification failures have rotted INVISIBLY behind that exclusion.
- The leaky class is large and only PARTIALLY quieted: **254 test binaries**
  (250 flat `tests/*.rs` + 4 `[[test]]` module groups), **67 files** carry
  process-spawn signals (`spawn_process|run_hermetic|fork|pidfd|lifeline|ambient`),
  and the `#[ignore]` coverage is incomplete — e.g.
  `wat_process_peer_ipc_round_trip.rs` has 3 tests, only 1 ignored. A perfect
  upfront classification is unattainable; any "just run it" leaks (the user has
  reaped leaked procs twice this week).
- A single `cargo test` invocation is all-or-nothing: one hang blocks the whole
  inventory.

## What it delivers

`scripts/integration-run.sh` — a per-binary, leak-CONTAINED integration runner
that produces the **failure inventory** the triage consumes. After it exits, ZERO
processes survive it — by construction, not by convention.

Named `-run`, not `-gate` (honesty): it gates nothing until the tier is green; at
campaign close its invocation folds into `green-gate.sh` (a later stone).

## The contract decision (pinned)

**Containment over classification.** Each test binary runs in its OWN session
(`setsid`) under a `timeout`; after each binary, the runner reaps the ENTIRE
session (`pkill -s <sid>`). An un-quieted leaky test can therefore leak only
*inside* a session the runner is about to kill — escape is structurally
impossible, and a hang costs one binary's timeout, never the run.

**The mechanic is PROVEN** (orchestrator probe, this session): a deliberately
leaked child (`setsid bash -c 'sleep 300 & exit'`) survived its session leader,
was visible via `pgrep -s <sid>`, and `pkill -s <sid>` emptied the session.

## The shape

```
scripts/integration-run.sh [--all] [--timeout SECS] [--out FILE]
```

1. **Enumerate** the wat crate's test binaries: every `tests/*.rs` basename + the
   `[[test]]` module-group names from `Cargo.toml`.
2. **Tier** (default): EXCLUDE binaries whose source matches the leaky-signal
   heuristic `spawn_process|run_hermetic|fork|pidfd|lifeline|ambient` (~67 — the
   arc-170 process class; their substance is arc-170 territory). `--all` includes
   them (still contained). The heuristic is documented IN the script as a
   heuristic; containment backstops misclassification in BOTH tiers.
3. **Build once**: `cargo build --release --tests -p wat` up front, so per-binary
   runs don't race the compiler.
4. **Run each binary**: `setsid timeout <SECS> cargo test --release -p wat --test
   <name>` — capture exit code + the `test result:` line + failure names; then
   `pkill -s <sid>` (always, success or fail). Default timeout 60s.
5. **Inventory** (the deliverable): one line per binary —
   `name <TAB> status(pass|fail|timeout) <TAB> passed/failed/ignored <TAB> failing-test-names` —
   plus a footer: totals and an error-class histogram (grep the captured output
   for `NoMatchingClause|UnresolvedReference|MalformedForm|TypeMismatch|UnboundSymbol`).
   Default out: `target/integration-inventory.tsv` (transient); the orchestrator
   commits a snapshot into this arc dir as the triage baseline.
6. **Exit code**: 0 iff every run binary passed (so it can gate later); non-zero
   otherwise — but the STONE's success is the inventory, not a green exit.

## Out of scope = rejected (affirmative cuts)

- **Greening the failures** — that is the triage (subsequent stones, conferre
  each: real substrate gap vs stale test).
- **Folding into `green-gate.sh`** — only when the tier is green (campaign close);
  premature folding would turn the gate permanently red.
- **Sibling-crate corpora** — v1 enumerates `-p wat` only; `crates/wat-holon-lru`'s
  19 failures are already characterized (struct-rot, named in `94261f45`), and the
  other crates' suites are small. A later pass widens if the inventory warrants.
- **A committed leaky-binary list** — v1 computes the tier at runtime from the
  heuristic (self-maintaining); a committed excusare-style list with an anti-rot
  check is the warded follow-on IF the heuristic proves noisy.

## Verification (the load-bearing checks)

1. **Leak-safety**: snapshot stray test processes before and after the full run —
   the after-set minus the before-set must be EMPTY. This is the stone's RED/GREEN
   contract, demonstrated on the real tier (not just the probe).
2. **Completeness**: the inventory has one line per enumerated tier binary — no
   binary silently skipped (timeouts appear AS `timeout` lines, not gaps).
3. **The baseline**: the fresh failure breakdown (counts + classes) — the
   campaign's grounded input, replacing the stale "~190" estimate.

## Rooms

- `scripts/green-gate.sh` — the sibling precedent (style, header comments, the
  arc-239 why-this-exists discipline). The new script mirrors its voice.
- `Cargo.toml:88-100` — the `[[test]]` module-group entries (enumeration source).
- `tests/` — 250 flat binaries (enumeration source).
- The proven containment mechanic (above) — the script's core loop.
