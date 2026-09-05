# BRIEF — STONE: `StepValue` faces `WatAST`; stop corrupting rationals and bigints

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Never operate on a path containing
`.claude/worktrees/`. Do not commit, push, stash, or revert. Do not run the full floor.

Read `DESIGN-STONE-stepvalue-is-watast-and-the-round-trip-is-lossy.md` (sibling) first.

## ⛔ The bug you are fixing — reproduce it BEFORE you change anything

```
(:wat::eval-step! (:wat::core::quote 1/2))   terminal renders  "1/2"   ← a StringLit
(:wat::core::quote 1/2)                               renders   1/2    ← a RationalLit
```

Same for a bigint. **`i64` is the control and survives.** Build a probe in
`wat-scripts/scratch-pad/` covering rational, bigint, and an i64 control, run it, and capture the
corrupted output verbatim. **A probe first seen green proves nothing** — this campaign has a memory
named for it and its own round-trip probe once returned a false perfect score.

## Rooms, in order

1. **`src/holon/ast.rs:928`, `try_recognize_holon_value`** — read its doc first. Its job is a
   PREDICATE (*"is this WatAST already a value?"*) and it answers by BUILDING a `HolonAST`. The two
   `SURPRISE` arms — `RationalLit` and `BigIntLit` lowered to `HolonAST::string(…)` — are where the
   loss happens.
2. **`src/runtime.rs:11996`, `enum StepValue`** — `Next(WatAST)` already faces WatAST;
   `Terminal(HolonAST)` and `AlreadyTerminal(HolonAST)` do not.
3. **`src/runtime.rs:12203, 12297, 12303, 12578`** — the four `holon_to_watast` conversions that
   exist only to bridge room 2. They should **disappear**, not be rewritten.
4. **`src/runtime.rs:12322`** — `try_recognize_holon_value`'s call site in the stepper, and the
   place to decide what the predicate should return.

## ⚠ The constraint that shapes the whole stone

`try_recognize_holon_value` is `pub(crate)` and **may have callers that genuinely want a holon** —
the VSA path. **Find them first.** Whatever you change must keep those working. If the honest shape
is a new value-predicate beside the existing function rather than a change to it, that is fine and
expected; say which you chose and why.

⛔ **Do not "fix" the two SURPRISE arms by inventing a holon rational or bigint leaf.** holon-rs has
none; that is a dependency fact, not an oversight. The fix is that the stepper should never have
been round-tripping through holon at all — the input `WatAST` was correct and was destroyed.

## rustc is your census, and this time it can see

Unlike `src/types.rs` (where wat types are declared as data and rustc is blind), `StepValue` is a
real Rust enum. Change its field types and the compiler names every site. Let the error list be the
worklist; watch the count fall monotonically. If a fix RAISES it, stop and report both counts.

## STOP triggers

- **STOP-1** — if a caller of `try_recognize_holon_value` genuinely needs the holon and cannot be
  served, STOP and report which. Do not break the VSA path to fix the eval path.
- **STOP-2** — do not touch the VSA surface: `BundleResult`, `Holons`, `Reckoner/new-discrete`,
  `wat/holon*.wat`, `wat/test.wat`'s `assert-coincident`, `wat/cache.wat`'s `hologram-svc`.
  All correct.
- **STOP-3** — do not touch `:wat::holon::to-wat` (`intrinsic/holon/atom.rs:647`) or the
  `Value::holon__HolonAST` coercion arm (`runtime.rs:7047`). Both legitimate.
- **STOP-4** — `reflect/verbs.rs`'s two conversions are out of scope; they are hidden residue behind
  an honest surface and are their own stone.
- **STOP-5** — no assertion in any existing test may weaken. Several `step_*` tests were rewritten
  hours ago as WatAST shape-matches; if one now needs to change again, it must prove the same thing.

## Verification

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::types)'
cargo nextest run --release -E 'binary_id(wat::value)'
cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'
cargo clippy --release --all-targets -- -D warnings
```

## What to report

The probe's corrupted output BEFORE and its correct output AFTER, verbatim, for rational, bigint and
the i64 control; what you did with `try_recognize_holon_value` and why; every caller of it you found;
the rustc error waterfall; the Summary line per scoped run; and anything that surprised you.
