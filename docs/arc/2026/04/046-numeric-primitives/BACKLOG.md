# wat-rs arc 046 — numeric primitives uplift — BACKLOG

**Shape:** three slices. One implementation, one docs sweep, one
INSCRIPTION + cross-refs. No substrate gaps; pure addition.

---

## Slice 1 — runtime + check + tests

**Status: ready.**

`src/runtime.rs`:
- Add 5 dispatch entries:
  - `:wat::core::f64::max` → `eval_f64_arith(... |a, b| Ok(a.max(b)))`
  - `:wat::core::f64::min` → `eval_f64_arith(... |a, b| Ok(a.min(b)))`
  - `:wat::core::f64::abs` → new `eval_f64_unary(args, env, sym, ":wat::core::f64::abs", f64::abs)`
  - `:wat::core::f64::clamp` → new `eval_f64_clamp(args, env, sym)` (3-arg)
  - `:wat::std::math::exp` → `eval_math_unary(args, env, sym, "exp", f64::exp)`
- Add helper `eval_f64_unary` (mirrors `eval_math_unary` but takes
  full op-name string, since the namespace differs).
- Add helper `eval_f64_clamp` (3-arg, value + lo + hi all f64).
- Inline tests in the module-level `mod tests` block:
  - `f64::max` / `f64::min` happy paths + equal values
  - `f64::abs` positive / negative / zero
  - `f64::clamp` in / below / above / `lo == hi` edge
  - `math::exp` positive / negative / zero / large
  - Type-error tests for at least one new op (mirror the existing
    `f64::round` rejection pattern)
  - Arity-error tests for at least one new op (rough parity)

`src/check.rs`:
- Register 5 new type schemes:
  - `f64::max`, `f64::min` → `f64 × f64 -> f64`
  - `f64::abs` → `f64 -> f64`
  - `f64::clamp` → `f64 × f64 × f64 -> f64`
  - `math::exp` → join the existing `["ln", "log", "sin", "cos"]`
    loop → `["ln", "log", "sin", "cos", "exp"]`

**Sub-fogs:**
- **1a — Type-coercion in `eval_math_unary`.** It auto-promotes
  `i64 -> f64`. The new `eval_f64_unary` should match (or stay
  strict like `eval_f64_arith`?). Decision: **match `eval_math_unary`
  precedent** — math primitives accept `i64` for ergonomics
  (`(:wat::std::math::ln 1)` shouldn't fail on type). f64::abs
  follows suit. f64::clamp matches `eval_f64_arith`'s strictness
  (all-f64) since it's ":wat::core::f64::*", same family as
  arithmetic.

  Hmm — inconsistent? Let me reconsider. `f64::round` is strict
  (no i64 promotion per `eval_f64_round`). Its peer ops
  (`f64::+/-/*//`) are also strict. The `:wat::core::f64::*`
  family is consistently strict.

  `:wat::std::math::*` allows i64 promotion per `eval_math_unary`.

  **Decision: keep namespace consistency.** `f64::abs` and
  `f64::clamp` strict (no i64 promotion). `math::exp` allows
  promotion (matches its math siblings).

## Slice 2 — docs sync

**Status: ready** (independent of slice 1; can run in parallel).

`docs/USER-GUIDE.md`:
- §3 mental-model overview line 397 — extend the language-core
  primitive listing to include `f64::max/min/abs/clamp`.
- §15 Forms appendix — add 5 new rows for the new primitives.
  Plus drift catch: 6 existing math primitives (`ln`, `log`,
  `sin`, `cos`, `pi`, `exp`) were never in the appendix. Fix
  the gap alongside the new additions — same edit, same arc.

`docs/CONVENTIONS.md`: no edits needed (table descriptions are
summary-level, naturally accommodate new entries).

`README.md`: no edits needed (no per-form enumeration).

**Sub-fogs:**
- **2a — Forms appendix row format.** Match the existing
  `:wat::core::f64::+/-/*//` row's three-column shape: form,
  args, type/notes.

## Slice 3 — INSCRIPTION + cross-refs

**Status: obvious in shape** (once slices 1 – 2 land).

- `docs/arc/2026/04/046-numeric-primitives/INSCRIPTION.md`.
  Records: 5 primitives shipped, the lab-vs-substrate framing
  question that drove the arc, the doc drift catch (math
  primitives never in appendix), the namespace-consistency
  decision (strict for f64::*, i64-promoting for math::*).
- `docs/proposals/<...>/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  in **lab repo** — row documenting wat-rs arc 046.
  (058 CHANGELOG lives in lab repo per arc 011's precedent.)
- Lab arc 015 (in_progress in the lab repo) resumes after this
  arc ships — uses substrate primitives directly, drops the
  `f64-max` lab helper, possibly drops `clamp` lab helper.

---

## Working notes

- Opened 2026-04-24, the morning the known-good marker was set.
  Arc 046 surfaces the "drift surfaces on demand" pattern — the
  caller (lab arc 015) hit something missing, the substrate adds
  it, the caller resumes.
- Five primitives in one arc is more than the typical
  one-thing-per-arc rhythm, but the cluster is cohesive (all
  numeric ops, all the same plumbing pattern). Splitting would
  surface five "trivial mirror of existing pattern" arcs.
- The framing question (substrate vs userland) is the durable
  worth recording, not the implementation itself.
