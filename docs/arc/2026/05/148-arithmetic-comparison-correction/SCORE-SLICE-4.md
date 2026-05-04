# Arc 148 Slice 4 — SCORE

**Sweep:** sonnet, agent `a28c76c87c3522d66`
**Wall clock:** ~18 min (1080s) — **WAY UNDER** the 60-90 min Mode A
predicted band; used 15% of the 120-min time-box.
**Output verified:** orchestrator independently re-ran all 9 baselines
+ worked example + spot-checked Path C wiring + ran full workspace
`cargo test`.

**Verdict:** **MODE A CLEAN SHIP.** 10/10 hard rows pass; 4/4 soft
rows pass. Path C chosen (recommended). 3 honest deltas surfaced;
all are within-scope substrate observations the BRIEF didn't fully
anticipate.

## The boss fight, second attempt

First attempt (pre-arc-150) discovered the missing variadic-define
equipment + STOPPED before shipping the wrong shape. Arc 150 closed
the gap in ~24 min. Sonnet's second attempt at slice 4 with the new
gear: **~18 min, clean ship**.

The substrate-as-teacher cascade compounds — slice 4's failed
assumption surfaced arc 150's foundation gap; arc 150 closed it;
slice 4 shipped against the now-honest substrate without bridging.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EDITS to `src/runtime.rs` (+536 LOC) + `src/check.rs` (+169 LOC) + `wat/core.wat` (+129 LOC) + `tests/wat_polymorphic_arithmetic.rs` (+219 LOC). NO new files in `src/`. NO new test files. Net +813 LOC (within the 600-1500 LOC band). |
| 2 | 8 mixed-type Rust primitives | ✅ All 8 registered: `:wat::core::{+,-,*,/}` × `{i64-f64, f64-i64}`. Each has dispatch arm + TypeScheme + freeze-pipeline entry + dispatch_substrate_impl helper. |
| 3 | 4 binary Dispatch entities | ✅ `:wat::core::+,2`, `:-,2`, `:*,2`, `:/,2` declared at `wat/core.wat:90,96,102,108`. Each has 4 arms covering (i64,i64) / (f64,f64) / (i64,f64) / (f64,i64). |
| 4 | 8 same-type variadic wat fns | ✅ `:wat::core::{i64,f64}::{+,-,*,/}` declared in `wat/core.wat`. Use arc 150's variadic define syntax. Lisp/Clojure arity rules: `+`/`*` 0-ary returns identity; `-`/`/` use `(first :T) & (xs :Vector<T>)` pattern enforcing 1+ args (Delta 3). |
| 5 | Polymorphic variadic surface (Path A/B/C) | ✅ **Path C chosen.** `infer_polymorphic_arith` renamed to `infer_arithmetic` + extended for variadic arity (Lisp/Clojure rules). Function at `src/check.rs:6753`. Worked example `(:wat::core::+ 0 40.0 2) → :f64 42.0` passes via `eval_arithmetic_variadic` at `src/runtime.rs:5149`. |
| 6 | RETIRED: eval_poly_arith + dispatch arms + freeze entries | ⚠️ MIXED — see Delta 1. `eval_poly_arith` retired (replaced by `eval_arithmetic_variadic` + `apply_arith_pair` + `ArithOp`). Polymorphic dispatch arms now route to the new variadic eval. **Freeze-pipeline polymorphic entries KEPT** per Path C requirement (the polymorphic surface is still a substrate primitive needing redex registration). 8 NEW mixed-type leaves added to freeze pipeline. |
| 7 | infer_polymorphic_arith handling | ✅ Path C: renamed to `infer_arithmetic`; body extended for variadic arity (handles 0/1/2+ per Lisp/Clojure rules). Dispatch site at `src/check.rs:3326` updated. |
| 8 | All baseline tests still green | ✅ `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3; `wat_polymorphic_arithmetic` 33/33 (was 20; +13 new); `wat_arc148_ord_buildout` 46/46; `wat_arc150_variadic_define` 16/16; `wat_variadic_defmacro` 6/6. **146/146 across the substrate-foundation tests.** |
| 9 | Worked example from DESIGN passes | ✅ Test `slice4_variadic_add_mixed_numerics_design_worked_example` confirms `(:wat::core::+ 0 40.0 2) → :f64 42.0`. Mixed-numeric variadic via dispatch + per-pair routing works end-to-end. |
| 10 | Honest report | ✅ Sonnet's report covers all required sections. Path chosen + rationale named clearly; counts of new/retired entities explicit; 3 honest deltas surfaced with reasoning. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (600-1500) | ✅ +933 / -120 = net +813. Within band; honest scope. |
| 12 | Style consistency | ✅ Mixed-type Rust primitives mirror eval_poly_arith's per-pair impl style. Dispatch entity shape mirrors arc 146 length/get/etc. Variadic wat fns mirror arc 150's foldl-over-rest pattern. Path C rename mirrors slice 5's `infer_comparison` shape. |
| 13 | clippy clean | ✅ 33 lib + 41 all-targets warnings; verified by stash-and-recompare — no new warnings. |
| 14 | Audit-first discipline | ✅ Sonnet correctly chose Path C without burning time on Path A/B exploration. Path A's parametric `Vector<T>` ruled out cleanly (mixed args can't unify); Path B's untyped vector ruled out (substrate addition with unclear semantics). |

## The 3 honest deltas (sonnet)

### Delta 1 — Freeze pipeline polymorphic entries KEPT (BRIEF deviation)

The BRIEF's "retire 4 polymorphic freeze entries" instruction was
Path-A/B-conditional. Under Path C, the polymorphic surface
(`:wat::core::+` etc.) is still a substrate primitive — its
freeze-pipeline pure-redex entries remain valid and necessary;
removing them would break canonical-form firing for `(:wat::core::+ ...)`.

Sonnet kept the 4 polymorphic entries AND added the 8 NEW mixed-type
leaves to the freeze pipeline (substrate-internal addressing reachable
per arc 109 no-privacy doctrine).

**Architectural assessment:** correct judgment. The brief was
implicitly Path-A/B-biased; Path C requires keeping the substrate
primitive's freeze metadata. Honest within-scope adjustment.

### Delta 2 — `dispatch_substrate_impl` arith helpers (substrate work)

When a Dispatch entity routes to a leaf (e.g.,
`(:wat::core::+,2 1 2.0)` → routes to `:wat::core::+,i64-f64`),
`eval_dispatch_call` walks `apply_function` lookup → `dispatch_substrate_impl`.
Neither path goes through `dispatch_keyword_head`, so sonnet added
16 value-level helper entries (4 same-type i64 + 4 same-type f64 +
4 mixed i64-f64 + 4 mixed f64-i64) using 4 generic helper functions
that mirror the AST-level `eval_<T>_arith` shape.

Without these, direct calls to the binary Dispatch entity would
`UnknownFunction` on the leaf name.

**Architectural assessment:** honest substrate work the BRIEF didn't
anticipate. Same shape as arc 146 slice 2's Delta 1 (substrate-impl
fallback for Dispatch arms). Required for the architecture to hold
end-to-end.

### Delta 3 — Same-type variadic for `-`/`/` uses `(first :T) & (xs :Vector<T>)` pattern

Arc 150's variadic define enforces `args.len() >= fixed_arity`. To
express "1+ args required" (the `-`/`/` semantics: 0-ary errors;
1-ary inserts identity-on-left; 2+-ary folds), sonnet used:

```scheme
(:wat::core::define
  (:wat::core::i64::- (first :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
  (:wat::core::cond
    ((:wat::core::empty? xs) (:wat::core::i64::-,2 0 first))   ;; 1-ary: insert identity
    (:else (:wat::core::foldl xs first :wat::core::i64::-,2)))) ;; 2+-ary: fold
```

The 0-ary case (no `first`, no rest) trips arc 150's
`args.len() >= 1` check and surfaces as ArityMismatch automatically.

**Architectural assessment:** clean substrate observation. Arc 150's
variadic define is the right tool; the (one fixed param + rest)
shape elegantly expresses 1+-required. Worth carrying forward as
a pattern note for future variadic surfaces with similar arity
requirements.

## Calibration record

- **Predicted Mode A (~50%)**: ACTUAL Mode A clean. Path C chosen as
  recommended. Calibration matched.
- **Predicted runtime (60-90 min Mode A; up to 120 min cap)**:
  ACTUAL ~18 min. **WAY UNDER** band — used only 15% of the 120-min
  time-box. The boss fight was the easiest slice in arc 148 thanks
  to: (a) arc 150's variadic-define equipment, (b) slice 5's Path C
  precedent, (c) arc 146's Dispatch pattern, (d) slices 2/3's
  substrate foundations all already in place.
- **Time-box (120 min)**: NOT triggered.
- **Predicted LOC (600-1500)**: ACTUAL net +813 (within band).
- **Honest deltas (predicted 0-2; actual 3)**: Delta 1 was
  Path-C-conditional brief deviation; Delta 2 was substrate work the
  brief didn't anticipate; Delta 3 was a substrate observation about
  arc 150's variadic-define semantics. All within scope; all
  correctly handled.

## Workspace failure profile (pre/post slice)

- **Pre-slice baseline:** documented arc 130 HologramCacheService
  flakes (multi-threaded; 4-5 tests vary run-to-run) + 1
  pre-existing `call_stack_populates_on_assertion` panicking-test.
- **Post-slice (default cargo test):** 1832 passed / 5 failed —
  all 5 failures are the documented arc 130 + pre-existing
  panicking-test. **NO new failures introduced by this slice.**

## What this slice closes

- **The polymorphic-handler anti-pattern for arithmetic is RETIRED.**
  `infer_polymorphic_arith` → `infer_arithmetic` (rename + variadic
  extension). The "anti-pattern" framing was the issue; the function
  itself is honest custom inference.
- **Every arithmetic op is a first-class entity at runtime.** All
  4 polymorphic surfaces + 4 binary Dispatch entities + 8 same-type
  variadic wat fns + 8 same-type binary leaves + 8 mixed-type leaves
  are queryable via `signature-of` / `lookup-define` / `body-of`.
  Discoverable, addressable, reflectable.
- **The substrate's "extend the carrier" + "Dispatch + per-Type
  leaves" + "variadic wat fn over binary dispatch"** patterns are
  all exercised end-to-end in arc 148. The methodology IS the proof.
- **LLM affordance achieved** for arithmetic per the comma-typed-leaf
  rule:
  - Default reach: `(:wat::core::+ x y z)` — Lisp-natural
  - Type-locked: `(:wat::core::i64::+ x y z)` — same shape, type
    enforced at call site
  - Direct binary: `(:wat::core::+,2 x y)` — substrate addressing
  - Mixed-type direct: `(:wat::core::+,i64-f64 x y)` — rarely needed
    by humans/LLMs; substrate honesty per no-privacy doctrine

## What this slice unlocks

- **Slice 6** — closure paperwork (small)
- **Arc 146 slice 5** — closure (was BLOCKED on arc 148 completion)
- **Arc 144 closure** — verification + paperwork (becomes tractable)
- **Arc 109 v1 closure trajectory** — one major chain link closes

## Pivot signal analysis

NO PIVOT. The 3 deltas are within-scope adjustments; Path C was the
recommended path; sonnet executed cleanly.

The cascade keeps converging. Each substrate-as-teacher discovery
strengthens the foundation; each arc compounds; the LLM-facing
surface grows more honest with every slice. The methodology IS the
proof. The rhythm holds — and accelerates dramatically when the
prerequisites have been deliberately laid down.

**Time observation worth recording:** the slice that took multiple
sessions to design (the Path A/B/C debate; the variadic-define
discovery; the locked architecture) shipped in 18 minutes once
sonnet had the equipment. Arc 109's cumulative substrate work pays
dividends — each new slice rides on a thicker foundation.
