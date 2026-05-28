# Stone 237.8b — Recipe-lock + numeric grid (arithmetic + ordering for i64, f64 via wat-`defclause`)

**The recipe-stone.** Establishes the durable pattern: **per-Type binary
primitives are 2-ary Rust intrinsics; the polymorphic surface is a wat
`defclause` whose clauses dispatch by arity × arg-Type; per-op identity
defaults via Lisp tradition.** Once locked + inscribed (`feedback_per_type_binary_primitives`),
future per-Type ops (`%`, comparison family extensions, future numeric types
like `u8`/`u32`) follow the recipe with zero re-thinking.

**Proves the recipe across two op-families × two types** (arithmetic +
ordering for i64, f64) — the smallest demonstration that the doctrine is
reusable, not just one-shot.

## STATUS (post-probe, 2026-05-27 night)

**FM-2-bis probe `tests/probe_arc237_8b_defclause_arithmetic.rs` ran at HEAD `3e3acbbb`+.** Empirical findings:

| Gate | State | Notes |
|---|---|---|
| 1 (defclause `&` rest-binder support) | **RED** — defclause's argspec parser rejects `&`: *"defclause arg-vector triple at position 1 must be `name <- :T`; got symbol where `<-` was expected"* | **8b BLOCKED until precursor ships.** Stone **237.8b-prep** required: mint defclause `&` rest-binder support (parser extension + clause-matching + binding-to-Vector<T> at eval). The recipe's 3+-ary fold clauses need this. |
| 2 (defclause first-match by arg-`<-`-Type) | GREEN | defclause CAN dispatch on arg-Type with `<-` annotation; no `:guard` required. |
| 2-cross (cross-type → :NoMatchingClause) | GREEN | Mixed (i64, f64) args yield `:NoMatchingClause` correctly. **THE DECISION enforced via clause-absence, no special-case logic.** |
| 3 (0-ary clause body literal `0` infers as `:i64`) | GREEN | `([] -> :wat::core::i64 0)` works; Lisp identity defaults trivial. |
| 4a (i64 ordering primitives correctness) | GREEN | Existing 237.3 aliases (`:i64::<`, `:i64::>`, `:i64::>=`) work correctly. |
| 4b (f64 NaN ordering) | IGNORED (mint-confirmer) | Awaiting `:wat::core::f64::<` mint. |

**Strategy revision**: 8b becomes a TWO-STONE block:
- **237.8b-prep** — mint defclause `&` rest-binder support. Substrate stone. Small surface (parser + check-time + eval-time additions; ~30-50 lines). FM-2-bis probe = existing `probe_arc237_8b_defclause_arithmetic.rs` Gate 1 (un-ignore after extension lands).
- **237.8b** — the recipe-lock + numeric grid (this stone), shipped AFTER 8b-prep makes Gate 1 green.

Per the spawn-block winding discipline: parent stone (8b) cannot close until precursor (8b-prep) closes. 8b-prep ships first; 8b uses the just-shipped capability.

## Locked decisions (per dialogue 2026-05-27 night)

1. **Per-Type primitives are STRICTLY BINARY** — drop `'2` suffix. `:wat::core::i64::+'2` → `:wat::core::i64::+`.
2. **`::` separator stays** — `/` is reserved for instance-methods (`:Vector/empty?`, `:Record/assoc`); `::` is for namespace-functions (binary ops in the type's namespace; not method-on-instance dispatch). `//` collision on division seals it.
3. **Polymorphic surface in wat via `defclause`** — bare `:wat::core::+`/`-`/`*`/`/`/`<`/`>`/`<=`/`>=` migrate to wat-defclauses dispatching to per-Type primitives.
4. **Lisp arity rules preserved per Clojure/CL tradition**:
   - `+`/`*` 0-ary: i64 identity (0 / 1); 1-ary: arg unchanged; 2-ary: binary; 3+-ary: fold
   - `-`/`/` 0-ary: ERROR (no clause → `:NoMatchingClause`); 1-ary: negate/reciprocal per-Type identity-on-left; 2-ary: binary; 3+-ary: fold
   - `<`/`>`/`<=`/`>=`: 2-ary only (no variadic for ordering in this stone; future extension if demand surfaces)
5. **HARD CUT** of `infer_arithmetic` + `eval_arithmetic_variadic` + `is_numeric` (Rust handlers); HARD CUT of `infer_comparison`'s ordering arms (leaving only `=`/`not=` routes for 8c); HARD CUT of per-Type variadic wat fns at `wat/core.wat:104-132`; no shims, no aliases.
6. **`!=` → `not=` reconcile** — the existing `:wat::core::i64::!=` (Stone 237.3 alias) renames to `:wat::core::i64::not=`. The surface polymorphic is `:wat::core::not=`; per-Type aligns. One canonical path.

## Crawl (ground truth, HEAD `3e3acbbb`)

### Existing per-Type primitives + their state

| Op family | i64 state | f64 state |
|---|---|---|
| Arithmetic `+`/`-`/`*`/`/` | shipped as `:i64::+'2` etc. (check.rs:16281; runtime.rs binary handlers) — 4 primitives, suffix `'2` | shipped as `:f64::+'2` etc. (check.rs:16300+) — 4 primitives, suffix `'2` |
| Equality `=` / `not=` | shipped as `:i64::=` + `:i64::!=` (check.rs:16479+; routes to `eval_eq`/`eval_not_eq`) — Stone 237.3 aliases | **MISSING** — no `:f64::=` or `:f64::not=` |
| Ordering `<` / `>` / `<=` / `>=` | shipped: `:i64::>` `<` `>=` (3 primitives, routes to `eval_compare`); **`:i64::<=` MISSING** | **MISSING** — entire family unminted |

So 8b's mint surface:
- 1 mint: `:wat::core::i64::<=`
- 6 mints: `:wat::core::f64::=`, `:f64::not=`, `:f64::<`, `:f64::>`, `:f64::<=`, `:f64::>=`
- 1 rename: `:wat::core::i64::!=` → `:wat::core::i64::not=` (HARD CUT old name)
- 8 renames: `:wat::core::{i64,f64}::{+,-,*,/}'2` → drop `'2`
- = 16 substrate changes

### wat-side variadic fns to DELETE (absorbed by defclauses)

`wat/core.wat:104-132` defines 8 variadic per-Type fns:
- `(:wat::core::i64::+ & (xs :Vector<i64>) -> :i64)` — 0-ary returns 0 fold seed
- `(:wat::core::i64::* & (xs :Vector<i64>) -> :i64)` — 0-ary returns 1 fold seed
- `(:wat::core::i64::- (first :i64) & (xs :Vector<i64>) -> :i64)` — 1-arity-min; 1-arity = negate (0 - first); 2+ = fold
- `(:wat::core::i64::/ (first :i64) & (xs :Vector<i64>) -> :i64)` — 1-arity-min; 1-arity = reciprocal (1 / first); 2+ = fold
- Same shape × 4 for f64

These become defclause clauses on the bare `:wat::core::+`/etc. (variadic clauses fold over per-Type rest-binders). After 8b: the per-Type variadic fns DELETE; the bare ops carry the variadic behavior in defclause clauses.

**Crucial reuse**: the body of the variadic fold clause IS the body of the existing variadic wat fn. So the defclause's 3+-ary clause body is a near-direct port of the wat/core.wat:104-132 fn bodies.

### Consumer-sweep audit (rename + deletion cascade)

`:i64::+'2` etc. (binary; rename target) — ~16 sites in wat/, wat-tests/, tests/, examples/, crates/. Spot-checks:
- `wat/stream.wat:662` — `(:wat::core::i64::-'2 remaining 1)` — internal substrate caller
- `wat/Record.wat:103` — `(:wat::core::i64::/'2 n 3)` — internal substrate caller
- `tests/probe_arc237_stone3_guard_ensure.rs` — defclause probe (already uses bare `:i64::+` in clause bodies; some `:i64::*'2` usages)
- `wat-tests/test.wat`, `wat-tests/counter-actor-proof-*.wat` — test usages

`:i64::+` etc. (variadic; deletion target) — ~10 sites; most look like binary calls (`x y`) which work post-rename; few uses of variadic-form-with-3+-args need migrating to the polymorphic `(:wat::core::+ ...)` defclause surface.

Cascade prediction: **small-but-nonzero**. Most call sites are already binary-shaped; the rename is mechanical; the per-Type-variadic-to-polymorphic-variadic switch is local. Probably <20 sites total. Substrate-as-teacher cascade will surface and migrate them.

### Handlers to DELETE / REDUCE

- DELETE `infer_arithmetic` (src/check.rs:13211) — replaced by defclause check-time dispatch via clause-matching
- DELETE `eval_arithmetic_variadic` (src/runtime.rs:9910) — replaced by defclause runtime dispatch
- DELETE `is_numeric` (src/check.rs:13293) — sole remaining caller after 8a was `infer_comparison`'s cross-numeric path (deleted in 8a) and `infer_arithmetic` (deleted in 8b); after 8b, no callers; HARD CUT
- DELETE ordering arms of `infer_comparison`'s dispatch (check.rs:6478-6492) — `<`/`>`/`<=`/`>=` route to defclause instead; `infer_comparison` reduces to handling `=`/`not=` only (those migrate in 8c)
- DELETE per-Type variadic wat fns at `wat/core.wat:104-132` — absorbed by defclause clauses

## The recipe locked

```wat
;; THE RECIPE — per-Type binary primitive + wat-defclause polymorphic surface
;;
;;   Layer 1 (Rust): per-Type binary primitive  — :wat::core::<Type>::<op>
;;                   ALWAYS 2-ary; irreducible; one fn per Type per op.
;;
;;   Layer 2 (wat):  polymorphic defclause      — :wat::core::<op>
;;                   Clauses dispatch by arity (0/1/2/3+) × arg-Type.
;;                   Per-op identity defaults via Lisp tradition.
;;                   Variadic via 3+-ary clause with & rest-binder folding
;;                   the per-Type binary primitive over rest.
```

Example for `+` (the canonical recipe-instance):

```
(:wat::core::defclause :wat::core::+
  ;; 0-ary identity: i64 0 (Lisp tradition)
  ([] -> :wat::core::i64 0)
  ;; 1-ary: per-Type arg unchanged
  ([x <- :wat::core::i64] -> :wat::core::i64 x)
  ([x <- :wat::core::f64] -> :wat::core::f64 x)
  ;; 2-ary: direct per-Type binary call
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::+ x y))
  ;; 3+-ary: per-Type fold over rest
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl rest (:wat::core::i64::+ x y)
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc n))))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl rest (:wat::core::f64::+ x y)
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::+ acc n)))))
```

For `-`/`/`: drop the 0-ary clause (no `:NoMatchingClause` on 0-ary args triggers via 237.4 rich error); 1-ary clauses do per-Type identity-on-left:

```
(:wat::core::defclause :wat::core::-
  ;; NO 0-ary clause — :NoMatchingClause fires per 237.4
  ;; 1-ary per-Type: negate (identity-on-left = 0)
  ([x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::- 0 x))
  ([x <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::- 0.0 x))
  ;; 2-ary, 3+-ary: same shape as + ...
  )
```

For `*`: 0-ary returns 1 (multiplicative identity); else same as +.
For `/`: 0-ary error; 1-ary is reciprocal (identity-on-left = 1).
For `<`/`>`/`<=`/`>=`: 2-ary only:

```
(:wat::core::defclause :wat::core::<
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::< x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::bool (:wat::core::f64::< x y)))
```

Cross-type `(:wat::core::+ 1 2.0)` → `:NoMatchingClause` (no clause matches mixed i64/f64). THE DECISION enforced by ABSENCE of a clause, not by special-case Rust check. Cleaner than 8a.

## FM-2-bis probe gates (must settle before BRIEF)

`tests/probe_arc237_8b_defclause_arithmetic.rs` — 4 load-bearing gates + regression contract.

### Gate 1: defclause supports `&` rest-binders in args-vec

The 3+-ary fold clauses need `[x <- :T y <- :T & rest <- :Vector<T>]`. The defclause spec at INTERSTITIAL line 9742+ said `Args: [name <- :Type name <- :Type]` — didn't explicitly mention rest-binders. **Empirical probe: write a minimal defclause with `& rest <- :Vector<i64>` and dispatch a variadic call to it.** If unsupported, defclause needs extension first (a sub-stone) before 8b can ship.

### Gate 2: defclause first-match dispatches by arg-Type with `<-` annotations

The recipe needs `(+ 1 2)` → i64 clause; `(+ 1.0 2.0)` → f64 clause; `(+ 1 2.0)` → `:NoMatchingClause` (cross-type rejected). The existing probe (`probe_arc237_stone3_guard_ensure.rs`) only exercises `:guard`-based dispatch, not pure `<-`-Type-annotation dispatch. **Empirical probe: 2 clauses differing only in arg-`<-`-Type; verify correct dispatch + cross-type :NoMatchingClause.**

### Gate 3: 0-ary clause body literal type inference

`([] -> :wat::core::i64 0)` — does the literal `0` type-check as `:wat::core::i64`? Default numeric literal typing should make this work. If it fails (e.g., needs explicit type ascription), the 0-ary identity clauses need different syntax.

### Gate 4: per-Type ordering aliases promotable to true per-Type without breaking existing callers

The 237.3 i64 ordering aliases route through polymorphic `eval_compare`. Need to verify: when we add f64 ordering primitives + the missing i64::<=, the polymorphic eval_compare correctly dispatches per-Type (especially f64 NaN — `1.0 < f64::NAN` should return false, not Some(false)→true). **Empirical probe: exercise i64 ordering + f64 ordering + NaN edge case via the per-Type primitives (after mint).**

If any gate red → reshape; possibly defer 8b until defclause extension lands.

## Slicing

**One stone** (237.8b). The pieces are coupled:
- Renames + mints are inseparable from defclause migration (the defclauses reference the renamed/minted primitives)
- DELETE of Rust handlers is inseparable from defclause working (without the handlers, the bare ops would have no inference; the defclauses are the replacement)
- Consumer cascade is unbounded but spot-checked small

Predicted Mode-A: **60–90 min**. Wakeup hang-detector: 60min (harness clamp).

## What this stone does NOT do

- Equality migration (`=`/`not=` per-Type defclause) — 237.8c (full primitive equality grid + composite recursive equality).
- `infer_comparison`'s `=`/`not=` arms — stay routed there until 8c migrates them.
- DispatchRegistry HARD CUT — 237.8d (mechanical; 0-tenant after 8b finishes evacuating).
- INSCRIPTION + memory mint — 237.9 (folds arc 146 + arc 148 + arc 237 + the recipe doctrine).
- Comparison primitives for non-numeric types (String/Char/time `<` etc.) — future stone after 9 ships.
- `%` (modulo) + bit ops — future stones; the recipe locked here applies.
- New numeric types (u8/u32/i32/u64/etc.) — future stones; the recipe locked here applies.

## Constraints

- Edits in `src/check.rs` + `src/runtime.rs` + `src/lexer.rs` (if mixed-type entries need adjustment for renames) + `wat/core.wat` + probe + consumer-cascade sites.
- NO holon-rs. NO touch of `infer_polymorphic_holon_pair_*` / `infer_polymorphic_time_arith` (out of scope, different polymorphism).
- NO `DispatchRegistry` deletion (237.8d's job).
- NO equality migration (237.8c's job).
- Green-gate: `cargo test --release --lib -p wat` + `cargo build --release --tests --workspace`. RAW commands, no scripts.
- HARD CUT discipline — DELETE `'2` suffix; DELETE per-Type variadic wat fns; DELETE Rust handlers; DELETE `infer_comparison`'s ordering arms; DELETE `:i64::!=` (rename to `:i64::not=`); no shims.
- The substrate-as-teacher cascade is acknowledged scope — sonnet iterates failures to migrate any cross-type or variadic-per-Type call sites.
