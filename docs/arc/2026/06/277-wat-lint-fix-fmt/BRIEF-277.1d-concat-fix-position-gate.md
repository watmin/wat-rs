# BRIEF — Arc 277.1d: the concat-fix position gate

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** Build, run the named
tests, report. Another agent weighs independently.

## The work

Make the concat→format fix POSITION-AWARE: a bare-symbol concat inside a defmacro body rewrites to the
`:wat::core::string::interpolate` INTRINSIC (expand-time-legal, arc 284); outside a defmacro it stays the
`:wat::core::format` MACRO (zero-cost). Same template + kwargs; only the head keyword changes. This makes
the sweep safe (it broke the stdlib because `format` is refused at expand time).

## The contract — implement EXACTLY the DESIGN

Read **`docs/arc/2026/06/277-wat-lint-fix-fmt/DESIGN-277.1d-concat-fix-position-gate.md` § "The contract"**
and apply its 5 edits to `wat/lint.wat` verbatim:
1. `is-defmacro-form? [form] -> bool` — list whose head ast-name == `":wat::core::defmacro"` (guard with
   `kw-or-sym?`, mirror `concat-head?`).
2. `rule-concat-abuse-form` gains an `in-defmacro?` param; on hit → pass it to `make-concat-finding`; on
   recurse → child gets `(:wat::core::or in-defmacro? (:wat::lint::is-defmacro-form? form))`.
3. `make-concat-finding` gains `in-defmacro?`; threads to `concat-format-fix`.
4. `concat-format-fix` gains `in-defmacro?`; `head-str = (if in-defmacro? ":wat::core::string::interpolate"
   ":wat::core::format")` used as the call head in `new-text`. Everything else unchanged.
5. `lint-file` calls `(rule-concat-abuse-form form path false)`.

The ladder rule + `make-ladder-finding` are UNCHANGED (contains? is position-independent).

## Read in order
1. The DESIGN (above).
2. `wat/lint.wat` — `rule-concat-abuse-form` (~503), `make-concat-finding`, `concat-format-fix`,
   `lint-file` (~540 where it calls the concat rule), and `concat-head?`/`kw-or-sym?` (the helper shape to
   copy for `is-defmacro-form?`).
3. `tests/probe_arc277_1d_concat_fix_position_gate.rs` — remove the `#[ignore = "arc 277.1d …"]`.

## STOP triggers
1. If the defn-body case stops emitting `format` (277.1c-fix probe regresses) — STOP.
2. If any floor count beyond +1 deftest moves — STOP (additive gate only).
3. If `or`/`is-defmacro-form?` mis-flags a non-defmacro form (e.g. a defn) as in-defmacro — STOP.

## Blast radius
`wat/lint.wat` + a `wat-tests/lint.wat` deftest (defmacro→interpolate, defn→format) + un-ignore the probe.
No Rust changes. No git.

## Verify (paste output verbatim)
```
cargo test --release -p wat --test probe_arc277_1d_concat_fix_position_gate   # 1/1 GREEN
cargo test --release -p wat --test probe_arc277_1c_concat_format_autofix      # still GREEN (defn→format)
cargo test --release -p wat --test probe_arc277_1b_ladder_autofix             # still GREEN
cargo test --release --test test 2>&1 | grep "test result"                    # deftest 264 passed / 1 failed (was 263, +1)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result         # deporder 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                   # lib 929/36 (UNCHANGED)
```
Report: the diff of the 5 edits, the command outputs verbatim, any delta. Do not claim green you did not see.
