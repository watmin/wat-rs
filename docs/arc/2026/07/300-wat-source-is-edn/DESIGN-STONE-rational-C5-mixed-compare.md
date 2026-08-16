# DESIGN — Stone C5: mixed-numeric comparison passes the checker (consistency with C4 + eval + clj)

**Thesis.** C4 adopted mixed-numeric *arithmetic* (`(+ 1 2.0) => 3.0`). But mixed-numeric *comparison and
equality* are inconsistent: **eval accepts them** (the `values_compare`/`values_equal` arms C1–C4 added —
`(< 1 2.0)` → `true`, `(= 1 1.0)` → `false`), while **the checker rejects them** (arc 237.8a deleted the
cross-numeric path in `infer_equality`). So a real program (`startup_from_file`) rejects `(< 1 2.0)` at
check even though eval would compute it. C5 makes the checker accept mixed-numeric `= not= < > <= >=` →
`bool`, matching eval and clj — the same reversal C4 made for arithmetic, applied to the comparison family.

## Grounded state (this session)

```
                      CHECK (startup_from_file)   EVAL (eval_in_frozen)   clj
(< 1 2.0)   i64↔f64        ERR (rejects)               true                true
(<= 1 2N)   i64↔bigint     ERR (rejects)               true                true
(= 1 1.0)   i64↔f64        ERR (rejects)               false               false   ← C4 fixed eval, not check
(> 3.0 1/2) f64↔rational   ERR (rejects)               true                true
```

The checker is the outlier: all mixed-numeric comparison/equality is well-formed in clj (→ `bool`), and
wat's eval already computes it; only the check-side gate (237.8a's `cross-numeric path DELETED`) still rejects.

## The pinned contract

- The checker accepts mixed-**numeric** comparison/equality (`= not= < > <= >=`) → `:wat::core::bool`.
  Numeric = `{i64, f64, bigint, rational}` (+ `u8` if it participates). Same-type + subtype + both-record
  compatibility (the existing paths) are unchanged; C5 ADDS "both operands numeric → compatible".
- **`=` stays category-aware at eval** (C4): `(= 1 1.0)` → `false`, `(= 1 1N)` → `true`. C5 only makes the
  *check* accept it as well-formed; the eval result is unchanged.
- **Ordering `< > <= >=`** on mixed numerics → the numeric-value comparison (`(< 1 2.0)` → `true`).
- **Out of scope:** `=` on non-numeric heterogeneous types (`(= 1 "a")` — clj → false, wat still rejects);
  that's a broader `=`-semantics question, not the C4-surfaced numeric inconsistency. Flag, don't fold.

## Rooms

```clojure
{:check "src/check.rs — infer_equality (~12412), the `types_compatible` decision (~12456). Add a
         `both_numeric(a_resolved, b_resolved)` arm (both ∈ {i64,f64,bigint,rational}) → compatible.
         Undoes the 237.8a 'cross-numeric path DELETED' (comments at :12395, :12440) for the numeric case."
 :eval  "values_compare (runtime.rs ~8360) + values_equal (~8156) — VERIFY the full mixed-numeric matrix is
         present (i64↔f64 ✓, i64↔bigint ✓ C1, bigint↔f64 ✓ C4, rational↔f64 ✓ C2, rational↔i64 ✓ C2,
         rational↔bigint ✓ C1/C2). Add any missing pair the C5 probe surfaces (e.g. bigint↔rational compare)."
 :tests "flip the two 237.8a COMPARISON-reject tests to expect Ok:
         tests/types/probe_arc237_8a_no_implicit_coercion.rs::comparison_i64_f64_mixed_rejected_at_check
         tests/function/probe_arc237_8b_defclause_arithmetic.rs::regression_cross_type_lt_rejected
         (+ their cmp/lt `.wat.bad` fixtures — now valid, like the C4 arith flip)."}
```

## Out of scope

- No new type, no eval-semantics change (`=` category-awareness stays as C4 set it).
- Non-numeric `=` (`(= 1 "a")`) — separate `=`-semantics question, flagged not folded.
- Arithmetic (C4 done), i64-overflow (C3), bigint/rational (C1/C2) — untouched.

## STOP triggers

- STOP if a mixed-numeric comparison that CHECKS does not also EVAL to the right value — check and eval
  MUST agree (that consistency is the whole point).
- STOP if `(= 1 1.0)` eval-result changes from `false` (category-aware `=` is C4's contract, unchanged).
- STOP if loosening the checker accepts a NON-numeric mixed comparison (`(= 1 "a")`) — numerics only.
- STOP if same-type / subtype / record-compatibility comparison behavior changes (only ADD the numeric arm).

## RED spec

`tests/value/probe_rational_C5_mixed_compare.rs` + a co-located fixture: `startup_from_file` a `.wat` doing
`(< 1 2.0)` / `(= 1 1.0)` / `(<= 1 2N)` → **Ok** (RED at HEAD: Err); and eval those → `true`/`false`/`true`.

---

## ⚠ AMENDMENT 2026-08-15 — "the numeric-value comparison" was not true as implemented

**This stone is not superseded and its thesis is not in question.** Accepting mixed-numeric comparison at
the checker was the right reversal, and it correctly superseded arc 237.8a. The amendment is narrower: one
line of the pinned contract described something the implementation did not do.

The contract above pins:

> *"**Ordering `< > <= >=`** on mixed numerics → **the numeric-value comparison** (`(< 1 2.0)` → `true`)."*

From the day this stone landed until stone **C5b**, the implementation performed a **coerce-to-`f64`**
comparison instead — which agrees with a numeric-value comparison only below 2⁵³. Above it:

```clojure
(:wat::core::< 9007199254740992.0 9007199254740993)   ⇒ false      ; TRUE is correct
```

2⁵³+1 is not f64-representable and rounds to 2⁵³, so the operands compared equal. The reverse direction
returned `false` too — correct **by accident**, not by a second correct answer.

`DESIGN-STONE-C5b-exact-mixed-numeric-order.md` makes the implementation match this contract, by promoting
to the narrowest **exact** common representation instead of down to `f64`. **The wording above stands as
written; C5b is what made it true.**

### The divergence this creates from Clojure — deliberate, and stated here so it is never a surprise

This stone justified itself two ways: *"matching eval and clj"* **and** *"the numeric-value comparison."*
**Those two part company at exactly 2⁵³**, and this stone did not say which one wins. C5b settles it:
**the numeric-value comparison wins.** Above 2⁵³, wat's mixed-numeric ordering is expected to give the
mathematically correct answer even where Clojure's would not.

⚠ **The claim that Clojure returns `false` here is UNVERIFIED.** It was recorded from reasoning about
Clojure's `Numbers.lt` double/long promotion, not from a run — there is no JVM in this loop by standing
direction (*"i do not wish to have the jvm requirement in our CI tooling — so this remains local one
offs"*). Anyone who runs it should record the result here. If Clojure turns out to agree with wat, the
divergence disappears and only this paragraph needs deleting.

### Two things this stone's own scope note got right, and one the follow-up NOTE got wrong

- Right: `=` **stays category-aware** and is therefore structurally immune to the precision defect — it
  never coerces, so it never rounds. `=`/`not=` were never affected.
- The follow-up NOTE said *"the family is six ops"*. It is **four** — `< > <= >=`. See C5b.
