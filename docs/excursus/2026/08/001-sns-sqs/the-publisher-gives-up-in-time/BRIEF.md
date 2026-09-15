# BRIEF — the publisher gives up in time, not in tries

**Read `DESIGN.md` beside this first.** It carries the one contract decision, **two struck rulings that
govern different things and must not be mixed**, the sibling loop that was examined and found sound,
and five trap-doors.

## The work, in one paragraph

The publisher's publish fold (`circuit.wat:2274`) is bounded by `(range 0 256)` and raises on every
non-`Accepted` outcome, so a slow topic kills the process — and since stone 2 each timeout now costs
10 s **plus a redial**, making 256 attempts a ~43-minute bound. Replace the count with a **declared
wall-clock ceiling**, and replace the six raises with a **closed outcome enum whose exhaustion variant
the match must name**, carrying `elapsed-ms` and `ceiling-ms` — never an attempt count. Surface it on
the phases line so a give-up is visible, and add the ceiling as a `:fanout::Input` field.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/fanout/circuit.wat:2274` | the fold. `(range 0 256)` — the bound you are replacing. |
| `circuit.wat:2263`–`:2271` | ⛔ the six raising arms: `_ →`"publish not Accepted", `Lost`, `Stopped`, `Closed`, **`TimedOut` ("the peer is alive and silent" — the crash the builder named)**, and the `Malformed` placeholder. |
| `circuit.wat:2278` | `"fanout: publisher batch never accepted"` — today's exhaustion raise, one level out. |
| ⭑ `circuit.wat:606`–`:692` the **ack ladder** | THE BLESSED SHAPE. `limit-ms = vis-ns/1e6` at `:667`–`:669` with the reason in a comment; `elapsed >= limit-ms → SeenRetry::Exhausted` at `:684`–`:686`. Copy the *structure*: a time bound read at the boundary, a named exhaustion, no raise. |
| `circuit.wat:747` the worker's own `(range 0 256)` | ⭐ THE SIBLING — read it and confirm what the DESIGN says: it already returns `SeenRetry::Exhausted` instead of raising. **You are not changing it.** Report that you checked. |
| `circuit.wat:1389` `:fanout::Verdict` · `:1398` `require!` | ⚠ the OTHER ruling. `Verdict` governs parent-side pollers and forbids `Exhausted`. Do **not** reuse it here and do **not** change `require!` — see the DESIGN's table. |
| `circuit.wat` `:fanout::Input` | the ceiling becomes a field. ⚠ Fields are mandatory, so every construction site grows — `:user::run*`, `run-p*`, `run-chaos*`, `run-drop*`, `delay-full-rate`, `vis-zero-reaches`. |
| `circuit.wat:2396` `:fanout::publisher-stats` | where the parent currently learns only `"publisher stats closed"`. The exhaustion has to reach the report through here. |
| `the-harness-takes-a-record/SCORE.md` | how the last stone added Option fields and converted the `:user::` constructors — the same mechanical work. |

## Implementation sketch

```wat
(:wat::core::defenum :fanout::PublishLadder :wat::enum::Pure
  :Accepted  [...]
  ;; TIME, not tries. The match cannot drop it; the payload cannot lie about why.
  :Exhausted [elapsed-ms <- :wat::core::i64  ceiling-ms <- :wat::core::i64  last <- :wat::core::String])
```

The fold's exit condition becomes `elapsed >= ceiling-ms`, read at the boundary the way the ack ladder
reads its clock (trap-door 4 — do **not** thread a clock through the nested-Tuple accumulator without
saying why). Each of the six arms stops raising and folds into the ladder's outcome, carrying `last` so
the report names which transport fact ended it.

## Blast radius

`wat-scripts/fanout/circuit.wat` only, plus the `:fanout::Input` construction sites inside it.
**Expected 0 elsewhere** — confirm rather than inherit.

## STOP triggers

1. ⛔ **STOP-1 — if the ceiling can be set below one publish deadline (10 s), say so and gate it.** A
   ceiling shorter than a single attempt is a ladder that gives up before trying, which would pass a
   naive "it no longer crashes" check while publishing nothing.
2. ⛔ **STOP-2 — if the exhaustion cannot reach the parent's report**, STOP. A silent give-up turns a
   crash into missing data with `distinct=8000` failing for no stated reason. That is strictly worse
   than today's loud death.
3. **STOP-3 — do NOT reuse `:fanout::Verdict` and do NOT change `require!`.** Different ruling,
   different governed population. The DESIGN's table says which is which.
4. **STOP-4 — do NOT touch the worker's `(range 0 256)` at `:747`.** It already has the blessed shape.
   Confirm that in the SCORE so this is not read as a one-sided fix.
5. **STOP-5 — if threading the clock requires a fourth slot on the nested-Tuple accumulator**, STOP and
   report it. `the-waiter-folds-carry-a-named-aggregate` closed that shape deliberately.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- ⭑⭑ **THE ACCEPTANCE GATE — the slow topic no longer kills the publisher:**
  ```
  printf '#fanout/Input {:n 1 :m 1 :j 1 :sub-cap 32 :fill-first? true :vis-ms 1000
    :inbox-cap 64 :delay-bp 10000 :delay-ms 11000 …}\n' | ./target/release/wat wat-scripts/fanout/circuit.wat
  ```
  Today: `exit=2`, `"publisher stats closed" ×3`. After: the run ends with a **named exhaustion
  carrying elapsed and ceiling**, and no `assertion-failed!` from the publisher.
- ⭑ **The happy path is untouched:** the record invocation → `distinct=8000;dup=0`, and every existing
  counter unchanged. A ceiling that alters a healthy run is a ceiling in the wrong place.
- ⭑ **A negative control:** with the injector **disarmed**, `Exhausted` must never be produced. A
  ladder that gives up on a healthy topic is measuring the clock wrong.
- ⚠ Report where the ceiling defaulted to and why that value.

## Shape to copy

`circuit.wat:606`–`:692` — the ack ladder, in this same file, already doing all of it. And
`exhaustion-is-a-named-variant/SCORE.md` for how that stone landed the closed-enum shape across three
ladders.
