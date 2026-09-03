# SCORE — zero is not a wait

**STRUCK.** Executor: grok, 2026-09-03. Every row re-run by me on a quiet box.

```
Summary [ 358.369s] 5207 tests run: 5207 passed (3 slow), 15 skipped
FLOOR=0        .floor/2026-09-03T19-26-21Z/        my own run
```

Grok's floor showed **3 failed**, captured the ARM, fixed both causes (loose `contains!`
assertions → `assert_eq!`; `nap` typed `Duration` → `NonZeroDuration`), and — correctly — did **not
re-floor**. I ran it. Green.

## ⛔ THE HEADLINE IS NOT THE WALL. IT IS MY CENSUS.

The contract decision — *"the seven unit constructors return `NonZeroDuration`"* — rested on one
argument: **"zero literal zero-durations exist, so the wall forbids only what nobody writes."**

That census was taken twice-blind:

1. It grepped `Nanosecond|Microsecond|Millisecond|Second` and **omitted `Minute`, `Hour`, `Day`.**
2. It searched `wat/ wat-scripts/ tests/` and **never touched `wat-tests/` — 30+ `.wat` files, an
   entire test corpus I did not search once, all session.**

What was there:

```wat
;; Zero is a valid non-negative Duration.
(:wat::test::deftest :wat-tests::time::test-duration-zero-is-valid
  (:wat::core::let [d (:wat::time::Hour 0)]
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 0ns>")))
```

**A test whose name is the direct counter-argument to the design, and the design never saw it.** Four
sites across `wat-tests/time.wat`, plus `timer-family.wat`'s `nap`.

It is also *correct* — zero **is** a valid Duration, as a measurement, which is exactly what this
stone asserts. The repair in the tree preserves the test's real subject by producing zero via
`(:wat::time::- t t)` instead of by construction. That is the design's own distinction applied to
sites the design did not know existed.

★ **Third instance this session of reporting a filtered instrument as a fact about the tree** — after
`grep -c` undercounting and "the `after` intrinsic does not validate". The other two were caught
before shipping. This one shipped, into the one claim the contract decision stood on.

## ⛔ STOP-5 — the green needed an explanation. It has one, and it is measured.

The BRIEF named one expected red: `probe_async_publish::refused_subscriber_is_retried_not_dropped`,
TIMEOUT 30.015 s at `.floor/2026-09-03T09-14-58Z/`. On grok's floor and on mine it is
`PASS [1.654s]`. The stone edited no file in that path.

**Disposition: the defect is untouched and the race did not fire.** Measured, not argued —
`wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat` re-run against the post-stone tree,
3/3:

```
gap=0;   after-drain=none; pending=1; recovered-after-naps=0;  verdict=would-return
gap=300; after-drain=got;  pending=0; recovered-after-naps=-1; verdict=SPINS-FOREVER
```

`take-one` still destructively consumes the message `wait-pending` then waits for, and still stalls
permanently when the gap exceeds the 200 ms window. **Two green floors do not dispose of it.** The
test is a coin flip that lost once and has won twice; the disposition comes from **Stone D**,
which removes the race — not from runs that happen not to hit it.

★ And that is the argument for Stone D stated as sharply as it can be: a race whose failure mode is
an *unfalsifiable hang* is worse than one that asserts, because the losing run produces no evidence
and every winning run reads like a fix.

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★ zero has no form | ✅ **rung 3** — `PASS literal_zero_wait_has_no_form`; the `.wat.bad` fails to freeze, message names positive / MEASUREMENT / COMMITMENT |
| 1b | relocation, gate unedited | ✅ probe moved, lint gate untouched. The residual grep hit is a comment, not code |
| 2 | the wall discriminates | ❌→✅ **the row was MINE and unsatisfiable — see below.** Rewritten and re-run: `thread-1ms=FIRED; process-1ms=FIRED; wall-discriminates=yes`, 3/3, EXIT=0 |
| 3 | ★ negative control | ✅ both directions; `NonZeroU64::new(0)` is `None`; message verbatim below |
| 4 | call sites spelled identically | ⚠ **true for `(Millisecond 50)`** — zero edits to the 56 sites or the 64 `:after` sites. False for the census misses, which are mine |
| 5 | a measurement of zero still works | ✅ `(time::- t t)` → `<Duration 0ns>` |
| 6 | readouts accept both | ✅ |
| 7 | arithmetic accepts both | ✅ via `check.rs:14137` path dispatch, no coercion invented |
| 8 | the doc stops certifying | ✅ `non-negative delay` → zero hits |
| 9 | computed-interval flush-out | ✅ **rung 2** — named raise, message verbatim below |
| 10 | purity unchanged | ✅ `purity.rs` untouched; classification unchanged |
| 11 | blast radius | ⚠ exceeded **by my census misses**, not by drift |
| 12 | the floor | ✅ **5207/5207, my run** — and STOP-5 above |

### Row 2 was my error, and the BRIEF compounded it

I asked the control to *"still pass unchanged, both cells FIRED"* — where cell one was
`(fire-thread 0)`. **The row required a zero wait to keep working, in the stone that removes zero
waits.** Unsatisfiable by construction. The BRIEF then ordered the file left unedited, so the
executor could not have fixed it without violating an instruction. Grok reported ❌ and left it —
the correct call on both counts.

Rewritten to the job that actually remains and matters: **a wall that rejects everything is not a
wall.** A positive duration must still fire at **both** loci, which is what makes refusing zero
meaningful rather than a blanket ban. Green 3/3.

### The message, verbatim

```
(:wat::time::Nanosecond 0): a wait must be positive; whether you wait lives in the operation,
not the magnitude of the duration — zero is a legal MEASUREMENT (:wat::time::- on equal
Instants) and an illegal COMMITMENT: it disarms the timer
```

Arc 097's sentence (`time.rs:351`) in the axis it missed.

## What landed

Constructors return `NonZeroDuration`. `(Millisecond 50)` is still `(Millisecond 50)`. `after` and
`Alarm.after` accept only the new type; the field keeps its name (rename is Stone C).
`eval_kernel_after`'s `nanos < 0` guard was **converted to a `NonZeroDuration` extraction, not
kept** — the right call: a guard that cannot fire is decorative, and the type now carries it.
`Duration` is unchanged and still the measurement.

**`no_loose_string_assert` is a new lint gate grok wrote against its own tests** after the ARM
showed five `assert!(msg.contains(...))`. That is failure-engineering unprompted — the class, not
the case — and it is now on the floor at `PASS [0.082s]`.

## The honest limit

**Literal zero is rung 3. Computed zero is rung 2** — a runtime raise, because a checker cannot
evaluate a runtime value. `wat/telemetry/span.wat:131`'s computed flush interval is exactly this
shape. That is the material's limit, not a shortfall, and it is where the ladder stops for this
stone. The raise surfaces in wat as `LociDiedError/Panic`, so at process locus a computed zero
**kills the child** rather than raising catchably — worth knowing before Stone B moves the queue's
delays onto this type.

## Still open

- **Stone B** — queue `wait-ns` → `:Immediate` / `:UpTo`; delete `sqs.wat:737`'s clamp. Unblocked now.
- **Stone C** — `Alarm :delay`, `Milliseconds`, `visible`/`unacked`.
- **Stone D** — the helper vocabulary. **Owns the race above.** The reproducer is committed and
  deterministic; it does not need the floor to cooperate.
- **S15** teardown wedge · **S16** SCORE divergence · **S17** `Duration`'s `i64` · **S18** spurious
  `Closed` · **S19** `time::+` · **S20** `wait`'s five senses.
