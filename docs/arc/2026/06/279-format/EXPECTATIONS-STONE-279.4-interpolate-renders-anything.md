# EXPECTATIONS — 279.4 · `interpolate` renders anything

**Written before the strike, 2026-08-17, against `8b2cdbf2`** (279.3 green: floor 4697/4697,
clippy 0, ignores 13). Fixed here so the result cannot move the goalposts.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 0 | ★ the raise exists **before** the change | the new kept test, run pre-edit | **FAILS**, with a `TypeMismatch` naming `expected: "String \| i64 \| f64 \| bool \| u8"` |
| 1 | the record renders after | same test, post-edit | passes; **named** fields (`{:x 1}`), not `{:field-0 1}` |
| 2 | ★ the String arm still renders bare | `(interpolate "hello {name}" :name "world")` | `hello world` |
| 3 | the partial renderer is GONE | `grep -rn 'render_unquoted' --include=*.rs .` | **0 lines** |
| 4 | one door, three callers | `grep -rn 'render_str_total' --include=*.rs src/` | def + `eval_str` + `eval_string_join` + `eval_string_interpolate` |
| 5 | 156 sites unbroken | floor + `every_wat_scripts_file_loads_on_the_current_runtime` | 0 FAIL |
| 6 | the checker's comment reads true | `sed -n '13930,13936p' src/check.rs` | describes behaviour that now exists |
| 7 | floor | `scripts/floor.sh` → Summary line | **≥4698 passed, 0 failed, 0 timed out** |
| 8 | clippy | `cargo clippy --release --all-targets` | **0** |
| 9 | ignores held | the `#[ignore]` grep | **13** |

Row 7's baseline: `8b2cdbf2` ran **4697**. The kept test adds at least one row, so **4698+** is the
expected shape. A *lower* count is a finding, not a rounding.

## Independent prediction

**Runtime: 20–35 minutes**, dominated by two release builds. The edit is one call-site swap plus one
function deletion — net **negative** lines in `src/`. The test fixture is the only authoring work.

**Time-box: 70 minutes** (2× upper bound).

## Trap doors — named before the strike

1. **Row 0 gets skipped.** The likeliest failure and the one that makes the whole stone vacuous. A
   rider that changes `src/` first and *then* writes the test proves nothing: rows 1 and 2 would
   pass on a no-op. The brief orders it first for this reason.
2. **The type registry is dropped at the new call site.** Silent: records degrade to `{:field-0 1}`
   and only row 1's *named-fields* clause catches it. This is the third time this exact argument has
   had to be threaded (`str`, `join`, now `interpolate`) — which is the argument for the door.
3. **`render_unquoted` is left in place, unused.** Then clippy's `dead_code` should fire (it did for
   `render_str_total` mid-flight during 279.3) — but it is `pub`, so **dead_code may NOT fire**.
   Row 3 is a grep for exactly this reason and must not be inferred from a green clippy.
4. **The five diagnostics goldens shift again.** Deleting `render_unquoted` removes ~18 lines from
   `string_ops.rs` — a *different* file from `runtime.rs`, so the `runtime.rs:24781` pins should NOT
   move. If they do, something else changed and it is worth knowing.
5. **A `deftest` fixture needs a record and gets a `defrecord` wrong.** Ordinary authoring friction,
   not a stone risk; the fixture file already has working `deftest` shapes to copy.

## What would make me call this Mode B

- Row 3 not clean — the function widened, deprecated, or left unused instead of deleted. That ships
  the right output through the shape the stone exists to remove.
- Row 0 unreported, or reported as "I reasoned it would fail."
- Row 2 made to pass by special-casing `Value::String` at the `interpolate` call site rather than
  relying on the door's first arm.
- Any `#[ignore]` added, for any reason.

## What I will re-run myself before committing

Rows 2, 3, 4, 7, 8, 9 — independently, on my own invocation, per FM 9. The rider's numbers are a
hypothesis until my own run agrees. Row 0 I cannot re-run after the fact by construction, which is
exactly why the brief requires its output captured **verbatim** rather than summarised.
