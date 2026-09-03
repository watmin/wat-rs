# DESIGN — zero is not a wait

**Stone A of four.** Rung 3: not "reject zero at the call site" — **zero-as-a-wait has no form.**

> Builder, 2026-09-03: *"remove this sleep 0 bullshit from my language….. we annihilate all
> deadlocks on contact - they never survive us"*

## WHY — one defect, four layers

A **mode spelled as a magnitude**, where the magnitude's identity element silently means *"don't"*:

| layer | expression | what `0` actually means | site |
|---|---|---|---|
| queue | `wait-ns 0` | "don't wait — sweep" | `sqs.wat:58`, forked at `:487` on `(<= wait 0)` |
| stdlib | `Alarm [after <- Duration]` | — | `wat/service.wat:67` |
| intrinsic | `after(Duration(0))` | — | `resource.rs:478` |
| syscall | `it_value = {0,0}` | **disarm** (POSIX) | `process.rs:1365` |

### ★ THE WALL WAS BUILT THREE TIMES AND AIMED AT THE SIGN EVERY TIME

| guard | site | rejects | admits |
|---|---|---|---|
| `unit_constructor` | `time.rs:351` | `n < 0` | **`0`** |
| `:wat::time::-` | `time.rs:772` | `ns < 0` | **`0`** |
| `:wat::kernel::after` | `runtime.rs:26462` | `nanos < 0` | **`0`** |

⚠ **An earlier draft of this DESIGN claimed "the `after` intrinsic does not validate." That was
false** — read from an empty grep and reported as a fact about the tree. `runtime.rs:26462` rejects
negatives. The correction makes the finding *sharper*: the problem is not absent guards, it is
**three guards with one shared blind spot**, which is exactly why `value.rs:293`'s deferred `u64`
stone would also have missed it. `0u64` is still zero.

And the surface **certifies** the broken value. `resource.rs:453`:

> `@arg duration :wat::time::Duration **non-negative** delay before the timer fires`

An affirmative guarantee covering the one value that does not fire.

### Measured cost

- **Two deadlocks.** `dfacde23c` — *"the wakeup is level-triggered — the deadlock is gone and the
  polls with it"* — fixed the services and left the test helpers on `wait-ns 0`. The floor timeout at
  `.floor/2026-09-03T09-14-58Z/` is the second, in the half never migrated.
  `wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat` reproduces it deterministically,
  3/3: `gap=300 → SPINS-FOREVER`, `recovered-after-naps=-1` across 100 polls × 50 ms.
- **A locus-transparency break and a teardown wedge.**
  `wat-scripts/scratch-pad/probe-zero-duration-disarms-at-process.wat`, 3/3 — thread `ns=0` **FIRED**,
  process `ns=1ms` **FIRED**, process `ns=0` **`CLOSED`**, and the program **cannot exit (124)**. Its
  control, identical but for that one cell, exits **0**.
- **A manufactured `Closed`.** The process-side arm is the substrate's word for *"the peer went
  away"* — indistinguishable from a severed connection, straight into the tracker's open
  `Closed`-after-sever item, which treats `Closed` as fatal-and-real.
- **A live clamp.** `sqs.wat:737` silently floors any sub-millisecond delay at 1 ms. Scar tissue.

### Named and deferred three times

1. `326dbc45b` — *"The rung-3 version … is named in the DESIGN so it is not re-derived, and is
   explicitly **a different stone**."* No arc, no tracker line.
2. `SCORE-the-sane-circuit.md:54` — *"the substrate defect is untouched and **wants its own stone**."*
3. `value.rs:293` — *"must uphold non-negativity as a **caller contract**. (**A future stone** makes
   this type-enforced via `u64`.)"*

## WHAT IT DELIVERS

**`:wat::time::NonZeroDuration`** — stored as `NonZeroU64`. Negative and zero are unrepresentable
**in the storage**, not upheld by a caller contract.

```
:wat::time::NonZeroDuration   a COMMITMENT you write   — NonZeroU64
:wat::time::Duration          a MEASUREMENT you take   — i64, non-negative, may be 0
```

*A duration is a measurement; a wait is a commitment. Zero is a legal measurement and an illegal
commitment.*

The algorithm in a sentence: **the seven unit constructors mint `NonZeroDuration`; `after` and
`Alarm` accept only `NonZeroDuration`; `Duration` stays exactly what `:wat::time::-` produces; the
readout and arithmetic families accept either, through the nominal-path dispatch that already
exists.**

Exemplar for that dispatch — **`src/check.rs:14137-14146`**, `infer_polymorphic_time_arith`, matching
`TypeExpr::Path` by name. `NonZeroDuration` joins that match. No coercion mechanism is invented.

Exemplar for the panic message — **`time.rs:351-359`**, verbatim:

> *"Duration must be non-negative; … direction lives in the operation, not the sign of the duration"*

The new wall is that sentence in the other axis: **whether you wait lives in the operation, not the
magnitude of the duration.**

### ⛔ NAMES — ruled by the builder 2026-09-03 after three `intueri` casts. Do not re-derive.

**`Interval` was considered and is REJECTED**, on evidence, so no one proposes it again:

- `value.rs:284` — `Duration` already defines itself as *"a non-negative time **interval**"*.
- `interval` is the prose word for `Duration` at ten sites in `time.rs`, three of them user-facing
  `@arg` lines (`:799`, `:852`, `:885`). Minting `Interval` makes every one stale.
- **`process.rs:1370` — `it_interval: {0,0}` means "no repeat", and zero is the CORRECT value there.**
  `interval` is the one word in this stack where zero is healthy.

`NonZeroDuration` over `PositiveDuration`: "positive" names sign-and-zero together and walks the
reader back into the framing that failed three times. `NonZero` names the identity element and
nothing else — and it is the word the storage already carries, so the wat name, the Rust name, and
the invariant are one word.

## ⛔ THE ONE CONTRACT DECISION

**The seven unit constructors return `NonZeroDuration`.** `(:wat::time::Millisecond 50)` is a
specification someone typed; a written zero is always a mistake. `Duration` becomes exactly what it
is — the result of measuring.

**Census backing it:** `grep` for `(:wat::time::{Nano,Micro,Milli}second 0)` and `(:wat::time::Second 0)`
across `wat/`, `wat-scripts/`, `tests/` returns **nothing**. The wall forbids only what nobody writes.

**Consequence, and it is the point:** `(Millisecond n)` where `n` is *computed* now raises when `n`
is 0. At least one such path exists — `wat/telemetry/span.wat:131` passes
`(Record/metrics-flush-after-ms rec2)` into `Millisecond`. **Surfacing those is a finding, not a
cost.** Today they silently disarm.

## FILES — `src/` plus ONE line of `wat/`

| file | change |
|---|---|
| `src/value/value.rs:296` | `NonZeroDuration(NonZeroU64)` beside `Duration(i64)`; fix the `:291-294` deferral comment |
| `src/intrinsic/time.rs:342` | `unit_constructor` → `NonZeroDuration`; reject `n <= 0` |
| `src/intrinsic/time.rs:600+` | readout family accepts either |
| `src/check.rs:20792` | register `:wat::time::NonZeroDuration`; seven constructors' `ret` |
| `src/check.rs:14137` | `infer_polymorphic_time_arith` accepts either |
| `src/rete/purity.rs:2338` | purity/determinism rows |
| `src/intrinsic/kernel/resource.rs:453` | `@arg … **positive** delay`; **not** "non-negative" |
| `src/runtime.rs:26462` | the `nanos < 0` guard becomes structural |
| **`wat/service.wat:67`** | `Alarm [after <- :wat::time::NonZeroDuration]` — **type only** |

★ **No corpus codemod.** `(Alarm :after (Millisecond 1) :op :-tick)` is spelled identically before
and after — the constructor's return type moves, the call site does not. All 56 constructor sites and
all 64 `:after` sites are untouched. `wat/service.wat` is stdlib, so `fix.wat`'s BOOTSTRAP /
STASH-DANCE header governs that **one line**; read it before starting.

## OUT OF SCOPE = REJECTED (affirmative cuts, each with a home)

- **Making `Duration` itself non-zero.** Rejected: `:wat::time::-` on two equal Instants is a
  legitimate zero. A duration is a measurement.
- **`Duration`'s own `i64` → `u64`.** Rejected *here*: `Value::Duration(-n)` from Rust stays
  constructible and `value.rs:293`'s caller-contract hole stays open. **This stone does not close
  it** — an earlier draft claimed it did, which was false. **S17.**
- **The queue's `wait-ns`.** → **Stone B**: `wait <- Queue::Wait` with `:Immediate` and
  `:UpTo [<- NonZeroDuration]`, and `sqs.wat:737`'s clamp deleted. Needs Stone A's type; blocked
  until it lands. (`Poll`/`Block` were considered and rejected: `circuit.wat:144` *"Park, don't
  poll"* makes "poll" this arc's name for the 136,485-empty-call defect, and `Block` promises a sleep
  where `sqs.wat:515`'s `:deadline-ns (+ start-ns wait)` delivers a **bound**.)
- **`Alarm :after` → `:delay`.** → **Stone C**. 64 sites, 25 files, pure wat-fix codemod, zero
  semantic change. A rename must not ride a semantics wall.
- **`Millisecond` → `Milliseconds`; `pending`/`in-flight` → `visible`/`unacked`.** → **Stone C**.
- **The helper vocabulary** — `take-one`, `wait-pending`/`wait-inflight`, `q-depth`'s `(Tuple 1 1)`,
  `accept!`, the lying comment at `sns-fanout.wat:145`, the `1`-vs-`-1` sentinels. → **Stone D.**
  This is the layer that hung the floor; it gets its own strike.
- **The teardown wedge.** `NonZeroDuration` makes it unreachable from wat; `Value::Duration(0)` built
  in Rust still reaches `timerfd_settime`. Repro committed. **S15.**
- **The SCORE divergence** — `TIMED-OUT` (09-01) vs `CLOSED` (09-03). **S16.** The old line stands
  until someone settles it.

## THE PROOF THIS STONE MUST CARRY

A wall that never fires is a wall that was deleted. Both probes exist and both must **flip**:

1. **`probe-zero-duration-disarms-at-process.wat`** — today it freezes and runs (`process-ns0=CLOSED`,
   `EXIT=124`). After the stone it **must not type-check**: `(:wat::time::Nanosecond 0)` has no form.
   Its **control must still pass unchanged** — that is what proves the wall discriminates.
2. **The negative control.** Force a zero through in Rust (`Value::NonZeroDuration` cannot hold it —
   show the constructor refuses) and show the refusal message names the axis. If no input can make
   the wall fire, it is decorative — **STOP and report.**
3. **`wat/telemetry/span.wat:131`'s computed flush interval** — report what happens when that record
   field is 0. Today: silent disarm. After: a named raise. **Report the message verbatim.**
4. **The floor**, read from the Summary line, never a piped exit code. The open red at
   `probe_async_publish::refused_subscriber_is_retried_not_dropped` is **expected to remain red** —
   it is Stone D's, not this one's. Say so explicitly; do not "fix" it here.
