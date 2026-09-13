# SCORE — the rope can be looked at

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `d161c42fd` (DRAWN). Did not commit.

```
     Summary [ 521.773s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-13T02-48-24Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` → **CLIPPY=0**.
`cargo nextest run --release --no-run` → **NORUN=0**.
`git diff --stat -- wat/` is **EMPTY**. `src/` is this stone (DESIGN: first this session to reach it).
`close`'s `:restricted-to` and `take()` are **untouched**. Did not retarget `signal_kill_produces_close_outcome_signaled.{wat,rs}`.

---

## ⭑ THE HEADLINE — both variants, from wat, and the peer is still there

```
./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/scratch-pad/probe-lineage-status.wat
```

```
running=None;running2=None;process-closed=Some(Closed Some(0));process-closed2=Some(Closed Some(0));thread-closed=Some(Closed None);signaled=Some(Signaled 9);signaled2=Some(Signaled 9);stopped=Some(Failed Process peer stopped by signal 19);panicked=Some(Closed None)
EXIT=0
```

| cell | observed |
|---|---|
| still running | `None` |
| peek twice while live | `running=None;running2=None` |
| process `Closed` | `Some(Closed Some(0))`; second peek agrees |
| thread `Closed` | `Some(Closed None)` |
| ⭑ `Signaled` | `Some(Signaled 9)` after `Signal::Kill`; second peek agrees |
| ⭑ `Failed` | `Some(Failed Process peer stopped by signal 19)` after `Signal::Stop` — **not Signaled** |
| panicking thread | `Some(Closed None)` — **the disk, not the brief** (see below) |

`19` is SIGSTOP. Signaled means terminated; a frozen child is Failed. STOP-5 holds.

---

## WHAT LANDED

`:wat::kernel::lineage-status` — `Option<CloseOutcome>`, **no** `:restricted-to`, `with_ref` never `take`.

- process: `Pidfd::peek_status` — `WEXITED | WSTOPPED | WNOHANG | WNOWAIT`. Does **not** use `try_wait` (that reaps).
- thread: `JoinHandle::is_finished`. Does **not** join.
- ExitStatus map copied from `eval_peer_close_prime` (Closed / Signaled / Failed-stopped).
- `infer_lineage_status` — same Thread|Process surface as `close`, return is `Option`, not must-use.
- `CloseOutcome` still `Purity::Pure`.

Registered unrestricted, alphabetically, on the checker-skip ledger (bespoke infer, no TypeScheme — same as `close`).

---

## ⚠ THE BRIEF SAID A PANICKING CHILD → Failed. THE DISK SAYS Closed.

A wat `assertion-failed!` / `raise!` is `panic_any`, caught by the worker's `catch_unwind`. The worker then **returns normally**, so `join` is `Ok(())` and `close` maps that to `Closed`. A process panic `_exit`s `EXIT_PANIC` → `Closed Some(code)`.

Failed constructors on disk: thread join-panic (almost unreachable behind catch_unwind) / wait-fail / **stopped-not-terminated**.

So row 4's panicking child is `Some(Closed None)`. Failed fires from SIGSTOP, which is also row 8. Followed the disk.

## Signal::Stop — the cause twin of Kill

The FINDING named *"Signal enum has no Stop"* as why Failed was unreachable from wat. Kill already caused Signaled; without Stop the wat probe cannot freeze a child. Added `Signal::Stop` → `SIGSTOP` as the seventh variant. `eval_signal` names it explicitly (STOP-6, no wildcard). Did **not** widen `close`.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | verb exists, unrestricted | registered; **no** `:restricted-to` |
| 2 | `None` while running | `running=None` |
| 3 | ⭑⭑ `Signaled` from wat | `Some(Signaled 9)` |
| 4 | ⭑⭑ `Failed` from wat | `Some(Failed … signal 19)`. Panic is Closed on disk |
| 5 | `Closed` both tiers | process `Some(Closed Some(0))`, thread `Some(Closed None)` |
| 6 | ⛔ does not consume | second peeks agree; cell not taken |
| 7 | both tiers | neither raises |
| 8 | `Signaled` ≠ stopped | SIGSTOP → Failed, not Signaled |
| 9 | ⛔ `close` untouched | restrict + take still only on `close` |
| 10 | `CloseOutcome` still Pure | yes |
| 11 | floor 5237 | `.floor/2026-09-13T02-48-24Z/` |
| 12 | tests compile | NORUN=0 |
| 13 | clippy 0 | CLIPPY=0 |
| 14 | happy / chaos | `distinct=8000;dup=0` · CHAOS=0 |
| 15 | FINDING corrected | item 3 already split at DRAW; matrix Signaled/Failed now **FIRES (scratch)**; Send/TrySend Closed stay **UNREACHABLE BY DESIGN** |

Did not retarget the arc-170 Kill fixture (STOP-6).

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `d161c42fd` + the working tree.

```
floor   .floor/2026-09-13T03-00-06Z/  Summary [ 518.705s] 5237 tests run: 5237 passed, 22 skipped
        0 failure tokens · no ARM.txt
clippy  --release --workspace --all-targets -D warnings → exit 0   (this stone reaches src/)
probe   running=None · process-closed=Some(Closed Some(0)) · thread-closed=Some(Closed None)
        signaled=Some(Signaled 9) · stopped=Some(Failed … signal 19) · panicked=Some(Closed None)
happy   distinct=8000;dup=0        chaos  distinct=100;dup=0
blast   8 src/ files + 1 probe · git diff --stat -- wat/ EMPTY
```

**STRUCK. 15 rows, and the strike beat my brief on the one thing that would have broken the stone silently.**

## ⭑⭑⭑ ROW 6 — MY BRIEF TOLD IT TO USE `try_wait`, AND `try_wait` REAPS

This is the finding of the grading. My sketch said *"process: `try_wait()` → …"*. The strike did not:

```rust
/// `waitid(P_PIDFD, fd, WEXITED | WSTOPPED | WNOHANG | WNOWAIT)` —
/// child is left waitable so a later `wait_status` / `close` can reap.
/// Distinct from [`Self::try_wait`], which uses `WEXITED | WNOHANG`
/// without `WNOWAIT` and therefore REAPS. A peek that reaps would …
pub fn peek_status(&self) -> std::io::Result<Option<ExitStatus>>
```

⛔ **Had it followed my sketch, the stone's central contract — "does not consume" — would have been broken
in the one way the probe could not see:** a reaped child still answers a second peek from its cached exit
status, so `running2`/`signaled2` would have *agreed* while the lineage had in fact been consumed and
`close` could no longer reap it. The row would have passed and the verb would have been `close` with a
softer name.

★ Thread tier matched: `is_finished()` over `join.as_ref()` (`src/kernel/peer.rs`), and `None` on the
handle ⇒ already reaped ⇒ finished. **Never joins, never drains.**

## Rows verified at the true declaration sites

**Row 1 ✅ unrestricted.** The mechanism is a Rust attribute, `#[restricted_to(…)]`, and `close`'s lives at
`src/runtime.rs:25809`. `eval_lineage_status` (`:25966`) carries **none**, and `check.rs:4702` says so in
its own words: *"return is Option<CloseOutcome>. **Unrestricted.**"*

**Row 9 ✅ `close` untouched.** Zero changed lines mentioning `restricted_to` in `runtime.rs`, and
`#[restricted_to(":wat::kernel::close", ":wat::kernel::")]` still present, count 1. The `take()` consume is
unchanged. ⭑ **Arc 259 S2d is intact** — which was the whole reason this stone exists in this shape.

**Row 10 ✅** `Purity::Pure`. **Rows 2–8 ✅ my own run**, byte-identical to the strike's, true exit 0 —
including peek-twice agreement on three different end-states.

## ⚠ MY BRIEF WAS WRONG ABOUT THE MECHANISM AGAIN — fourth time this session

Row 4 said *"a child that **panics**, then peek → `Some(Failed …)`"*. The disk says **`Closed`**: a wat
`assertion-failed!` is `panic_any`, caught by the worker's `catch_unwind`, so the worker returns normally,
`join` is `Ok(())`, and that maps to `Closed`. My run confirms: `panicked=Some(Closed None)`.

`Failed` fires from **SIGSTOP** instead — *stopped-not-terminated* — which was my own row 8. So rows 4 and 8
turn out to be **one cell**, and the honest `Failed` trigger is the one I had filed under a different row.

⭑ **And reaching it required `Signal::Stop`, which I never authorized.** The FINDING itself had named
*"Signal enum has no Stop"* as a reason `Failed` was unreachable — I carried the cell into a brief without
carrying its blocker. The strike added the seventh variant, documented it as *"the OWNER — the child is
FROZEN, not terminated"*, updated the doc comment's own count (*"Six variants, three tiers"* → *"Seven…
four tiers"*), and matched it **explicitly** — all seven names, with an `other =>` that returns a
`TypeMismatch` naming the span rather than collapsing. ⚠ `close` was **not** widened to get there.

★★ Four times this session a mechanism I asserted in a brief was wrong and the executor followed the disk:
the §2d arms *"raise"* (they redial) · *"one arm"* (two sites) · *"a panic gives Failed"* (it gives Closed) ·
*"use `try_wait`"* (it reaps). **Every one was me describing a mechanism instead of reading it**, and every
one was caught by an executor citing the file.

## ⚠ Row 15 was PARTIAL, and I finished it myself

The taxonomy split was recorded in §5 item 3 and cross-referenced from the `Signaled` row — but **the two
retired cells' own rows still read plain `UNREACHABLE`** in the status column, which is exactly where a
future ranker looks and exactly how a working wall got ranked above real work the first time. I relabelled
both:

```
Send Closed      ⛔ UNREACHABLE BY DESIGN (arc 259 S2d, "the user never holds the rope"
                    — a wall holding, NOT a gap; do not rank)
TrySend Closed   ⛔ UNREACHABLE BY DESIGN (arc 259 S2d — same wall; do not rank)
```

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | verb unrestricted | ✅ no `#[restricted_to]`; `check.rs:4702` says *"Unrestricted"* |
| 2 | `None` while running | ✅ my run, twice |
| 3 | ⭑⭑ `Signaled` from wat | ✅ my run `Some(Signaled 9)`, agrees on re-peek |
| 4 | ⭑⭑ `Failed` from wat | ✅ my run `Some(Failed … signal 19)` — **via SIGSTOP, not a panic; my brief was wrong** |
| 5 | `Closed` both tiers | ✅ process `Some(Closed Some(0))`, thread `Some(Closed None)` |
| 6 | ⛔ does not consume | ✅ **and better than briefed** — `WNOWAIT`, not `try_wait` |
| 7 | both tiers | ✅ neither raises |
| 8 | `Signaled` ≠ stopped | ✅ SIGSTOP → `Failed`; same cell as row 4 |
| 9 | ⛔ `close` untouched | ✅ restriction + `take` intact, count 1 |
| 10 | `CloseOutcome` Pure | ✅ |
| 11 | floor | ✅ my own run, 5237/5237, 0 FAIL |
| 12 | tests compile | ✅ |
| 13 | clippy | ✅ my own run, 0 — on a stone that reaches `src/` |
| 14 | happy / chaos | ✅ my own runs, `8000/0` · `100/0` |
| 15 | FINDING corrected | ⚠ **partial at strike; completed by me** — the two retired rows now carry **BY DESIGN** |

## What I'd credit above all

**It refused my sketch where the sketch was dangerous, and said why in the source.** `try_wait` would have
passed every row while quietly reaping the child. That comment in `peek_status` is now the best
documentation in the repo of why a peek is not a wait — written because the executor read `try_wait`'s flags
instead of its name.

## What this hands the next stone

The census's `UNREACHABLE` column is now **two kinds**, and only one is rankable. ⛔ Before anything else is
drawn from §5, the remaining **9** `UNREACHABLE` cells should be classified on that axis — *gap* vs *wall
holding* — because that is what decided this stone was half the size it looked, and it is what stopped a
ruling from being reversed for a coverage number.
