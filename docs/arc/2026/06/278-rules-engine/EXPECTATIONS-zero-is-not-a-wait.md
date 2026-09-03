# EXPECTATIONS — zero is not a wait

Written **before** the strike. Every row re-run by me on a quiet box (`ps -eo args | grep -E
'cargo|nextest|release/wat'` empty first). The result cannot move these.

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | ★ **zero has no form** | `cargo nextest run --release -E 'test(probe_zero_is_not_a_wait)'` — the relocated `tests/kernel/probe_zero_is_not_a_wait.wat.bad` | the Rust gate **passes by asserting the file FAILS to load**, and the message names the axis (positive / identity element), not the sign |
| 1b | ★ **the relocation happened and nothing was weakened** | `git status --porcelain` + `grep -rn 'probe-zero-duration-disarms' wat-scripts/` | the probe is **moved** to `tests/kernel/*.wat.bad`, zero hits left under `wat-scripts/`, and `tests/lint/wat_scripts_fixes_load.rs` is **unedited** |
| 2 | ★ **the wall discriminates** | `./target/release/wat wat-scripts/scratch-pad/probe-zero-duration-control.wat` | **still passes unchanged**, `EXIT=0`, both cells FIRED. A wall that rejects everything is not a wall |
| 3 | ★ **the negative control** | force a zero at the Rust constructor; run it | the refusal fires and its message names **the identity element**, not the sign. If nothing can make it fire — STOP-3 |
| 4 | **call sites are spelled identically** | `git diff --stat -- wat/ wat-scripts/ tests/` | **exactly one file, one line**: `wat/service.wat:67`. Any other `.wat` edit is STOP-1 |
| 5 | **a measurement of zero still works** | probe: `(:wat::time::- inst inst)` on one Instant twice | returns `Duration` 0, no raise. A duration is a measurement |
| 6 | **readouts accept both types** | `(:wat::time::milliseconds (:wat::time::Millisecond 5))` → 5, and the same on a `Duration` from `:wat::time::-` | both type-check and return i64 |
| 7 | **arithmetic accepts both** | `(:wat::time::+ inst (:wat::time::Millisecond 5))` | type-checks, returns `Instant`, via `check.rs:14137` — not a new coercion |
| 8 | **the doc stops certifying the broken value** | `grep -n 'non-negative delay' src/intrinsic/kernel/resource.rs` | **zero hits.** The word is `positive` |
| 9 | **the computed-interval flush-out** | run `wat/telemetry/span.wat`'s path with `metrics-flush-after-ms = 0` | a **named raise**, message reported verbatim. Today: silent disarm. This is a finding, not a cost |
| 10 | **purity unchanged** | the arc-278 purity gate | the seven constructors keep Pure + Deterministic |
| 11 | **substrate untouched elsewhere** | `git diff --stat` | only the nine files the BRIEF names |
| 12 | **the floor** | `scripts/floor.sh`, **Summary line** | `5200 run / 5199 passed / 1 timed out`. ⛔ **The one red is EXPECTED and is not yours** |

## ⛔ ROW 12 IS THE TRAP AND IT IS DELIBERATE

The floor is **already red** at `.floor/2026-09-03T09-14-58Z/` —
`probe_async_publish::refused_subscriber_is_retried_not_dropped`, TIMEOUT 30.015s. That red belongs
to **Stone D** (the helper vocabulary), and its mechanism is proven and committed at
`wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat`: `take-one` destructively consumes
the message `wait-pending` then waits for.

**This stone must not fix it, and must not appear to.** If it goes green, something changed that the
DESIGN did not predict — that is STOP-5, and it is a finding, not a win. A green there without an
explanation is a worse outcome than the red.

## RUNTIME PREDICTION

**90–150 minutes.** A new `Value` variant threads through the checker, the purity table, and the
readout family; the count of touch-points is knowable from the BRIEF's rooms, but `Value` is matched
exhaustively in many places, so the compiler will produce a **cascade of non-exhaustive-match
errors**. That cascade is the progress meter, not a crisis — each error names the next site. Expect
the bulk of the time there, and expect it to waterfall to zero.

## TRAP-DOOR RISKS — named so they are not surprises

1. **`Value` is matched exhaustively across the tree.** Adding a variant is a wide, mechanical
   cascade. Do not stash-and-revert; walk it down.
2. **`NonZeroU64` is not `Copy`-compatible with the `i64` paths** that currently read
   `Value::Duration(ns) => ns`. `time.rs:816` and `:1331` both do this. Both need the new arm.
3. **`wat/service.wat` is frozen into the binary.** Changing its one line while the checker change is
   in flight is the BOOTSTRAP case. `fix.wat`'s STASH-DANCE header is the supported path — it is not
   a licence to hand-edit anything else.
4. **The `-ago` / `-from-now` families take `i64`, not a Duration** (`check.rs:20850-20877`). They
   look like they should be affected and are not. Do not "fix" them.
5. **The `wat-scripts` gate will go red if the probe is not relocated.**
   `every_wat_scripts_file_loads_on_the_current_runtime` type-checks every `.wat` under
   `wat-scripts/`, and the subject probe is built to stop type-checking. That red is **predicted**,
   and the only correct response is the move to `tests/kernel/*.wat.bad`. Weakening the gate, or
   deleting the probe, fails the stone.
6. **The probe at row 1 must fail to COMPILE, not fail at runtime.** A runtime panic is rung 2 — the
   value still had a form. If the best achievable is a runtime raise, that is a real result: report
   it plainly as rung 2 and say what blocked rung 3. Do not report it as success.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 4 showing `.wat` edits beyond the one line — the contract decision was then wrong, and the
  stone's cheapness was the argument for it.
- Row 2 not run. Row 1 alone proves a wall exists, not that it discriminates.
- Row 3 reported as "could not construct a zero" without showing the attempt. That is the
  `[[feedback_a_green_test_can_prove_nothing]]` shape.
- The floor reported from a piped exit code rather than the Summary line.
- Row 12 green with no explanation.
