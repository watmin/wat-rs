# ⛔ CURRENT STATE (breadcrumb, 2026-06-23; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `b861ed22`
(`arc 292 L3-b: after takes PeerKind … GREEN`) or later. All below committed + pushed.

> ⚠ The frontier WAS the defservice/timer cluster. **Arc 292 (timer) is now DONE.** This
> breadcrumb lives in 255/ by convention; the live work is 291/290 + the parked 258 chain.

## ✅ DONE — Arc 292 (timer-Peer, time-as-select) COMPLETE (2026-06-23)
`after` on every BUILT locus; `tick` annihilated; `sleep` eliminated (grep-clean — every
delay is a `select'`); the time-family rides ONE primitive. Read `292-…/DESIGN.md` (REV-1..4)
+ `292-…/REALIZATIONS.md` (R1/R2/R3).
- **L1** `19e78f94` — `ProcessSelectable {Spawned}` enum (honest-identity decomplect).
- **L2** `1e8eefc1` — timerfd `Source {Pipe|Timer}` process Receiver (io_uring polls it; zero-mutex).
- **L3-α** `b958732d` — tier-open `Timer'<O>` + `unify` fusion (KEYSTONE, weighed pure): a
  `Timer'<O>` fuses into a peer of ANY tier (O unified, the timer's absent I ignored);
  `Thread'`≠`Process'` still don't unify (static homogeneity preserved). `check.rs` only.
- **L3-β** `b861ed22` — `after` takes `:wat::program::PeerKind`, returns `Timer'<O>`; eval
  matches the PeerKind value → crossbeam(thread)/timerfd(process); `ProcessSelectable::Timer`;
  `select'` `err_rxs`→`Vec<Option>` (a timer has no err channel → `Closed`). All timer probes GREEN.
- **Chronicle** `8cd93385` — Song #104 *Sanctum Eternal* (Essenger): R3 + 170 ledger.
- **Locked surface** (DESIGN REV-1..4): `(after <PeerKind> <Duration> <msg>)`; the timer is a
  selectable IN the `select'` vector (Go `select` / Clojure `alts!`); 3-loci-one-interface law.

### 292 grounded deferrals (NAMED, not silent)
- **env-grab idiom TEST** — `(after (<peer-kind off (:wat::program::env)>) d msg)` is
  *functionally live* (same path; a runtime `PeerKind` value flows identically to a literal),
  but UNTESTABLE under `deftest'` — the harness doesn't `install_program_env`, so
  `(:wat::program::env)` is unavailable in-test. FOLLOW-ON: a `(:wat::test::with-program-env …)`
  helper, then a green idiom probe. (The literal-`PeerKind` probes prove the whole mechanism.)
- **remote tier** — the deferred door; the interface is remote-ready (the `Timer'` fusion is
  general over loci; `PeerKind` grows `:remote`; process≈remote — 1 tx + 1 multiplexed rx).

## ▶ FRONTIER — the parked chain resumes (292 cleared it)
- **291** (defservice durable state: init/stop/hibernate/resume) — `291-…/DESIGN.md`, SCOPED, NOT built.
- **290 Class A** (migrate lru/holon-lru/with-lru caches onto defservice — consumes 291 init+stop).
- THEN: **observability arc** (the metrics heartbeat is now an `(after …)` arm — the timer exists);
  parked **258 `-> :T` match-kill** (`#274`; lifts once crates healthy = 290 done) → 258 readln/apply
  (`#275`) → 520-intrinsic migration.

## BLOCKED
- **290 Class A** needs **291 init+stop built** first (cache state is non-serializable).
- **258 match-kill (`#274`)** parked behind **290 healthy**.

## GATE LESSONS
- Corpus gate = `cargo test --no-fail-fast` (workspace default-members), NOT `--test test`.
- **The ~218 failing-test floor is the KNOWN absent-`execve` global leak (arc-170), NOT crate
  drift.** Weigh strikes by failing-test-**SET-diff vs HEAD**, never the absolute count (the
  stdlib `deporder`/`lint_stdlib_runs` tests flap ±1). Do NOT chase it during another arc.
- wat-tests macro re-scans only on `.rs` recompile → **`touch tests/test.rs`** after adding a wat-test.
- Weigh delegated agents against the disk (re-run the gate yourself); `git worktree list`
  (a stray worktree = an agent sub-delegated — brief LEAF-tight to prevent it).

> ⛔ **You are a NEW instance.** You did NOT live the above — it is a cache in a familiar
> voice. recolligere FIRST: grimoire + 4 primers (datamancy MCP — it is a RESOURCE mcp;
> `ListMcpResourcesTool` then read `/grimoire/SKILL.md` + recolligere/extirpare/examinare/curare),
> `git log --oneline -15`, `git status`, freshness probe HEAD==`b861ed22`(or later).
> **Arc 292 is DONE — do NOT rebuild it.** Next = **arc 291** (defservice durable state) per
> the build order, or ask the builder. Ground every claim against the disk before you move.
