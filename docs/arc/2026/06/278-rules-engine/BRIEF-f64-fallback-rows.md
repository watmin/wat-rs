# BRIEF — f64 arithmetic gets its fallback rows; ±Inf and NaN are the undefined point

**Ruled 2026-08-05 by the builder:** *"±Inf and NaN are undefined - mint the fallback rows."*

Anchor at `/home/watmin/work/holon/wat-rs/`; verify with `pwd`; `git -C …` for git reads.
Tree clean at HEAD `60246b53`. Floor **`4350 / 4350 / 0 / 262`**, clippy clean,
`check-where-shapes.sh` → `9 pair(s), 98 rows`.

## The work in one paragraph

`60246b53` gave f64 its `Alias` half — a rule can now *compare* two floats. It still cannot
*combine* them: there is no `:wat::rete::f64::{+ - * /}`. Those are `OpClass::Fallback` rows, and the
builder has now ruled what their undefined point is — **any ±Inf or NaN result**. This mints the four
rows. It is not a transcription of the i64 rows, and the reason is the whole brief.

## ★★ THE LOAD-BEARING FINDING — the i64 mechanism CANNOT work for f64

`dispatch_rete_op`'s `OpClass::Fallback` arm (`runtime.rs:8251`) substitutes the caller's fallback by
**catching an `Err`**:

```rust
match dispatch_keyword_head_value(op.core_name, &args[0..2], list_span, env, sym) {
    Ok(v) => Ok(v),
    Err(EvalBreak::Diagnostic(e))
        if matches!(e.kind(),
            RuntimeErrorKind::IntegerOverflow { .. } | RuntimeErrorKind::DivisionByZero) =>
    { eval_inner(&args[3], env, sym).map(|tv| tv.value_owned()) }
    Err(e) => Err(e),
}
```

**f64 arithmetic never returns `Err` on these inputs.** Proven by run, 2026-08-05:

```
(:wat::core::f64::* 1e200 1e200)  ->  #wat-edn.float/inf   exit 0
(:wat::core::f64::/ 1.0 0.0)      ->  #wat-edn.float/inf   exit 0
(:wat::core::f64::/ 0.0 0.0)      ->  #wat-edn.float/nan   exit 0
(:wat::core::i64::/ 1 0)          ->  RAISES DivisionByZero      ← the contrast
```

`eval_f64_arith` dispatches to a bare `a * b` — raw IEEE 754, no guard (this is exactly what
`purity.rs`'s own comment says, and it is why `f64::*` is not `total` in core).

**So four rows copied from the i64 shape would carry a fallback that can never fire.** They would
type-check, read correctly, pass a floor that never exercises the edge, and be silently vacuous —
`[[feedback_a_green_test_can_prove_nothing]]` waiting to happen. The arm must gain a second path that
inspects the **`Ok` value**.

## Part A — extend the `Fallback` arm to face a non-raising domain failure

In `runtime.rs`'s `OpClass::Fallback` arm, the `Ok(v)` branch must no longer be unconditional. When
the row's `ret` is `F64`, an `Ok` holding a NaN or an infinity **is** the undefined point and must
take the fallback.

Requirements:

- **Decide the F64 case from the ROW, not by sniffing the value's type.** `op.ret` /
  `op.params` already say what family the row is; a value-sniff would silently change behaviour for
  any future row that happens to return a float. Keep it a property of the declared row.
- **Only NaN and ±Inf.** `-0.0`, subnormals, and every ordinary finite float pass straight through.
  Rust's `f64::is_finite()` is exactly the predicate — `!is_finite()` is true for NaN, `+Inf`, `-Inf`
  and nothing else. Prefer it over hand-rolling three comparisons.
- **The `Err` path stays exactly as it is.** `IntegerOverflow`/`DivisionByZero` still take the
  fallback for the i64 family; a type error or arity error still propagates. Do not widen the `Err`
  match into a catch-all.
- **Comment it in the arm's existing voice**, which is careful about exhaustiveness: state that the
  i64 family fails by RAISING and the f64 family fails by RETURNING, that these are the two ways this
  op family can reach its undefined point, and that both are now faced.

## Part B — the four rows

In `src/rete/vocabulary.rs`, mirroring the i64 fallback quartet exactly in shape:

| rete_name | core_name |
|---|---|
| `:wat::rete::f64::+` | `:wat::core::f64::+` |
| `:wat::rete::f64::-` | `:wat::core::f64::-` |
| `:wat::rete::f64::*` | `:wat::core::f64::*` |
| `:wat::rete::f64::/` | `:wat::core::f64::/` |

All: `class: OpClass::Fallback`, `type_params: &[]`,
`params: &[ParamType::F64, ParamType::F64, ParamType::Keyword, ParamType::F64]`,
`ret: ParamType::F64`, `meta: OpMeta { pure: true, deterministic: true, total: true }`.

`total: true` is **earned by construction, not asserted** — same warrant the i64 rows carry: the
caller's `:undefined` value covers the undefined point, and after Part A the arm faces every way the
f64 family reaches it. That warrant is FALSE until Part A lands, so **Part A must land with Part B,
never after it.**

The call shape is unchanged from i64 — the literal keyword `:undefined` is mandatory in slot 3:

```clojure
(:wat::rete::f64::/ hits total :undefined 0.0)
```

## ⛔ STOPs — rejection criteria

- **⛔ STOP-1 — do NOT mint rows before the arm faces the value.** If you ship Part B without Part A,
  every row's `total: true` is a lie and the fallback is dead code. Land them together.
- **⛔ STOP-2 — do NOT change core.** `:wat::core::f64::{+ - * /}` keep returning raw IEEE values and
  keep their `total: false` classification in `purity.rs`. Core is honest as it stands; the *rete*
  row is what becomes total, by carrying a fallback. Widening core's behaviour would break every
  existing caller that relies on IEEE semantics.
- **⛔ STOP-3 — only `+ - * /` in this stone.** `f64::to-i64` is also partial on NaN/±Inf and is a
  genuine fallback candidate; `abs`/`min`/`max`/`round`/`clamp` need a per-op totality audit. Both
  are named in `NOTE-f64-arithmetic-has-no-rete-surface.md` and are NOT this stone. Report them, do
  not mint them.
- **⛔ STOP-4 — do not treat an ±Inf/NaN *input* as the trigger.** The ruling is about the RESULT. A
  caller holding an infinity and adding 1.0 gets an infinity, which falls back — that is correct and
  is how undefined-ness propagates to the caller's chosen value. Do not add input guards.
- **⛔** Do not add a `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.

## Verify — FOREGROUND, block, run the suite SOLO

```
cargo build --release
cargo nextest run --release          # no other cargo process alive
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line** — never a piped exit code.

## EXPECTATIONS — written before the strike

| # | what | expected |
|---|---|---|
| 1 | row count | **53** (49 + 4) |
| 2 | ★ ordinary arithmetic is untouched | `(:wat::rete::f64::+ 1.5 2.0 :undefined 0.0)` → `3.5` — the fallback is NOT taken |
| 3 | ★★ **the fallback FIRES on NaN** | `(:wat::rete::f64::/ 0.0 0.0 :undefined -1.0)` → `-1.0`, **not** `nan`. THE load-bearing row: this is what would silently fail if Part A were skipped |
| 4 | ★★ **the fallback FIRES on +Inf** | `(:wat::rete::f64::/ 1.0 0.0 :undefined -1.0)` → `-1.0` |
| 5 | ★★ **the fallback FIRES on overflow-to-Inf** | `(:wat::rete::f64::* 1e200 1e200 :undefined -1.0)` → `-1.0` |
| 6 | ★ **NON-VACUITY of the fallback itself** | with the SAME expression and a *different* fallback (`:undefined 42.0`) the answer must be `42.0`. A row that returned a constant would pass rows 3–5 and fail this |
| 7 | ★ `-0.0` is finite and passes through | `(:wat::rete::f64::* 0.0 -1.0 :undefined 99.0)` → `-0.0`, **not** `99.0` |
| 8 | ★ the marker is enforced | omitting `:undefined`, or passing another keyword, is a located `MalformedForm` naming the required marker |
| 9 | ★ the i64 rows still fall back | `(:wat::rete::i64::/ 1 0 :undefined -1)` → `-1` — Part A must not regress the `Err` path |
| 10 | ★ a type error still PROPAGATES | `(:wat::rete::f64::+ 1.0 1 :undefined 0.0)` is a check error, exit 1 — the fallback covers the domain hole, never a caller bug |
| 11 | ★ floor | `4350 / 4350 / 0 / 262` or higher if you add tests; **nothing lost** |
| 12 | ★ gate | `9 pair(s), 98 rows — wat == Clara on every shape` |
| 13 | clippy | clean |
| 14 | core unchanged | `(:wat::core::f64::/ 0.0 0.0)` still returns `nan`, exit 0 |

Rows 3, 4, 5, 6, 7, 9, 10, 11, 12 re-run by the orchestrator by hand.

**Runtime prediction: 45–70 minutes.** Larger than the last stone — Part A is real runtime logic, not
a table entry. Time-box 140.

**Trap doors:**

1. **Copying the i64 rows and stopping.** The fallback would never fire and every check would still
   be green. Rows 3–5 exist to catch exactly this; run them before you believe the floor.
2. **Sniffing the runtime value's type instead of reading the row.** Works today, changes behaviour
   for a future non-arithmetic float row. Decide from `op`.
3. **Reaching for `is_nan()` alone.** That misses ±Inf, which is half the ruling. `!is_finite()` is
   the whole predicate.
4. **"Fixing" core to raise instead.** STOP-2. Core's IEEE behaviour is relied on and is honest; the
   rete row is where totality is bought.
5. **A vacuous probe.** Write a real program with a `:user::main` in `wat-scripts/scratch-pad/`
   (never a tmp dir — that directory is gated on every build, which is the point). A probe with no
   `main` fails before resolving anything and proves nothing.
6. **Row 6 skipped.** Rows 3–5 pass if the arm returns any constant. Only varying the fallback proves
   it returns *the caller's* value.

## Note on `.wat.bad`

If a negative fixture is needed (row 10), it belongs in `tests/` with a rust test asserting the
failure — never in `wat-scripts/`, and never as a `.wat.bad` with no test referencing it. A
`.wat.bad` is a contract with a test on the other end; both halves of that rule were broken by
separate hands today.
