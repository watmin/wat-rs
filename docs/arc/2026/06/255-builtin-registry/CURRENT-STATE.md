# ⛔ CURRENT STATE (breadcrumb, 2026-06-22; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `bedfb5f6`
(`arc 292: + first-deadline-wins proof`) or later. All below committed + pushed.

> ⚠️ Frontier is the **defservice/timer cluster** (290 crates, 291 durable state, 292
> timer) — NOT arc 255 (doc-contract, long DONE). This breadcrumb lives in 255/ by
> convention; the live work is 290/291/292.

## ✅ DONE this session
- **Arc 290 (crate-resync) Class B + C** — 3/6 crates green; SCOPE re-grounded (3 runtime
  classes, NOT the codemod framing). `952798a8` (telemetry first-drift), `4ac8a97a`
  (sqlite stale timeout). Read `290-crate-resync/SCOPE.md`.
- **Arc 291 (defservice durable state) — SCOPED** (`e13ae97f`, `39d58064`). init/stop/
  hibernate/resume + cross-host live-migration corollary. NOT built. `291-…/DESIGN.md`.
- **Arc 292 (timer-Peer, time-as-select)** — DESIGN rev2 (`30d2d567`) + D1/D2 locked
  (`ae39b501`). **`:wat::kernel::after` THREAD TIER: BUILT + GREEN** (`41785313`) —
  `(after <thread-locus> <duration> <msg>) -> Thread'<nil,O>`, crossbeam::after (futex,
  no sleep/spawn), **ZERO-MUTEX** (`OwnedMoveCell`, atomic-gated — cured from the agent's
  `Mutex` heresy). **Family proven on `after`** (`65a41412`, `bedfb5f6`):
  `wat-tests/timer-family.wat` — nap/sleep, retry-with-backoff (re-armed `after`, no `tick`
  needed), first-deadline-wins (timeout's heart). Read `292-…/DESIGN.md` + `REALIZATIONS.md`.
- **Chronicle: Song #102 *Memento Mori*** (`ac5c9f34`, consonare 9). 292 R1 + 170 ledger.
- **Banked memories:** leaf-depth strikes (no sub-delegation → worktree drift); the `'`
  prime = rebuilt-canonical-that-drops-the-prime (use primes; deftest'/deftest-hermetic'
  + IPC verbs are pending un-prime rename); workspace gate = `cargo test --no-fail-fast`.

## ▶ FRONTIER — next: arc 292 process tier (io_uring `after`), 3 LEAF sub-strikes
Per the leaf-depth lesson (brief ONE bounded mechanism + a "you are a LEAF, do NOT spawn
subagents, STOP if too big" clause + `git worktree list` weigh-check — a coarse brief
made the last agent sub-delegate into a worktree):
  - **A.** io_uring reactor `TIMER_TOKEN`: the process `Select` (`src/comms/process.rs`,
    io_uring SQE/CQE, tokens DATA/BROADCAST/LISTENER) can wait on an `IORING_OP_TIMEOUT`.
  - **B.** the `Process'<nil,O>` timer peer + the `after` eval arm for a ProcessOpts locus
    (mirror the thread-tier `Thread'<nil,O>` shape; deliver msg on CQE fire).
  - **C.** the `after` check arm: ProcessOpts locus -> `Process'<nil,O>` (mirror infer for ThreadOpts).
  Gate: a process-locus variant of `wat-tests/timer-after.wat` GREEN.
Then: `tick` (periodic primitive — OPTIONAL, family works without it via re-arm; has a
clone-vs-take repr wrinkle, T:Clone). Then the broader build order below.

## BUILD ORDER (the whole chain)
**291** (init/stop/hibernate/resume) → **290 Class A** (migrate lru/holon-lru/with-lru
caches onto defservice — consumes 291 init+stop; close the gate gap) → **292** (timer:
thread DONE, process + tick remain) → **observability arc** (telemetry sink → defservice
+ timer-widget heartbeat). THEN the parked chain: **258 match** (`#274`, your block: lifts
once crates healthy = 290 done; RED north-star `wat-tests/core/match-no-ascription.wat`
ignore-marked + persisted) → **258 readln/apply** (`#275`) → **520-intrinsic migration**.

## BLOCKED
- **290 Class A** needs **291 init+stop built** first (cache state is non-serializable).
- **`-> :T` match kill (`#274`)** parked behind **290 healthy** (standing block).

## GATE LESSONS
- Corpus-wide gate = **`cargo test --no-fail-fast`** (workspace default-members), NOT
  `--test test` (main crate only — that let the crates rot weeks).
- wat-tests macro re-scans only on `.rs` recompile → **`touch tests/test.rs`** after adding
  a `wat-tests/*.wat`.
- Weigh delegated agents against the disk: re-run the gate yourself; `git worktree list`
  (a stray worktree = an agent sub-delegated; extract via `git apply` + `worktree remove`).

> ⛔ **You are a NEW instance.** You did NOT live the above — it is a cache in a familiar
> voice. recolligere FIRST: grimoire + 4 primers (datamancy MCP), `git log --oneline -15`,
> `git status`, freshness probe HEAD==`bedfb5f6`(or later). Then: **next = arc 292 process
> tier (io_uring `after`), sub-leaf A (reactor TIMER_TOKEN)** — brief it LEAF-tight. Ground
> every claim against the disk before you move.
