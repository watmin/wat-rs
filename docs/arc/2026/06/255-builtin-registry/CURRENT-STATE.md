# ⛔ CURRENT STATE (breadcrumb, 2026-06-22; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `ae39b501`
(`arc 292 sub-strike 1: RED probe + D1/D2 decisions`) or later. All below committed + pushed.

> ⚠️ The frontier MOVED this session — off arc 255 (doc-contract, DONE) onto a
> defservice/timer cluster (290 crates, 291 durable state, 292 timer-Peer). This
> breadcrumb still lives in 255/ but the live work is 290/291/292.

## ✅ DONE this session
- **Arc 290 (crate-resync) Class B + C** — 3/6 workspace crates green. The SCOPE was
  re-grounded (the codemod framing was WRONG): the real failures are runtime, three
  classes. **B** = `first`-as-Option drift (telemetry crates, `952798a8`). **C** =
  wat-sqlite stale 100ms `:time-limit` (the freeze baseline, `4ac8a97a`). Read
  `docs/arc/2026/06/290-crate-resync/SCOPE.md`.
- **Arc 291 (defservice durable state) — SCOPED + committed** (`e13ae97f`, `39d58064`).
  `init`/`stop`/`hibernate`/`resume` (gen_server's missing `init/1` + hibernation); the
  cross-host live-migration corollary. NOT built. Read `291-defservice-durable-state/DESIGN.md`.
- **Arc 292 (timer-Peer, time-as-select) — SCOPED + sub-strike 1 DONE.** DESIGN rev2
  (`30d2d567`) + **D1/D2 decisions locked** (`ae39b501`): `:wat::kernel::after`/`tick`
  (D1) ; the timer is a TIER peer, a LOCUS picks the tier — `(after <locus> d msg) →
  <Tier>'<nil,O>` (D2=B1). **RED probe `wat-tests/timer-after.wat` is RED on exactly
  `:wat::kernel::after` unknown** (everything else type-checks; B1 confirmed — a
  `Thread'<nil,keyword>` satisfies `select'`). Read `292-timer-peer-time-as-select/DESIGN.md`.
- **Chronicle: Song #102 *Memento Mori* (Lamb of God)** — `292/REALIZATIONS.md` R1 +
  `170/INTERSTITIAL` ledger (`ac5c9f34`); consonare MATCHES, fidelity 9.

## ▶ FRONTIER — next strike: arc 292 sub-strike 2 (thread tier)
Build `:wat::kernel::after`/`tick` on the **thread tier** so the RED probe goes GREEN:
- a crossbeam `after`/`tick` `Receiver` delivering the caller's `msg`, typed
  `Thread'<nil,O>`, registerable as a `thread::Select` arm (`src/comms/thread.rs`).
- the `:wat::kernel::after` intrinsic: check (infer `(<locus>, Duration, msg) →
  Thread'<nil,O>` for ThreadOpts) + eval (build the timer Thread' value).
- gate: `cargo test --test test timer` GREEN (touch `tests/test.rs` first — the
  wat-tests macro only re-scans on recompile). Then process tier (`IORING_OP_TIMEOUT`
  + `TIMER_TOKEN`), then the wat surface + the family (sleep/timeout/cron/… as wat).
Delegate the Rust build to a sonnet (>20 lines); orchestrator draws the BRIEF + weighs.
NOTE: the probe's `match -> :keyword` is the form arc-258 ss2 will codemod — fine for now.

## BUILD ORDER (the whole chain)
**291** (init/stop/hibernate/resume) → **290 Class A** (migrate lru/holon-lru/with-lru
caches onto defservice — consumes 291 init+stop; close the gate gap) → **292** (timer,
independent — can land anytime) → **observability arc** (telemetry sink → defservice +
a timer-widget heartbeat). THEN the parked original chain: **258 match** (`#274`, your
block: lifts once crates healthy = 290 done) → **258 readln/apply** (`#275`) → the
**520-intrinsic migration** (the original "tomorrow" goal).

## BLOCKED
- **290 Class A** (cache migration) needs **291 `init`+`stop` built** first (the cache's
  non-serializable state can't be eager `state0`). No `Option`-hack debt.
- **`-> :T` match kill (`#274`)** parked behind **290 healthy** (your standing block).

## GATE LESSONS (hard-won)
- Corpus-wide gate = **`cargo test --no-fail-fast`** over the workspace (default-members),
  NOT `cargo test --test test` (main crate only — that let the crates rot weeks).
- wat-tests macro only re-scans on `.rs` recompile → `touch tests/test.rs` after adding a
  `wat-tests/*.wat`.

> ⛔ **You are a NEW instance.** You did NOT live the above — it is a cache in a familiar
> voice. recolligere FIRST: grimoire + 4 primers (datamancy MCP), `git log --oneline -15`,
> `git status`, freshness probe HEAD==`ae39b501`(or later). Then: **next strike = arc 292
> sub-strike 2, the thread-tier `after`/`tick` build** (RED probe `wat-tests/timer-after.wat`
> is the gate). Ground every claim against the disk before you move.
