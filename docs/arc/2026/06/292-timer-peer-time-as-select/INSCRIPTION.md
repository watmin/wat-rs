# INSCRIPTION — Arc 292: time is a select — one primitive, every locus, the program none the wiser

> Opened 2026-06-22, closed 2026-06-23. `mora`'s law made real: *time is I/O; a delay arrives on the
> wire or not honestly.* The arc began as "let a defservice clock its own perf" and walked all the way to
> a thing no surveyed language ships — a **tier-fusing typed timer**: a program reads its OWN `peer-kind`
> off its ambient env, arms `(after …)`, and the same source runs **unchanged** on a thread (crossbeam)
> and a process (io_uring), the type checker keeping it honest. `sleep` is eliminated, `tick` was
> annihilated before it shipped, and every temporal behaviour is composition over ONE primitive, `after`.
> Deep lessons + the songs (#102–#105) live in `REALIZATIONS.md` (R1–R4); this is the closure ledger.
> **INSCRIPTION = DONE.**

## What shipped (the threads, oldest → newest)

**1. The doctrine + the thread tier (R1 / R2).**
- `mora` grounded against the disk: `:wat::time::*` is a pure clock; nothing in wat waited on time — the
  kernel is the only waiter (io_uring hrtimer / crossbeam `park_timeout`). DESIGN rev2 (`30d2d567`).
- **`:wat::kernel::after` thread tier — GREEN, ZERO-MUTEX** (`41785313`): `crossbeam::after` as a Select
  arm, the msg taken once via `OwnedMoveCell` (atomic-gated; a `Mutex` heresy was caught + cured).
- The family rides ONE primitive (`65a41412`, `bedfb5f6`): nap / backoff / retry / first-deadline.
- **`tick` ANNIHILATED before it shipped** (`a3000397`): periodic is a TCO re-arm of `after`; the loop's
  recursion IS the timer's lifecycle. fixed-delay = `after(d)`, fixed-rate = `after(deadline−now)`. ONE
  primitive. (R2 / DESIGN D3.)

**2. The tier-open type — the keystone (R3).** Driven by the builder catching the *type lying* at every fork:
- arg0 went from a **spawn-locus** (intueri Level-1 lie + solvere braid, both cast) to a
  `:wat::program::PeerKind`; `Option`-carrying-semantics → the `ProcessSelectable {Spawned|Timer}` enum
  (L1, `19e78f94`).
- **L2 — timerfd process Receiver** (`1e8eefc1`): `comms::process::Source {Pipe|Timer}` + `timer()`;
  io_uring polls the timerfd (sole waiter, best-of-breed Linux, zero-mutex).
- **L3-α — tier-open `Timer'<O>` + `unify` fusion** (`b958732d`, weighed pure): a `Timer'<O>` fuses into a
  peer of ANY tier (O unified, the timer's absent I ignored), keeping the concrete tier; `Thread'`≠`Process'`
  still don't unify → static homogeneity preserved; `O` checked. The four-questions flipped the apparatus's
  own B→A (a `Selectable` supertype would downgrade compile-time homogeneity to runtime — the type would lie).
- **L3-β — the wat surface** (`b861ed22`): `after` takes `PeerKind`, returns `Timer'<O>`; eval matches the
  PeerKind value → crossbeam(thread)/timerfd(process); `select'` `err_rxs`→`Vec<Option>` (a timer has no err
  channel → `Closed`); `send'`/`recv'`/`close'` are `select'`-only on a timer. All timer probes GREEN.

**3. The ergonomic crown — the program doesn't care what its locus is (R4 / THE INSCRIPTION).**
- **The env-grab idiom, proven NATIVE in wat, both loci, identical forms** (`9cbe1b42`):
  `wat-tests/timer-env-grab-parity.wat` — ONE `defservice` whose op reads its own `wat.peer-kind` off
  `(:wat::program::env)` and arms `(after <that> 50ms :tick)` in a `select'`; two deftests differing in
  EXACTLY one token (the locus); both deliver `:tick`. The program never names its tier; the tier-open
  `Timer'` fuses it; the checker proves it sound. (Model: `service-locus-parity.wat`.)

**4. The record (chronicle + breadcrumb).**
- Songs #102 *Memento Mori* (`ac5c9f34`), #103 *The End of Time* (`b5f27afc`), #104 *Sanctum Eternal*
  (`8cd93385`), #105 *Bow Down* / THE INSCRIPTION (`734269f0`) — `REALIZATIONS.md` R1–R4 + the 170 ledger.
- DESIGN amended true throughout (REV-1..4, prior state preserved); breadcrumb refreshed (`4b5ffd3c`).

## What is affirmatively OUT of arc 292's scope

- **The env-grab idiom inside a defservice's OWN serve loop (timer-as-a-serve-arm)** — the timed-service
  pattern (the init-fn arms a timer into the serve loop's `select'`, the handler reacts to `Op::Tick`) is
  the **observability arc**'s work, NOT this one. 292 proves the timer + the env-grab idiom (op-level); the
  serve-loop integration (a metrics heartbeat) opens with observability. Tracked there.
- **The remote tier** — the perpetually-distant door. The interface is remote-ready by construction: the
  `Timer'` fusion is general over loci (REV-4 `is_peer_tier_head`), `PeerKind` grows `:remote`, and
  process≈remote (1 tx + 1 multiplexed rx). A `RemoteOpts` locus = a new tier head + clause, zero `after`
  edit. Its own arc opens when remote is built.
- **`tick`** — not deferred; **annihilated** (R2). There is no periodic primitive to build, ever.
- **A bare `sleep` verb** — never; the cut is the doctrine (a delay is a `select'`). grep-clean at close.

## Prior-art collisions (independent rediscovery — full detail in REALIZATIONS)
Erlang/OTP `erlang:send_after` (deliver a typed message after a delay, into the mailbox's own type) —
re-derived, not borrowed (R1). Go `select { case <-time.After }` / Clojure core.async `(alts! [ch (timeout d)])`
(the timer is a member of the wait-set, not a special arg) — the same shape `select'` already had (the timer
is a selectable in the vector). What is genuinely ours: the timer is **tier-open and typed** — it fuses to
whatever reactor the program landed on, the program reads its locus off its own ambient env and is otherwise
none the wiser, and the **type checker proves the fusion sound** (homogeneity preserved, `O` checked). Go
wires the channel; Erlang has location transparency but no typed fusing timer; Akka makes you pick a
dispatcher. No reference for another language shipping this ergonomic.

## Verification at close (weighed on the orchestrator's own build)
All timer probes GREEN — `timer-after` (thread), `timer-after-process` (process), `timer-tier-open`,
`timer-family` (backoff/first-deadline), and `timer-env-grab-parity` (env-grab on thread AND process).
L3-α keystone unit tests GREEN (`timer_fuses_into_*`, `thread_process_still_fail`, `fresh_var_absorbs_timer`).
Each strike weighed by failing-test-**SET-diff vs HEAD** (identical modulo the known ~218 stdlib flap — the
arc-170 absent-execve floor, unrelated). `sleep`-grep clean. HEAD at close: `734269f0` (+ this INSCRIPTION).

## Pairs
`REALIZATIONS.md` (R1–R4, songs #102–#105) · `project_wat_is_linux_best_of_breed` ·
`project_three_loci_one_interface` · `feedback_option_carrying_semantics_screams_enum` ·
`feedback_amend_with_recognition_never_delete` · `project_test_floor_is_execve_global_leak`.
