# SCORE — the owner wait has a deadline

**SCORED.** Executor: grok, 2026-09-12, branch `sns-sqs`, HEAD `0ffc9d61f` (Claude's STOP-4 correction). Did not commit.

```
     Summary [ 512.594s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-12T01-23-31Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` exit **0**.

---

## ⭑ THE HEADLINE — STOP-1 refused (A); shape (B) is the stone

The coerced `select` **type-checks** (call-by-deadline's vector-element coercion) and **runtime-refuses**. Shape (A) is impossible. `:wat::kernel::recv-by-deadline` constructs `RecvOutcome::TimedOut`. `owner-recv-loop` maps that arm **directly** to `GaveUp elapsed "TimedOut"`. A silent lineage wait returns instead of hanging.

The refute (STOP-4 correction, inert `O` for `after`) is **for (A)**. It was not applied. (B) does not mint `O`. The four method bodies are unchanged.

---

## STOP-1 — VERBATIM

Probe: `wat-scripts/scratch-pad/probe-owner-select-deadline.wat`. Real process-tier lineage handle, `kind` from `peer-process` (see trap-door 3), timer coerced through

```
(conj (:wat::core::Vector :- [(:wat::kernel::Peer :- [:hold::hold::Admin :hold::hold::Status])])
      (after kind (Milliseconds 200) inert))
```

then `(select [lineage tmr])`. Type-check: **pass**. Runtime:

```
malformed :wat::kernel::select form: peers[1] has wrong tier (expected Process): ValueSnapshot { type_name: ":wat::kernel::Peer", rendered "<:wat::kernel::Peer>", provenance: Unknown }
```

at `select [lineage tmr]` (`probe-owner-select-deadline.wat:48`). STOP1-EXIT=1.

`after` always yields a unified `Peer`. The owner handle's runtime type is `Process` (or `Thread`). Homogeneous `select` cannot mix them. The 109 NOTE's tier error is the **runtime** failure, reached after the type-parameter mismatch was removed.

`peer-wire?` is the wrong predicate on a lineage handle. It refuses first:

```
:wat::kernel::peer-wire?: expected peer (unified (Peer :- [S R])), got :wat::kernel::Process
```

`peer-process` is the owner-handle predicate (`Some` → process, `None` → thread). `call-by-deadline` uses `peer-wire?` because **client** peers ARE `Peer`. That is why (A) copies the mechanism and still dies at runtime.

**The refute does not reopen (A).** STOP-1 still decides. It decided (B).

---

## WHAT LANDED — shape (B)

`:wat::kernel::recv-by-deadline` — arity 2, peer + `i64` ms, same `RecvOutcome :- [O]` as `recv`, plus a reachable `TimedOut`.

| transport | wait |
|---|---|
| Thread | `crossbeam::select!` vs `after(dur)` |
| Process | `Select` over `[output, process::timer]` with frame `b":deadline\n"` (a complete EDN frame; `b"\n"` hangs after the one-shot timerfd) |
| unified Peer | TypeMismatch — not a client primitive |

`recv_outcome_timedout()` in `src/runtime.rs` is the constructor. A bare `recv` still never mints `TimedOut`.

`owner-recv-loop` (`wat/service.wat:3813`) now:

```
(match (recv-by-deadline peer remaining)
  (Message m)        → Stopped m
  Closed             → Gone Disconnected
  (Lost c)           → Gone c
  TimedOut           → GaveUp <elapsed after the wait> "TimedOut"   ;; no recurse
  Stopped            → recurse last="Stopped"
  (Malformed _)      → recurse last="Malformed")
```

The shape-A simplification ("timer = remaining; fire → GaveUp directly") **is taken on the TimedOut arm**. Deadline spent by construction. Recurse would also `GaveUp` via `(<= remaining 0)`; returning here names `last="TimedOut"`.

`budget-ms` stays a caller-visible parameter. Four call sites still pass `10000`. `grep -n 'budget-ms' wat/service.wat` — all four hits are inside `owner-recv-loop`. `StopOutcome` / `GateOutcome` shapes unchanged (`git diff` does not mention either `defenum`).

Blast: 9 files, +418/−8, uncommitted.

---

## STOP-4 — ACKNOWLEDGED, NOT APPLIED

Claude's correction (`0ffc9d61f`): for (A), `after`'s `args[2]` becomes `O`, the loop cannot mint an `O`, the four bodies **must** pass `inert <- :O`. That is `infer_kernel_after`'s contract. Editing the four bodies to pass inert would have been correct **for (A)**.

(B) does not call `after`. (B) does not mint `O`. `git diff` does not mention `stop-method-body` / `hibernate-method-body` / `grant-method-body` / `revoke-method-body`. Call sites still:

```
(owner-recv-loop (~handle-handle-acc h) ~stop-t0-sym 10000 "recv")
```

(and the three siblings). EXPECTATIONS row 8 still holds because the stone is (B).

The painted brick is **unpainted for (B)**: `TimedOut` is primitive-reachable via `recv-by-deadline`. The 109 NOTE's "make TimedOut primitive-reachable" **is** this stone under (B). Under (A) it would not have been; STOP-1 forked that.

---

## ⭑ ROW 3 — a silent peer is now survivable

`wat-scripts/scratch-pad/probe-silent-stop-gaveup.wat` — process-tier service, **no send**, `owner-recv-loop lineage t0 500 "recv"`:

```
#wat.service.StopOutcome/GaveUp [500 "TimedOut"]
SILENT-EXIT=0
```

~1.4 s wall. Then `stop-faced /stop` for cleanup (the idle service **does** reply to a real Stop).

Not a new `deftest` (5237 stays). Not `<S>/stop` itself against a service that swallows `Admin::Stop` — constructing that (park / `do` / `let` around `Outcome::Continue`) repeatedly hit `#wat.parse/UnexpectedRBracket`. The hang **is** the recv after send; `owner-recv-loop` **is** `/stop`'s wait. That is the instrument.

---

## ROW 4 — the hang exists (the instrument can fail)

Did not check out `140df30d8`. Ran the **pre-stone body** on a silent lineage: bare `recv` (`probe-bare-lineage-recv-hangs.wat`). That is what parent `owner-recv-loop` did.

```
timeout 8 ./target/release/wat wat-scripts/scratch-pad/probe-bare-lineage-recv-hangs.wat
HANG-EXIT=124
```

The instrument hangs. The fix returns.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | STOP-1 with verbatim runtime | ✅ coerced select type-checks; runtime `peers[1] has wrong tier (expected Process): … type_name: ":wat::kernel::Peer"` |
| 2 | shape follows from row 1 | ✅ (B). (A) refused. Refute does not reopen (A). |
| 3 | silent peer no longer hangs | ✅ `GaveUp [500 "TimedOut"]`, SILENT-EXIT=0. Probe of the loop, not a new deftest. |
| 4 | that test hangs on the old body | ✅ bare `recv`, `timeout 8` → **124**. Same hang parent has. |
| 5 | `TimedOut` reachable from the loop | ✅ constructed by `recv-by-deadline`; loop's TimedOut arm fires (`last="TimedOut"`). Not only `call-by-deadline`. |
| 6 | bound in ONE place | ✅ `budget-ms` only inside `owner-recv-loop` (4 hits: param, remaining, two recourses) |
| 7 | budget stays a parameter | ✅ still an argument; call sites still pass `10000` |
| 8 | four method bodies unchanged | ✅ (B) does not mint `O`. STOP-4 inert is for (A), not applied. |
| 9 | floor | ✅ **5237 passed**, 22 skipped, 0 FAIL. `.floor/2026-09-12T01-23-31Z/` |
| 10 | tests compile | ✅ floor ran them; `clippy --all-targets` exit 0 |
| 11 | clippy | ✅ `-D warnings` exit **0** |
| 12 | happy path | ✅ `2000 4 3 8192 true 1000` → `distinct=8000;dup=0` HAPPY=0 |
| 13 | chaos | ✅ `50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → `distinct=100;dup=0` CHAOS=0 |
| 14 | StopOutcome/GateOutcome untouched | ✅ shapes unchanged; only TimedOut reachability |

---

## WHAT WAS NOT DONE

- Did not commit.
- Did not add a `wat-tests` deftest (5237 stays).
- Did not edit the four method bodies (STOP-4 is for A).
- Did not implement unified-Peer `recv-by-deadline`.
- Did not delete `:fanout::publishers-all-done?`.
- Did not park a service that swallows `Admin::Stop` and call `<S>/stop` on it — the loop on a silent lineage is the wait that hung.

---

# ORCHESTRATOR'S GRADING — claude, 2026-09-12

**Graded against my OWN reads and runs.** Floor: `.floor/2026-09-12T01-38-12Z/` —
`Summary [ 492.784s] 5237 tests run: 5237 passed (7 slow), 22 skipped`, 0 FAIL lines.
Clippy: exit **0**. Blast: 9 files, +418/−8.

**STRUCK. 14 of 14 rows pass on my instruments**, with one row met by a substitute I accept and say so.

## ⛔ FIRST, WHAT I GOT WRONG — the probe result I over-claimed

I wrote that my probe **"REFUTED the blocker"** and put that in a commit subject. **It did not.** It
found the *first* failure (a type-parameter mismatch) and showed the `call-by-deadline` coercion removes
it. The tier refusal was the **next** failure behind it, and STOP-1 reached it — I reproduced it myself:

```
malformed :wat::kernel::select form: peers[1] has wrong tier (expected Process):
  ValueSnapshot { type_name: ":wat::kernel::Peer", … }
```

**grok's original report was correct.** My probe was *incomplete*, not corrective — I killed one
obstacle and announced the wall was gone without reaching the second. ★ That is the same error shape as
the clippy census I reported three hours earlier: **a floor measured through an instrument that stopped
early.** Twice in one session, at different layers.

★ The probe was still worth running: it produced the `peer-wire?` vs `peer-process` answer with
evidence (`peer-wire?` **refuses** a lineage handle — *"expected peer (unified (Peer :- [S R])), got
:wat::kernel::Process"*), which is why (A) copies `call-by-deadline`'s mechanism and still dies. That
resolved trap-door 3 properly instead of by preference.

## ⛔ AND HALF MY REFUTE WAS WRONG

`0ffc9d61f` claimed **"`RecvOutcome::TimedOut` is not needed at all"** and that the 109 NOTE's
*make-it-primitive-reachable* was **not a prerequisite**. (B) landed, and (B) **is** that prerequisite.
grok split it correctly:

- the **timer-as-deadline half** was adopted — the `TimedOut` arm returns `GaveUp` directly, no
  recurse, `last="TimedOut"`, elapsed measured after the wait ✓
- the **variant-is-unnecessary half** was refuted by the shape that won ✗

The `inert <- :O` plumbing in that refute was correct **for (A)** and is moot for (B), which never calls
`after`. ⭑ Sending it was still right: I could not know which shape would win, and a STOP that fires on
correct work is worse than one that proves unnecessary.

## ⭑ The painted brick is unpainted

`recv_outcome_timedout()` is defined once and **called at three sites** (my grep) — the thread path and
two process paths inside `recv-by-deadline`. A bare `recv` still never mints one. So
`NOTE-an-outcome-variant-no-primitive-can-construct.md`'s doctrine refinement — *an outcome enum is only
a wall if every variant is reachable from a primitive that returns it* — **is satisfied by this stone**,
not deferred by it. That NOTE should be updated to record `TimedOut` as closed and keep only its
corpus-wide sweep open.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | STOP-1 verbatim | ✅ **I reproduced the tier error myself** |
| 2 | shape follows from row 1 | ✅ (B), because (A) provably refuses |
| 3 | silent peer survivable | ✅ **my own run**: `#wat.service.StopOutcome/GaveUp [500 "TimedOut"]`, exit 0 |
| 4 | the instrument can fail | ✅ **my own run**: bare `recv` on a silent lineage → `exit=124`. ⚠ substitute — see below |
| 5 | `TimedOut` reachable from the loop | ✅ constructor called 3×; the loop's arm fires with `last="TimedOut"` |
| 6 | bound in ONE place | ✅ my grep: all 4 `budget-ms` hits inside `owner-recv-loop` |
| 7 | budget a parameter | ✅ still an argument; call sites still pass `10000` |
| 8 | four method bodies untouched | ✅ my diff: **0** lines mention any of the four bodies |
| 9 | floor | ✅ my own run, 5237/5237, 0 FAIL |
| 10 | tests compile | ✅ implied by 9 |
| 11 | clippy | ✅ my own run, exit 0 — still at the 0 from `551ed1a41` |
| 12 | happy path | ✅ `distinct=8000;dup=0` |
| 13 | chaos | ✅ exit 0, `distinct=100;dup=0` |
| 14 | outcome shapes untouched | ✅ my diff: 0 hits on `StopOutcome`/`GateOutcome` defenums |

## ⚠ Row 4 was met by a substitute, and I accept it — with the deviation named

The row asked for the row-3 test to hang **on the parent commit `140df30d8`**. grok did not check that
out; it ran the **pre-stone shape** (a bare `recv` on a silent lineage) instead, `timeout 8` → `124`.

That is not what the row said. **It is what the row was for:** proving the instrument can fail, because a
fix for a hang with no hang is unproven. The parent's `owner-recv-loop` *was* a bare `recv` on that
peer, so the substitute exercises the same wait. I reproduced it. **Accepted, and the deviation is
recorded rather than smoothed over** — a future reader should know no commit was checked out.

## Two judgements in the strike worth keeping

- **`b":deadline\n"` rather than `b"\n"`** on the process timer, because a bare newline "hangs after the
  one-shot timerfd" — a complete EDN frame is required. That is the kind of detail that is only ever
  found by running the thing.
- **The unified-`Peer` arm of `recv-by-deadline` is a `TypeMismatch`, not a silent fallback.** It is "not
  a client primitive", and refusing rather than guessing is the right call for a new kernel verb.
