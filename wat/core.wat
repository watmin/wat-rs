;; vigilatum: 2026-06-06T04:56:04Z — UPDATED-vigilia spec/DSL 5-spell guard
;; L1+L2=0 (cernere [CONVERGED 0+0: full vocabulary table, every expand-time
;; head verified on the pure-total allow-list] + probare [all 12 forms (16 at first earn; 4 ordering defclauses retired by Stone 245.8)
;; Expressed] + conferre [all 17 header claims verified; 6 USER-GUIDE
;; divergences fixed spec-side] + exigere [CONVERGED 0+0] + circumspicere
;; LAST [the false loads-early rationale killed at both sites; empty-step
;; behaviors documented + witnessed at their empirical failure shapes]).
;; Witness corpus: deftest-green(core-arithmetic + core-equality +
;; core-threading + core-collection-aliases + option-expect + record-def +
;; result-expect + struct-to-form + seq-fold-aliases); corpus 236/0/53;
;; checker-clean. Canonical record:
;; docs/arc/2026/06/249-total-pure-macros/WARD-COREWAT-REEARN.md.
;; RE-EARNED 2026-06-06T04:56:04Z (diff-scoped, the 245 clear: Stone 245.8 retired
;; the ordering defclauses — ordering is a relational check-side intrinsic; the
;; retirement note's claims verified live [infer_ordering / dispatch arms /
;; leaves present]; corpus 236/0/53; zero defclause remnants).
;;
;; wat/core.wat — the :wat::core::* stdlib surface: short-name aliases plus the
;; polymorphic arithmetic defclauses.
;;
;; Ordering (`<`/`>`/`<=`/`>=`) is a relational check-side intrinsic (Stone 245.8),
;; the sibling of equality. The defclauses that formerly lived here are retired;
;; the per-Type leaves (`:wat::core::i64::<` etc.) remain as the type-locked tier.
;;
;; Position in the stdlib array is not load-bearing for visibility:
;; register_stdlib_defmacros (src/macros/parse.rs) walks the entire
;; concatenated stdlib in one pre-expansion pass, so every defmacro
;; (defn, concat, ->, …) is registered before any expansion runs.
;; defclause/defalias stubs are likewise pre-registered before use
;; (src/freeze.rs: preregister_stdlib_defclause_stub).

;; ─── Short-name collection aliases ──────────────────────────────────────────
;;
;; The polymorphic-name collection ops — length / empty? / contains? / get /
;; conj / assoc — are Rust ∀T intrinsics: check-side inference lives in
;; src/collection/infer.rs, eval-side in src/runtime.rs. Their per-Type leaves
;; (:Vector/length, :HashMap/get, …) remain as the backing impls.
;;
;; Single-impl ops below get a short-name alias (not a dispatch — dispatch is
;; for genuine polymorphism); both the short and long names are honest.
(:wat::core::defalias :wat::core::dissoc  :wat::core::HashMap/dissoc)
(:wat::core::defalias :wat::core::keys    :wat::core::HashMap/keys)
(:wat::core::defalias :wat::core::values  :wat::core::HashMap/values)
(:wat::core::defalias :wat::core::concat  :wat::core::Vector/concat)

;; ─── Polymorphic arithmetic defclauses ───────────────────────────────────────
;;
;; Two layers: a per-Type binary primitive in Rust (:wat::core::<Type>::<op>,
;; always 2-ary) under a polymorphic wat defclause (:wat::core::<op>) that
;; dispatches by arity × arg-Type. Cross-type is rejected by CLAUSE ABSENCE —
;; no mixed-type clause exists, so (:wat::core::+ 1 2.0) → :NoMatchingClause,
;; no special-case Rust check needed.
;;
;; Arity (Lisp/Clojure): +/* → 0-ary identity (0 / 1), 1-ary unchanged, 2-ary
;; binary, 3+ fold. -// → no 0-ary, 1-ary identity-on-left (negate / reciprocal),
;; 2-ary binary, 3+ fold.

(:wat::core::defclause :wat::core::+
  ;; 0-ary identity: i64 0 (Lisp additive identity)
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
    & rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc n))
      (:wat::core::i64::+ x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- (:wat::core::Vector :- [:wat::core::f64])] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::+ acc n))
      (:wat::core::f64::+ x y)
      rest))
  ;; Arc 300 stone C1 — bigint: 1-ary, 2-ary, N-ary fold (mirrors i64/f64
  ;; above, one type over; arbitrary precision — NEVER overflows).
  ([x <- :wat::core::bigint] -> :wat::core::bigint x)
  ([x <- :wat::core::bigint
    y <- :wat::core::bigint] -> :wat::core::bigint (:wat::core::bigint::+ x y))
  ([x <- :wat::core::bigint
    y <- :wat::core::bigint
    & rest <- (:wat::core::Vector :- [:wat::core::bigint])] -> :wat::core::bigint
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::bigint
                       n <- :wat::core::bigint] -> :wat::core::bigint
        (:wat::core::bigint::+ acc n))
      (:wat::core::bigint::+ x y)
      rest))
  ;; Contagion: i64 ⊕ bigint → bigint (i64 promotes via i64::to-bigint; NEVER
  ;; demotes the bigint side back to i64).
  ([x <- :wat::core::i64
    y <- :wat::core::bigint] -> :wat::core::bigint
    (:wat::core::bigint::+ (:wat::core::i64::to-bigint x) y))
  ([x <- :wat::core::bigint
    y <- :wat::core::i64] -> :wat::core::bigint
    (:wat::core::bigint::+ x (:wat::core::i64::to-bigint y)))
  ;; Arc 300 stone C2 — rational: 1-ary identity (a genuine rational is
  ;; never integer-valued — Stone B's invariant — so identity never
  ;; collapses), 2-ary, N-ary fold. The fold step calls the raw per-type
  ;; intrinsic exactly like i64/f64/bigint above; `:wat::core::rational::+`
  ;; itself accepts a bigint accumulator (self-promoted) so the fold can
  ;; carry a COLLAPSED intermediate (this stone's pinned collapse: a
  ;; BigRational result reducing to a whole number becomes bigint) across
  ;; steps without needing a separate contagion arm inside the fold body.
  ([x <- :wat::core::rational] -> :wat::core::rational x)
  ([x <- :wat::core::rational
    y <- :wat::core::rational] -> :wat::core::rational (:wat::core::rational::+ x y))
  ([x <- :wat::core::rational
    y <- :wat::core::rational
    & rest <- (:wat::core::Vector :- [:wat::core::rational])] -> :wat::core::rational
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::rational
                       n <- :wat::core::rational] -> :wat::core::rational
        (:wat::core::rational::+ acc n))
      (:wat::core::rational::+ x y)
      rest))
  ;; Contagion: i64 ⊕ rational → rational (i64 promotes via i64::to-rational).
  ([x <- :wat::core::i64
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::+ (:wat::core::i64::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::i64] -> :wat::core::rational
    (:wat::core::rational::+ x (:wat::core::i64::to-rational y)))
  ;; Contagion: bigint ⊕ rational → rational (bigint promotes via bigint::to-rational).
  ([x <- :wat::core::bigint
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::+ (:wat::core::bigint::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::bigint] -> :wat::core::rational
    (:wat::core::rational::+ x (:wat::core::bigint::to-rational y)))
  ;; Contagion: rational ⊕ f64 → f64 (FLOAT CONTAGION — no collapse; convert
  ;; the rational down to f64, never promotes f64 to rational).
  ([x <- :wat::core::rational
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::+ (:wat::core::rational::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::rational] -> :wat::core::f64
    (:wat::core::f64::+ x (:wat::core::rational::to-f64 y)))
  ;; Arc 300 stone C4 — mixed-float contagion: i64 ⊕ f64 → f64, bigint ⊕ f64
  ;; → f64 (both operand orders; FLOAT CONTAGION — no collapse). Promote the
  ;; non-f64 operand via i64::to-f64 / bigint::to-f64 (both already exist),
  ;; then the existing f64::+. Mirrors C1's i64⊕bigint / C2's rational⊕f64
  ;; contagion arms immediately above, one type-pair over.
  ([x <- :wat::core::i64
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::+ (:wat::core::i64::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::i64] -> :wat::core::f64
    (:wat::core::f64::+ x (:wat::core::i64::to-f64 y)))
  ([x <- :wat::core::bigint
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::+ (:wat::core::bigint::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::bigint] -> :wat::core::f64
    (:wat::core::f64::+ x (:wat::core::bigint::to-f64 y))))

(:wat::core::defclause :wat::core::-
  ;; NO 0-ary clause — :NoMatchingClause fires
  ;; 1-ary per-Type: negate (identity-on-left = 0)
  ([x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::- 0 x))
  ([x <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::- 0.0 x))
  ;; 2-ary
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::- x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::- x y))
  ;; 3+-ary fold
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::- acc n))
      (:wat::core::i64::- x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- (:wat::core::Vector :- [:wat::core::f64])] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::- acc n))
      (:wat::core::f64::- x y)
      rest))
  ;; Arc 300 stone C1 — bigint: 1-ary negate (identity-on-left = 0, promoted
  ;; via i64::to-bigint), 2-ary, N-ary fold (mirrors i64/f64 above).
  ([x <- :wat::core::bigint] -> :wat::core::bigint
    (:wat::core::bigint::- (:wat::core::i64::to-bigint 0) x))
  ([x <- :wat::core::bigint
    y <- :wat::core::bigint] -> :wat::core::bigint (:wat::core::bigint::- x y))
  ([x <- :wat::core::bigint
    y <- :wat::core::bigint
    & rest <- (:wat::core::Vector :- [:wat::core::bigint])] -> :wat::core::bigint
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::bigint
                       n <- :wat::core::bigint] -> :wat::core::bigint
        (:wat::core::bigint::- acc n))
      (:wat::core::bigint::- x y)
      rest))
  ;; Contagion: i64 ⊕ bigint → bigint.
  ([x <- :wat::core::i64
    y <- :wat::core::bigint] -> :wat::core::bigint
    (:wat::core::bigint::- (:wat::core::i64::to-bigint x) y))
  ([x <- :wat::core::bigint
    y <- :wat::core::i64] -> :wat::core::bigint
    (:wat::core::bigint::- x (:wat::core::i64::to-bigint y)))
  ;; Arc 300 stone C2 — rational: 1-ary negate (identity-on-left = 0,
  ;; promoted via i64::to-rational — never collapses: negating a genuine
  ;; rational keeps its denominator unchanged), 2-ary, N-ary fold (mirrors
  ;; the `+` rational arms immediately above the previous defclause, one
  ;; operator over).
  ([x <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::- (:wat::core::i64::to-rational 0) x))
  ([x <- :wat::core::rational
    y <- :wat::core::rational] -> :wat::core::rational (:wat::core::rational::- x y))
  ([x <- :wat::core::rational
    y <- :wat::core::rational
    & rest <- (:wat::core::Vector :- [:wat::core::rational])] -> :wat::core::rational
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::rational
                       n <- :wat::core::rational] -> :wat::core::rational
        (:wat::core::rational::- acc n))
      (:wat::core::rational::- x y)
      rest))
  ;; Contagion: i64 ⊕ rational → rational.
  ([x <- :wat::core::i64
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::- (:wat::core::i64::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::i64] -> :wat::core::rational
    (:wat::core::rational::- x (:wat::core::i64::to-rational y)))
  ;; Contagion: bigint ⊕ rational → rational.
  ([x <- :wat::core::bigint
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::- (:wat::core::bigint::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::bigint] -> :wat::core::rational
    (:wat::core::rational::- x (:wat::core::bigint::to-rational y)))
  ;; Contagion: rational ⊕ f64 → f64 (FLOAT CONTAGION).
  ([x <- :wat::core::rational
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::- (:wat::core::rational::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::rational] -> :wat::core::f64
    (:wat::core::f64::- x (:wat::core::rational::to-f64 y)))
  ;; Arc 300 stone C4 — mixed-float contagion: i64 ⊕ f64 → f64, bigint ⊕ f64
  ;; → f64 (both operand orders; FLOAT CONTAGION). Mirrors the `+` C4 arms
  ;; immediately above the previous defclause, one operator over.
  ([x <- :wat::core::i64
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::- (:wat::core::i64::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::i64] -> :wat::core::f64
    (:wat::core::f64::- x (:wat::core::i64::to-f64 y)))
  ([x <- :wat::core::bigint
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::- (:wat::core::bigint::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::bigint] -> :wat::core::f64
    (:wat::core::f64::- x (:wat::core::bigint::to-f64 y))))

(:wat::core::defclause :wat::core::*
  ;; 0-ary identity: i64 1 (Lisp multiplicative identity)
  ([] -> :wat::core::i64 1)
  ;; 1-ary: per-Type arg unchanged
  ([x <- :wat::core::i64] -> :wat::core::i64 x)
  ([x <- :wat::core::f64] -> :wat::core::f64 x)
  ;; 2-ary
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::* x y))
  ;; 3+-ary fold
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::* acc n))
      (:wat::core::i64::* x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- (:wat::core::Vector :- [:wat::core::f64])] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::* acc n))
      (:wat::core::f64::* x y)
      rest))
  ;; Arc 300 stone C1 — bigint: 1-ary, 2-ary, N-ary fold (mirrors i64/f64
  ;; above, one type over; arbitrary precision — NEVER overflows).
  ([x <- :wat::core::bigint] -> :wat::core::bigint x)
  ([x <- :wat::core::bigint
    y <- :wat::core::bigint] -> :wat::core::bigint (:wat::core::bigint::* x y))
  ([x <- :wat::core::bigint
    y <- :wat::core::bigint
    & rest <- (:wat::core::Vector :- [:wat::core::bigint])] -> :wat::core::bigint
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::bigint
                       n <- :wat::core::bigint] -> :wat::core::bigint
        (:wat::core::bigint::* acc n))
      (:wat::core::bigint::* x y)
      rest))
  ;; Contagion: i64 ⊕ bigint → bigint.
  ([x <- :wat::core::i64
    y <- :wat::core::bigint] -> :wat::core::bigint
    (:wat::core::bigint::* (:wat::core::i64::to-bigint x) y))
  ([x <- :wat::core::bigint
    y <- :wat::core::i64] -> :wat::core::bigint
    (:wat::core::bigint::* x (:wat::core::i64::to-bigint y)))
  ;; Arc 300 stone C2 — rational: 1-ary identity, 2-ary, N-ary fold (mirrors
  ;; the `+`/`-` rational arms above, one operator over).
  ([x <- :wat::core::rational] -> :wat::core::rational x)
  ([x <- :wat::core::rational
    y <- :wat::core::rational] -> :wat::core::rational (:wat::core::rational::* x y))
  ([x <- :wat::core::rational
    y <- :wat::core::rational
    & rest <- (:wat::core::Vector :- [:wat::core::rational])] -> :wat::core::rational
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::rational
                       n <- :wat::core::rational] -> :wat::core::rational
        (:wat::core::rational::* acc n))
      (:wat::core::rational::* x y)
      rest))
  ;; Contagion: i64 ⊕ rational → rational.
  ([x <- :wat::core::i64
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::* (:wat::core::i64::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::i64] -> :wat::core::rational
    (:wat::core::rational::* x (:wat::core::i64::to-rational y)))
  ;; Contagion: bigint ⊕ rational → rational.
  ([x <- :wat::core::bigint
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::* (:wat::core::bigint::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::bigint] -> :wat::core::rational
    (:wat::core::rational::* x (:wat::core::bigint::to-rational y)))
  ;; Contagion: rational ⊕ f64 → f64 (FLOAT CONTAGION).
  ([x <- :wat::core::rational
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::* (:wat::core::rational::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::rational] -> :wat::core::f64
    (:wat::core::f64::* x (:wat::core::rational::to-f64 y)))
  ;; Arc 300 stone C4 — mixed-float contagion: i64 ⊕ f64 → f64, bigint ⊕ f64
  ;; → f64 (both operand orders; FLOAT CONTAGION). Mirrors the `+`/`-` C4 arms
  ;; above, one operator over.
  ([x <- :wat::core::i64
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::* (:wat::core::i64::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::i64] -> :wat::core::f64
    (:wat::core::f64::* x (:wat::core::i64::to-f64 y)))
  ([x <- :wat::core::bigint
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::* (:wat::core::bigint::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::bigint] -> :wat::core::f64
    (:wat::core::f64::* x (:wat::core::bigint::to-f64 y))))

(:wat::core::defclause :wat::core::/
  ;; NO 0-ary clause — :NoMatchingClause fires
  ;; 1-ary per-Type: reciprocal (identity-on-left = 1)
  ([x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::/ 1 x))
  ([x <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::/ 1.0 x))
  ;; 2-ary
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::/ x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::/ x y))
  ;; 3+-ary fold
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::/ acc n))
      (:wat::core::i64::/ x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- (:wat::core::Vector :- [:wat::core::f64])] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::/ acc n))
      (:wat::core::f64::/ x y)
      rest))
  ;; Arc 300 stone C1 — bigint: 1-ary reciprocal, 2-ary. `:wat::core::bigint::/`
  ;; COLLAPSES to `:wat::core::rational` when not evenly divisible (clj: `(/ 1N
  ;; 2N) => 1/2`), so unlike +/-/*, there is deliberately NO N-ary fold arm here
  ;; — folding would feed a possibly-Rational intermediate back into
  ;; `bigint::/`'s bigint-only 2-ary Rust intrinsic on the second+ step, an
  ;; honest TypeMismatch rather than a silent wrong answer. 3+-ary bigint
  ;; division is a clean `:NoMatchingClause` gap (out of C1's scope; C2's
  ;; rational arithmetic is the natural home for a fold that can carry a
  ;; collapsed intermediate).
  ([x <- :wat::core::bigint] -> :wat::core::bigint
    (:wat::core::bigint::/ (:wat::core::i64::to-bigint 1) x))
  ([x <- :wat::core::bigint
    y <- :wat::core::bigint] -> :wat::core::bigint (:wat::core::bigint::/ x y))
  ;; Contagion: i64 ⊕ bigint → bigint (2-ary only, same collapse caveat as above).
  ([x <- :wat::core::i64
    y <- :wat::core::bigint] -> :wat::core::bigint
    (:wat::core::bigint::/ (:wat::core::i64::to-bigint x) y))
  ([x <- :wat::core::bigint
    y <- :wat::core::i64] -> :wat::core::bigint
    (:wat::core::bigint::/ x (:wat::core::i64::to-bigint y)))
  ;; Arc 300 stone C2 — rational: 1-ary reciprocal (COLLAPSE-aware — e.g.
  ;; reciprocal of 1/3 is 3, which collapses to bigint), 2-ary, AND (unlike
  ;; bigint's `/` immediately above) an N-ary fold: `:wat::core::rational::/`
  ;; accepts a bigint accumulator (self-promoted — see its Rust doc), so this
  ;; fold CAN carry a collapsed intermediate across steps — this is the
  ;; "natural home" the bigint comment above points to.
  ([x <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::/ (:wat::core::i64::to-rational 1) x))
  ([x <- :wat::core::rational
    y <- :wat::core::rational] -> :wat::core::rational (:wat::core::rational::/ x y))
  ([x <- :wat::core::rational
    y <- :wat::core::rational
    & rest <- (:wat::core::Vector :- [:wat::core::rational])] -> :wat::core::rational
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::rational
                       n <- :wat::core::rational] -> :wat::core::rational
        (:wat::core::rational::/ acc n))
      (:wat::core::rational::/ x y)
      rest))
  ;; Contagion: i64 ⊕ rational → rational.
  ([x <- :wat::core::i64
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::/ (:wat::core::i64::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::i64] -> :wat::core::rational
    (:wat::core::rational::/ x (:wat::core::i64::to-rational y)))
  ;; Contagion: bigint ⊕ rational → rational.
  ([x <- :wat::core::bigint
    y <- :wat::core::rational] -> :wat::core::rational
    (:wat::core::rational::/ (:wat::core::bigint::to-rational x) y))
  ([x <- :wat::core::rational
    y <- :wat::core::bigint] -> :wat::core::rational
    (:wat::core::rational::/ x (:wat::core::bigint::to-rational y)))
  ;; Contagion: rational ⊕ f64 → f64 (FLOAT CONTAGION).
  ([x <- :wat::core::rational
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::/ (:wat::core::rational::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::rational] -> :wat::core::f64
    (:wat::core::f64::/ x (:wat::core::rational::to-f64 y)))
  ;; Arc 300 stone C4 — mixed-float contagion: i64 ⊕ f64 → f64, bigint ⊕ f64
  ;; → f64 (both operand orders; FLOAT CONTAGION). Mirrors the `+`/`-`/`*`
  ;; C4 arms above, one operator over.
  ([x <- :wat::core::i64
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::/ (:wat::core::i64::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::i64] -> :wat::core::f64
    (:wat::core::f64::/ x (:wat::core::i64::to-f64 y)))
  ([x <- :wat::core::bigint
    y <- :wat::core::f64] -> :wat::core::f64
    (:wat::core::f64::/ (:wat::core::bigint::to-f64 x) y))
  ([x <- :wat::core::f64
    y <- :wat::core::bigint] -> :wat::core::f64
    (:wat::core::f64::/ x (:wat::core::bigint::to-f64 y))))

;; ─── mod / rem / quot — clj's integer-division trio (i64 only) ──────────────
;;
;; Arc 278 numeric-tower increment. Scope: i64 only this stone — bigint/rational
;; mod/rem/quot is a tracked tower-contagion follow-on (named out-of-scope, not
;; deferred). Unlike +/-/*// above, these are 2-ARY ONLY: clj's `mod`/`rem`/
;; `quot` take exactly 2 args (no 0-ary identity, no 1-ary, no N-ary fold, no
;; cross-type contagion arms) — CLAUSE ABSENCE rejects anything else, same
;; no-privacy doctrine as the rest of this file.
(:wat::core::defclause :wat::core::quot
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::quot x y)))

(:wat::core::defclause :wat::core::rem
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::rem x y)))

(:wat::core::defclause :wat::core::mod
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::mod x y)))

;; ─── kwargs-lower — shared kwargs lowering macro (Arc 260.1b Part B) ─────────
;;
;; Extracted from the inlined companion macro body inside `defn`'s kwargs branch.
;; Called by each companion macro that `defn` emits; the companion is now a thin
;; forwarder that supplies the baked-in constants and splices the call-args.
;;
;; Parameters (all :wat::WatAST — macro params are always unevaluated syntax):
;;   impl-kw    — WatAST Keyword node for the $impl fn (e.g. :<name>$impl)
;;   kwargs-ty  — WatAST Keyword node for the Kwargs record type (e.g. :<name>::Kwargs)
;;   field-names — WatAST Vector node of field-name Symbol nodes in declared order
;;   n-pos      — WatAST IntLit: count of leading positional params
;;   ns         — WatAST Keyword node for pascal->kebab-in namespace scoping
;;   call-args  — rest: the actual call arguments (positional + kwargs / map / record)
;;
;; EXTRACTION from WatAST params:
;;   n-pos-int: (Option/expect (string::to-i64 (write-forms n-pos)) "...")
;;   fnames:    (ast->children field-names)   — field-names is already a Vector node
;;   ns-kw:     (keyword/from-string (keyword/to-string ns))
;;
;; NOTE: strip-leading-colon (Arc 260.1b Part A) is a defn and cannot be called
;; from a macro program-body (not in is_pure_total). The `:foo-bar` → `foo-bar` strip
;; uses `(string::subs ks 1 (string::length ks))` directly (always present for callers).
(:wat::core::defmacro :wat::core::kwargs-lower
  [impl-kw    <- :wat::WatAST
   kwargs-ty  <- :wat::WatAST
   field-names <- :wat::WatAST
   n-pos      <- :wat::WatAST
   ns         <- :wat::WatAST
   & call-args <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    [;; Extract typed values from the WatAST params
     n-pos-int  (:wat::core::Option/expect
                   (:wat::core::string::to-i64 (:wat::core::write-forms n-pos))
                   "kwargs-lower: n-pos must be an integer literal")
     fnames     (:wat::core::ast->children field-names)
     nf         (:wat::core::length fnames)
     ns-kw      (:wat::core::keyword/from-string (:wat::core::keyword/to-string ns))
     ;; Split call-args into positional and tail.
     ;; Arc 118.2a — was `(:wat::core::take call-args n-pos-int)` / `(:wat::core::drop …)`. Both
     ;; flipped LAZY; this is `:wat::core::kwargs-lower`, a program-body macro forwarded to from
     ;; EVERY kwargs-style call site (bootstrap-critical, same wall as `:wat::core::defn`'s own
     ;; kwargs-form macro above) — `n-pos-int` is a runtime-computed count (not a small fixed
     ;; literal), so the `rest`×N trick doesn't apply here; `foldl`+`range`+`get`+`conj` (all
     ;; Rust-native, unaffected by the flip) rebuild both slices eagerly.
     call-args-len (:wat::core::length call-args)
     pos        (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                    (:wat::core::conj acc (:wat::core::Option/expect (:wat::core::get call-args i) "kwargs-lower: pos index OOB")))
                  (:wat::core::Vector :wat::WatAST)
                  (:wat::core::range 0 n-pos-int))
     tail       (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                    (:wat::core::conj acc (:wat::core::Option/expect (:wat::core::get call-args i) "kwargs-lower: tail index OOB")))
                  (:wat::core::Vector :wat::WatAST)
                  (:wat::core::range n-pos-int call-args-len))
     tlen       (:wat::core::length tail)
     ;; is-map: tail has exactly 1 element and it is a map literal
     is-map     (:wat::core::if (:wat::core::= tlen 1)
                   
                   (:wat::core::= (:wat::core::ast-kind (:wat::core::first tail)) "map")
                   false)
     ;; is-pt: passthrough — tail has 1 element and it is NOT a map (explicit record)
     is-pt      (:wat::core::if (:wat::core::= tlen 1)
                   
                   (:wat::core::if is-map  false true)
                   false)
     ;; kvflat: flat [k0 v0 k1 v1 …] — either ast->children of map node or tail itself
     kvflat     (:wat::core::if is-map
                   
                   (:wat::core::ast->children (:wat::core::first tail))
                   tail)
     nkv        (:wat::core::i64::/ (:wat::core::length kvflat) 2)]
    (:wat::core::if is-pt
      
      ;; Passthrough: explicit-record call; splice pos-args + single record arg
      `(~impl-kw ~@pos ~(:wat::core::first tail))
      ;; Normal: reorder by field declaration order using pascal->kebab-in matching
      (:wat::core::let
        [ovals
         (:wat::core::foldl
           (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                            fi  <- :wat::core::i64]
             -> (:wat::core::Vector :- [:wat::WatAST])
             (:wat::core::let
               [fn-node
                (:wat::core::Option/expect
                  (:wat::core::get fnames fi)
                  "kwargs-lower: field index OOB")
                fkebab
                (:wat::core::string::pascal->kebab-in ns-kw
                  (:wat::core::ast-name fn-node))
                ;; Scan kvflat for the key matching fkebab; accumulate in a
                ;; single-element Vector (found) to preserve the matched value.
                found
                (:wat::core::foldl
                  (:wat::core::fn [iacc <- (:wat::core::Vector :- [:wat::WatAST])
                                   ki   <- :wat::core::i64]
                    -> (:wat::core::Vector :- [:wat::WatAST])
                    (:wat::core::let
                      [kn
                       (:wat::core::Option/expect
                         (:wat::core::get kvflat (:wat::core::i64::* ki 2))
                         "kwargs-lower: kv-key index OOB")
                       ks
                       (:wat::core::ast-name kn)
                       ;; Strip leading ":" from ":foo-bar" → "foo-bar"
                       ;; (string::strip-leading-colon is a defn, not in is_pure_total;
                       ;;  callers always provide keywords so the colon is always present)
                       kkb
                       (:wat::core::string::subs ks 1 (:wat::core::string::length ks))
                       vn
                       (:wat::core::Option/expect
                         (:wat::core::get kvflat (:wat::core::i64::+ (:wat::core::i64::* ki 2) 1))
                         "kwargs-lower: kv-val index OOB")]
                      ;; Only record the first match (iacc empty → still searching)
                      (:wat::core::if (:wat::core::empty? iacc)
                        
                        (:wat::core::if (:wat::core::= kkb fkebab)
                          
                          (:wat::core::conj iacc vn)
                          iacc)
                        iacc)))
                  (:wat::core::Vector :wat::WatAST)
                  (:wat::core::range 0 nkv))
                ;; If no key matched → macro-error; otherwise take found[0]
                v
                (:wat::core::if (:wat::core::empty? found)
                  
                  (:wat::core::macro-error
                    (:wat::core::string::interpolate "kwargs-lower: missing argument :{fkebab}" :fkebab fkebab))
                  (:wat::core::Option/expect
                    (:wat::core::get found 0)
                    "kwargs-lower: found[0]"))]
               (:wat::core::conj acc v)))
           (:wat::core::Vector :wat::WatAST)
           (:wat::core::range 0 nf))]
        ;; Arc 294 item 9a — aggregate ctor kwargs mode: when kwargs-ty is the sentinel
        ;; `:wat::core::agg-positional`, emit PURE POSITIONAL to the (prime) ctor `(~impl-kw ~@ovals)`
        ;; — no Kwargs-record wrap. Else defn's shape: positional + a trailing Kwargs record.
        (:wat::core::if (:wat::core::= (:wat::core::ast-name kwargs-ty) ":wat::core::agg-positional")
          
          `(~impl-kw ~@pos ~@ovals)
          ;; Arc 294 item 9a — kwargs-lower is the machinery that KNOWS: it holds the reordered
          ;; values positionally, so it constructs the `::Kwargs` bundle through the PRIME
          ;; `:<name>::Kwargs'` (bare is now the kwargs UX macro). Uniform flip, no exemption.
          `(~impl-kw ~@pos (~(:wat::core::keyword-node (:wat::core::string::concat (:wat::core::ast-name kwargs-ty) "'")) ~@ovals)))))))

;; ─── Named-function binding ───────────────────────────────────────
;;
;; Arc 260.1a — defn detects a trailing `& [argspec]` kwargs section:
;;   - `& [name <- :T …]` (Vector tail)  → mints :<name>::Kwargs record, reshapes fn,
;;                                          destructures fields into the body scope.
;;   - `& sym <- :T`      (Symbol tail)  → variadic rest; pass through unchanged (no touch).
;;   - no `&` at all                     → backward-compat pass-through (UNCHANGED).
;;
;; PROGRAM-BODY path (mirrors defservice): top-level `let` evaluates at macro-expand time;
;; quasiquotes appear only at the tail of each branch. Checker skips non-quasiquote `let`/`fn`
;; binders; generated let/fn binders inside quasiquote slots use `symbol-node` (Unquote at
;; definition time → checker passes). `~reshaped-params` and `~let-binders-vec` are both
;; Unquote nodes at definition time → checker skips their contents.
;;
;; Backward-compat: NO `& [...]` kwargs section → defn behaves EXACTLY as today.
;;
;; The original: `defn` macro just binds a function value to a name:
;; (:wat::core::def :name (:wat::core::fn …)). :wat::core::fn is the one and
;; only function constructor; defn forwards the argspec/arrow/ret/body to it
;; unchanged via rest-binder splicing, and an optional metadata-map threads
;; through too — the substrate peels binding-level metadata from the fn-form,
;; so the macro template stays metadata-blind and UNCHANGED.
(:wat::core::defmacro :wat::core::defn
  [name <- :wat::WatAST
   & rest <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, quasiquotes only at branch tails.
  (:wat::core::let
    [;; Arc 300.1 — faithful-Clojure def-name: a namespaced Symbol name
     ;; (`user/main`, `my/ctor`) is the keyword FQDN's twin. Rebuild it as a
     ;; Keyword node (`:user::main`) so BOTH branches below (kwargs + backward-
     ;; compat) consume a keyword uniformly — `keyword/to-string name` and
     ;; `~name` both expect a keyword. Bare (no `/`) → `:name`. Additive: a
     ;; Keyword name (the rust-scheme surface) passes straight through.
     name
     (:wat::core::if (:wat::core::= (:wat::core::ast-kind name) "symbol")
       
       (:wat::core::let
         [name-raw (:wat::core::ast-name name)
          name-fqdn
          (:wat::core::if (:wat::core::string::contains? name-raw "/")
            
            (:wat::core::let
              [slash-parts (:wat::core::string::split name-raw "/")
               ;; `first` returns the element directly (raises if empty);
               ;; `last` returns an Option (arc-278 accessor asymmetry).
               ns-part  (:wat::core::first slash-parts)
               nm-part  (:wat::core::Option/expect (:wat::core::last slash-parts)
                          "defn faithful name: missing name")
               ns-path  (:wat::core::string::join "::" (:wat::core::string::split ns-part "."))]
              (:wat::core::string::concat ":"
                (:wat::core::string::concat ns-path
                  (:wat::core::string::concat "::" nm-part))))
            (:wat::core::string::interpolate ":{name-raw}" :name-raw name-raw))]
         (:wat::core::keyword-node name-fqdn))
       name)
     ;; Arc 109 gamma-i row 6 — a `:- [T U ...]` binder MAY ride at the front of
     ;; `rest`, immediately after the name (before the args-vector) — the same
     ;; position the substrate's `fn`-form peel (`peel_type_binder`,
     ;; `src/function/metadata.rs`) recognizes. `defn` never peeled it: the
     ;; backward-compat branch forwards `rest` unchanged (`~@rest`, unaffected
     ;; either way), but the KWARGS branch indexes `rest` positionally
     ;; (params-vec/ret-type/body-forms), so an unpeeled binder shifts every
     ;; index by 2 and the branch mis-reads `:-` itself as the args-vector.
     ;; `rest2` is `rest` with the binder stripped — used ONLY where the kwargs
     ;; branch below indexes positionally; `rest` itself is UNTOUCHED so the
     ;; backward-compat branch's `~@rest` splice stays byte-identical.
     has-binder   (:wat::core::if (:wat::core::i64::>= (:wat::core::length rest) 1)

                    (:wat::core::let
                      [b0 (:wat::core::Option/expect (:wat::core::get rest 0) "defn binder detect: b0")]
                      (:wat::core::if (:wat::core::= (:wat::core::ast-kind b0) "keyword")
                        (:wat::core::= (:wat::core::ast-name b0) ":-")
                        false))
                    false)
     ;; the binder's bare type-param names, in source order (empty when no binder).
     binder-names-ch
                  (:wat::core::if has-binder
                    (:wat::core::ast->children
                      (:wat::core::Option/expect (:wat::core::get rest 1)
                        "defn binder: `:-` must be followed by a `[...]` vector"))
                    (:wat::core::Vector :wat::WatAST))
     ;; the binder rendered as a `<T,U>` string SUFFIX — the exact shape `name-tp`
     ;; already takes from a `<T,U>`-spelled name, so every downstream
     ;; `{b}::Kwargs{p}` / `{b}$impl{p}` interpolation is unchanged by construction.
     binder-tp    (:wat::core::if has-binder
                    (:wat::core::string::concat "<"
                      (:wat::core::string::concat
                        (:wat::core::string::join ","
                          (:wat::core::foldl
                            (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) nd <- :wat::WatAST]
                              -> (:wat::core::Vector :- [:wat::core::String])
                              (:wat::core::conj acc (:wat::core::ast-name nd)))
                            (:wat::core::Vector :wat::core::String)
                            binder-names-ch))
                        ">"))
                    "")
     rest2        (:wat::core::if has-binder
                    (:wat::core::rest (:wat::core::rest rest))
                    rest)
     ;; Arc 109 gamma-i row 3 CORRECTION — a declaration carrying BOTH a name-embedded
     ;; `<...>` type-param spelling AND a `:- [...]` binder is a contradiction, a property
     ;; of the LANGUAGE, not of the kwargs branch alone. Checked HERE, in the outer let
     ;; shared by BOTH branches below (kwargs AND backward-compat/plain), so one
     ;; `macro-error` covers both paths from one place — the same door `defn` already
     ;; uses for its other macro-time diagnostics (`:632`, `:838`). Mirrors the Rust-side
     ;; message verbatim (`take_declared_binder` in `src/types.rs`; the mirrored check in
     ;; `try_parse_fn_shape_def`, `src/runtime.rs`) so every spelling of the rule reads
     ;; identically. `name-str` was already computed above as part of the name
     ;; normalization; reused here rather than recomputed.
     name-str-parametric? (:wat::core::string::ends-with? (:wat::core::keyword/to-string name) ">")
     _binder-contradiction-check
                  (:wat::core::if (:wat::core::if has-binder name-str-parametric? false)
                    (:wat::core::macro-error
                      (:wat::core::string::interpolate
                        "defn: declaration `{name-str}` carries BOTH a name-embedded `<...>` type-param spelling and a `:- [...]` binder — pick one; a declaration with both is a contradiction, never something to silently resolve"
                        :name-str (:wat::core::keyword/to-string name)))
                    nil)
     params-vec   (:wat::core::first rest2)
     params-ch    (:wat::core::ast->children params-vec)
     params-len   (:wat::core::length params-ch)
     ;; Detect `& [...]` tail: params-len >= 2 AND second-to-last is a Symbol named "&"
     ;; AND last element is a Vector node. `& sym <- :T` (variadic rest) is excluded
     ;; because the element right after `&` is a Symbol (not a Vector).
     has-kwargs   (:wat::core::if (:wat::core::i64::>= params-len 2)
                    
                    (:wat::core::let
                      [stl-node  (:wat::core::Option/expect  
                                   (:wat::core::get params-ch (:wat::core::i64::- params-len 2))
                                   "defn kwargs detect: stl index")
                       last-node (:wat::core::Option/expect  
                                   (:wat::core::get params-ch (:wat::core::i64::- params-len 1))
                                   "defn kwargs detect: last index")]
                      (:wat::core::if (:wat::core::= (:wat::core::ast-kind stl-node) "symbol")
                        
                        (:wat::core::if (:wat::core::= (:wat::core::ast-name stl-node) "&")
                          
                          (:wat::core::= (:wat::core::ast-kind last-node) "vector")
                          false)
                        false))
                    false)]
    (:wat::core::if has-kwargs
      
      ;; ── KWARGS BRANCH (Arc 260.1a) ───────────────────────────────────────────
      (:wat::core::let
        [name-str        (:wat::core::keyword/to-string name)
         ;; ── Arc 278 parametric names: the name / type-param SPLIT ────────────────────
         ;; A kwargs defn MAY be generic (`:my::svc/start<T>` — every parametric
         ;; `defservice`'s auto start/resume is exactly this). Its companions must append
         ;; the suffix to the BASE and RE-ATTACH the params at the end
         ;; (`:my::svc/start::Kwargs<T>`), never the naive `:my::svc/start<T>::Kwargs`
         ;; (a malformed type name). Mirrors the same split in wat/service.wat.
         ;; IDENTITY when there are no params: `name-tp` = "" and `name-base` IS `name-str`,
         ;; so every companion name is byte-identical to the pre-split concatenation.
         ;; DECLARED types/fns carry the params; CTOR / ACCESSOR / by-name-resolution
         ;; keywords (and the companion MACRO's own name) take the bare base.
         ;;
         ;; Arc 109 gamma-i row 6 — a `:- [T ...]` binder is the SECOND spelling of the
         ;; same fact `name-str`'s `<T,U>` suffix already carries (`has-binder`/`binder-tp`,
         ;; computed above from `rest`, before it was ever bound to a defstruct/defmacro
         ;; splice). `name-str-parametric?` names the ORIGINAL name-suffix test so
         ;; `name-base` (which strips a real `<T>` suffix off `name-str`) is unaffected by
         ;; which spelling supplied the params. `name-tp` prefers the binder when present.
         ;; `name-str-parametric?` itself is NOT rebound here — reused from the OUTER let
         ;; (where `_binder-contradiction-check`, above, already rejected the both-spellings
         ;; case for EVERY `defn`, kwargs or plain, before either branch is chosen) — so
         ;; reaching this point with `has-binder` true guarantees `name-str-parametric?` is
         ;; false, and the two never silently disagree here.
         name-parametric? (:wat::core::if has-binder true name-str-parametric?)
         name-base       (:wat::core::if name-str-parametric?
                           (:wat::core::first (:wat::core::string::split name-str "<"))
                           name-str)
         name-tp         (:wat::core::if has-binder
                           binder-tp
                           (:wat::core::if name-str-parametric?
                             (:wat::core::string::subs name-str
                               (:wat::core::string::length name-base)
                               (:wat::core::string::length name-str))
                             ""))
         ;; the companion MACRO's own head — always the bare name (a macro takes no type args)
         name-base-node  (:wat::core::keyword-node
                           (:wat::core::string::interpolate ":{b}" :b name-base))
         ;; :<name>::Kwargs — the minted bundle type. DECLARED → carries the type params.
         kwargs-ty       (:wat::core::keyword/from-string
                           (:wat::core::string::interpolate "{b}::Kwargs{p}" :b name-base :p name-tp))
         kwargs-ty-str   (:wat::core::keyword/to-string kwargs-ty)
         ;; the BARE bundle name — the CONSTRUCTOR head and the ACCESSOR prefix, both of
         ;; which key on the base (identity when the defn is monomorphic).
         kwargs-ty-base-str (:wat::core::string::interpolate "{b}::Kwargs" :b name-base)
         ;; The inner argspec Vector node (the last element of params-ch)
         kw-argvec       (:wat::core::Option/expect  
                            (:wat::core::last params-ch)
                            "defn kwargs: no inner argspec vector")
         kw-ch           (:wat::core::ast->children kw-argvec)
         kw-len          (:wat::core::length kw-ch)
         n-kw-fields     (:wat::core::i64::/ kw-len 3)
         ;; Validate: no nested `&` inside the kwargs section (flat, one level).
         ;; Iterates over field-name positions (0, 3, 6, …); macro-errors on `&`.
         _validate       (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::nil i <- :wat::core::i64]
                             -> :wat::core::nil
                             (:wat::core::let
                               [fname-node (:wat::core::Option/expect  
                                              (:wat::core::get kw-ch (:wat::core::i64::* i 3))
                                              "defn kwargs validate: field name index")]
                               (:wat::core::if (:wat::core::= (:wat::core::ast-name fname-node) "&")
                                 
                                 (:wat::core::macro-error
                                   "defn kwargs section is flat: no nested & — one level")
                                 nil)))
                           nil
                           (:wat::core::range 0 n-kw-fields))
         ;; Mint the kwargs bundle as a STRUCT (defstruct): a kwargs bundle is a LOCAL
         ;; calling-convention artifact (never stored/shipped) that must accept impure args
         ;; (fns, sockets, resources) — so it is impure/struct, NOT a pure record. Arc 259/278.
         record-def      `(:wat::core::defstruct ~kwargs-ty ~kw-argvec)
         ;; HYGIENIC hidden kwargs binder: fresh-symbol stamps a fresh unique scope (arc 274.1) so the
         ;; binder is capture-proof BY CONSTRUCTION — it cannot collide with any caller variable, even one
         ;; literally named "kwargs". (The field binders below stay plain symbol-node — they are
         ;; INTENTIONALLY user-facing, clojure {:keys}.)
         kw-sym          (:wat::core::fresh-symbol "kwargs")
         ;; kwargs-ty as a WatAST Keyword node (needed for with-children)
         kwargs-ty-node  (:wat::core::keyword-node
                            (:wat::core::string::interpolate ":{kwargs-ty-str}" :kwargs-ty-str kwargs-ty-str))
         ;; Build reshaped params children: drop trailing `& [...]` (last 2), append kw-sym <- kwargs-ty
         ;; Arc 118.2a — was `(:wat::core::take ...)`. `take` flipped LAZY (returns Stream); this is
         ;; `:wat::core::defn`'s OWN macro body — it runs at macro-expansion time, BEFORE any
         ;; wat-defined helper (`mapv`/`into`/etc.) is resolvable, and even `conj`ing onto a Stream
         ;; would fail. `foldl`+`get`+`conj` stay Rust-native and eager, unaffected by the flip.
         base-ch         (:wat::core::foldl
                           (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                             (:wat::core::conj acc
                               (:wat::core::Option/expect (:wat::core::get params-ch i) "defn kwargs: base-ch index")))
                           (:wat::core::Vector :wat::WatAST)
                           (:wat::core::range 0 (:wat::core::i64::- params-len 2)))
         arrow-sym       (:wat::core::symbol-node "<-")
         reshaped-ch     (:wat::core::conj
                           (:wat::core::conj
                             (:wat::core::conj base-ch kw-sym)
                             arrow-sym)
                           kwargs-ty-node)
         reshaped-params (:wat::core::with-children params-vec reshaped-ch)
         ;; ret-type: rest2[2] (after params-vec and ->). Arc 109 gamma-i row 6 — reads
         ;; `rest2` (binder-stripped), not `rest`, so a binder-spelled kwargs defn's
         ;; indices realign the same way `params-vec` above already does.
         ret-type        (:wat::core::Option/expect
                            (:wat::core::get rest2 2)
                            "defn kwargs: no return type")
         ;; body forms: rest2[3..] (everything after params-vec -> ret-type)
         ;; Arc 118.2a — was `(:wat::core::drop rest 3)`. `drop` flipped LAZY; this is
         ;; `:wat::core::defn`'s own macro body (bootstrap-critical — see `base-ch` above) and
         ;; `body-forms` is unquote-spliced (`~@body-forms`) below, needing a concrete Vec.
         ;; `rest`/`rest2` stays eager/container-preserving on a real Vector, so drop 3 via
         ;; 3x `rest` (same trick as `:wat::rete::defrule`'s and `:wat::service::defservice`'s
         ;; fixes). Arc 109 gamma-i row 6 — reads `rest2`, see `ret-type` above.
         body-forms      (:wat::core::rest (:wat::core::rest (:wat::core::rest rest2)))
         ;; Build destructure let-binder items:
         ;;   [field1-sym (:<name>::Kwargs/field1 __kwargs__)  field2-sym (…) …]
         ;; Arc 118.2a — `field-indices` (was `(:wat::core::map (fn [i] (* i 3)) (range 0 n-kw-fields))`)
         ;; is ELIMINATED: `map` flipped LAZY and this is `:wat::core::defn`'s own bootstrap-critical
         ;; macro body (same wall as `base-ch`/`body-forms` above). Iterate `(range 0 n-kw-fields)`
         ;; directly (raw positions 0,1,2,…) and multiply by 3 inline instead of pre-computing the
         ;; 0,3,6,… index Vector — same result, one fewer intermediate, no `map` needed at all.
         let-binder-items (:wat::core::foldl
                            (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                             fi  <- :wat::core::i64]
                              -> (:wat::core::Vector :- [:wat::WatAST])
                              (:wat::core::let
                                [i             (:wat::core::i64::* fi 3)
                                 fname-node    (:wat::core::Option/expect
                                                 (:wat::core::get kw-ch i)
                                                 "defn kwargs let-binder: field name index")
                                 fname-str     (:wat::core::ast-name fname-node)
                                 ;; HYGIENIC field binder: REUSE the original argspec symbol node
                                 ;; (fname-node), NOT a string rebuild. The argspec binder and the
                                 ;; fn body are authored in the same hygiene context, so they share a
                                 ;; scope; rebuilding via (symbol-node fname-str) would mint a fresh
                                 ;; EMPTY-scope symbol that matches the body only at top level (where
                                 ;; both are scope-less) but NOT when the defn is emitted inside
                                 ;; another macro's quasiquote (which stamps a scope on both the
                                 ;; argspec and the body). Reusing fname-node keeps binder ≡ body at
                                 ;; any macro-emission depth.
                                 binder-sym    fname-node
                                 ;; Accessor keyword: :<name>::Kwargs/<field-name>
                                 accessor-kw   (:wat::core::keyword/from-string
                                                 (:wat::core::string::concat kwargs-ty-base-str
                                                   (:wat::core::string::interpolate "/{fname-str}" :fname-str fname-str)))
                                 ;; Accessor call: (:<name>::Kwargs/<field> __kwargs__)
                                 accessor-call `(~accessor-kw ~kw-sym)]
                                (:wat::core::conj
                                  (:wat::core::conj acc binder-sym)
                                  accessor-call)))
                            (:wat::core::Vector :wat::WatAST)
                            (:wat::core::range 0 n-kw-fields))
         ;; Wrap let-binder-items as a WatAST::Vector (kw-argvec is the shape template)
         let-binders-vec (:wat::core::with-children kw-argvec let-binder-items)
         ;; ── Arc 260.1b: companion macro additions ────────────────────────────
         ;; impl-head-colon-str: ":<name>$impl" — the $impl fn's keyword string
         impl-head-colon-str (:wat::core::string::interpolate ":{b}$impl{p}" :b name-base :p name-tp)
         ;; the $impl CALL head baked into the companion macro — bare base (a call resolves
         ;; on the base name; the type args are inferred from the arguments there).
         impl-call-colon-str (:wat::core::string::interpolate ":{b}$impl" :b name-base)
         ;; kwargs-ty-colon-str: ":<name>::Kwargs" — the CONSTRUCTOR head kwargs-lower emits.
         kwargs-ty-colon-str (:wat::core::string::interpolate ":{kwargs-ty-base-str}" :kwargs-ty-base-str kwargs-ty-base-str)
         ;; n-pos: count of leading positional params (all params before `& [...]`)
         n-pos               (:wat::core::i64::/ (:wat::core::i64::- params-len 2) 3)
         ;; fname-nodes: Vector<WatAST> of field-name symbol nodes in declared order.
         ;; Arc 118.2a — was `map`; same bootstrap wall as `base-ch`/`body-forms`/`let-binder-items`
         ;; above. `foldl`+`conj` stay Rust-native and eager.
         fname-nodes         (:wat::core::foldl
                               (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                                 (:wat::core::conj acc
                                   (:wat::core::Option/expect
                                     (:wat::core::get kw-ch (:wat::core::i64::* i 3))
                                     "defn kwargs fname-nodes: index")))
                               (:wat::core::Vector :wat::WatAST)
                               (:wat::core::range 0 n-kw-fields))
         ;; field-names-ast-vec: WatAST Vector node of fname symbol nodes
         ;; (baked into the companion macro via (:wat::core::quote ~field-names-ast-vec))
         field-names-ast-vec (:wat::core::with-children kw-argvec fname-nodes)
         ;; impl-name-node: WatAST Keyword node for the $impl fn (used in the def form)
         impl-name-node      (:wat::core::keyword-node impl-head-colon-str)
         impl-call-node      (:wat::core::keyword-node impl-call-colon-str)
         ;; call-args-sym: rest-arg binder for the companion defmacro.
         ;; symbol-node (not fresh-symbol): the argspec name key is stored as a bare string
         ;; by parse_defmacro_form (ident.as_str()); env_key for a scoped fresh-symbol would
         ;; NOT match the bare "call-args" binding key inserted by expand_program_body.
         ;; No capture risk: call-args is an internal macro parameter, not user-visible.
         call-args-sym       (:wat::core::symbol-node "call-args")
         ;; ── W2a: the kwargs-check name + the recursion guard ──
         ;; kwargs-check-name-str: "<name>::kwargs-check" (bare string); kwargs-check-kw: the
         ;; keyword node ":<name>::kwargs-check" for the auto-minted fn's own def head.
         ;; NOTE (arc 278): the ::kwargs-check / ::Coords / ::GrantHandles / ::grant-worker /
         ;; ::revoke-worker family below is minted ONLY for a Peer-bearing (dialing) kwargs
         ;; defn (`mint-coords?`). Those are keyed off the BASE so a parametric name still
         ;; yields a WELL-FORMED companion; a parametric DIALING kwargs defn (type params that
         ;; must reach these carriers) is unproven and out of this strike's scope.
         kwargs-check-name-str (:wat::core::string::interpolate "{name-base}::kwargs-check" :name-base name-base)
         kwargs-check-kw       (:wat::core::keyword-node
                                  (:wat::core::string::interpolate ":{kwargs-check-name-str}" :kwargs-check-name-str kwargs-check-name-str))
         ;; GUARD: this defn is ITSELF a kwargs-check (it has `& [...]`, so it took the kwargs
         ;; branch too) → do NOT mint ITS checker (infinite mint). Suffix test on the bare name.
         is-check (:wat::core::string::ends-with? name-base "::kwargs-check")
         ;; Arc 109 ③ — angle brackets are ILLEGAL for types, so a parametric type slot
         ;; (`Peer<S,R>`) that used to be ONE Keyword node (whose `ast-name` was the whole
         ;; angle-bracket string) now arrives as the reference FORM `(Head :- [args])`, a
         ;; List — and `ast-name` only reads Symbol/Keyword/StringLit, so it raises on that
         ;; shape outright. The kwargs-companion machinery below (`::kwargs-check`/`::Coords`/
         ;; `::GrantHandles`) tests "is this field's type a (Peer :- [S R])?" and swaps
         ;; `Peer`→`Address`/`TypedCapability` in EIGHT places by `ast-name` +
         ;; `string::contains?`/`string::split`+`string::join` — every one assumed the
         ;; Keyword-only shape. These two LOCAL closures (bound here, not top-level `defn`s —
         ;; a program-body macro's default-deny purity gate, arc 249 stone 249.2b-i, refuses
         ;; any user-defined GLOBAL head; a `let`-bound closure invoked by its bound SYMBOL is
         ;; not a keyword-headed call at all, so it never reaches that gate) are the ONE door
         ;; both shapes go through for the eight call sites below.
         ;;
         ;; kwargs-type-slot-name: structural type-NAME text of a kwargs field's type slot,
         ;; whether spelled as a bare Keyword or the `(Head :- [args])` List form (reads the
         ;; List's own head).
         kwargs-type-slot-name
           (:wat::core::fn [node <- :wat::WatAST] -> :wat::core::String
             (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
               (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children node)))
               (:wat::core::ast-name node)))
         ;; kwargs-type-slot-swap-head: rebuild a type-slot node with its HEAD keyword's text
         ;; substring-substituted `old`->`new`, preserving shape: a bare Keyword becomes a
         ;; bare Keyword; a `(Head :- [args])` List keeps the SAME `:- [args]` tail — only
         ;; Head's text changes, so the args (however deeply nested) survive untouched.
         kwargs-type-slot-swap-head
           (:wat::core::fn [node <- :wat::WatAST old <- :wat::core::String new <- :wat::core::String] -> :wat::WatAST
             (:wat::core::let
               [nm         (kwargs-type-slot-name node)
                swapped-kw (:wat::core::keyword-node (:wat::core::string::join new (:wat::core::string::split nm old)))]
               (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
                 (:wat::core::let
                   [ch     (:wat::core::ast->children node)
                    tail   (:wat::core::rest ch)
                    new-ch (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) x <- :wat::WatAST]
                               -> (:wat::core::Vector :- [:wat::WatAST])
                               (:wat::core::conj acc x))
                             (:wat::core::conj (:wat::core::Vector :wat::WatAST) swapped-kw)
                             tail)]
                   (:wat::core::with-children node new-ch))
                 swapped-kw)))
         ;; ── the head-swapped argvec: fold kw-ch, swap Peer TYPE nodes only ──
         ;; kw-ch is flat triples [fname@j·3, arrow@j·3+1, type@j·3+2]; only the type position
         ;; (j mod 3 == 2) is ever swapped, and only when it names a (Peer :- [S R]) (data-typed
         ;; fields pass through as `child` unchanged).
         swapped-ch (:wat::core::foldl
                      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) j <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                        (:wat::core::let
                          [child   (:wat::core::Option/expect (:wat::core::get kw-ch j) "w2a swapped-ch index")
                           is-type (:wat::core::= (:wat::core::i64::mod j 3) 2)
                           nm      (:wat::core::if is-type (kwargs-type-slot-name child) "")
                           is-peer (:wat::core::if is-type (:wat::core::string::contains? nm "Peer") false)
                           swapped (:wat::core::if is-peer
                                     (kwargs-type-slot-swap-head child "Peer" "Address")
                                     child)]
                          (:wat::core::conj acc swapped)))
                      (:wat::core::Vector :wat::WatAST)
                      (:wat::core::range 0 kw-len))
         swapped-argvec (:wat::core::with-children kw-argvec swapped-ch)
         ;; ── arc 170 W2 Strike 1a (record redirect): mint <fqdn>::Coords + checker returns it ──
         ;; The coords CARRIER is a NAMED RECORD, not a positional Tuple — addressed by field NAME,
         ;; so N-service reconciliation has NO positional-accessor cap AND data fields fall out for
         ;; free (a data field is just another named field). `<fqdn>::Coords` is a defRECORD (pure,
         ;; EDN-crossable — it IS the PoolMsg::Setup wire payload) whose fields are the HEAD-SWAPPED
         ;; argvec (each (Peer :- [S R]) → (Address :- [S R]); data fields keep their own type), SAME field
         ;; names + order as `::Kwargs`. Reuses the `swapped-argvec` field nodes verbatim.
         coords-ty-str (:wat::core::string::interpolate "{name-base}::Coords" :name-base name-base)
         coords-kw     (:wat::core::keyword-node (:wat::core::string::interpolate ":{coords-ty-str}" :coords-ty-str coords-ty-str))
         ;; arc 294 9a kwargs flip: bare aggregate name is now the KWARGS MACRO; the POSITIONAL
         ;; ctor moved to the type-name PRIME. This is GENERATED code constructing a just-minted
         ;; aggregate positionally (see kwargs-check-def below) — use the prime, never the bare name.
         coords-prime-kw (:wat::core::keyword-node (:wat::core::string::concat ":" (:wat::core::string::concat coords-ty-str "'")))
         ;; has-peer-field?: does the kwargs section declare ≥1 `(Peer :- [S R])` field? This is the
         ;; SEMANTIC gate for "is this a DIALING work-fn" — the ONLY kind that needs a `::Coords`
         ;; dial-carrier (services are declared `Peer` and dialed; data fields ride along). Read
         ;; the ORIGINAL field types (`kw-ch` position j·3+2), NOT the swapped ones: a swapped
         ;; `Peer`→`Address`, but a defservice `start`/`resume` init-param can ALSO be declared
         ;; `Address` directly (e.g. `s2s-thread-probe.wat`'s `:echo-addr <- Address<…>`) — so
         ;; testing the swapped `Address` would false-match those. Only a real `Peer` field
         ;; (unmangled source) marks a dialing work-fn. This keeps `::Coords` (a pure defrecord)
         ;; from being minted for the many NON-dialing kwargs defns that hold impure fields — every
         ;; `defservice`'s auto `start`/`resume` (`[& [locus <- :wat::spawn::Locus …]]`,
         ;; wat/service.wat:1114) carries a STRUCT `locus`; `probe-kwargs-struct.wat`'s
         ;; `:probe::apply-it` carries a `Fn`. None declare a `Peer`, so none mint Coords → the
         ;; 293.W ImpureFieldInPureAggregate containment they'd otherwise hit never fires. A dialing
         ;; work-fn's own fields are all crossable (Peer→Address + EDN data); a hypothetical
         ;; dialing bundle carrying an impure data field would still (correctly) fail Coords minting
         ;; with a LOCATED 293.W diagnostic naming that field — the honest error, not a Tuple fallback.
         has-peer-field (:wat::core::foldl
                          (:wat::core::fn [acc <- :wat::core::bool i <- :wat::core::i64] -> :wat::core::bool
                            (:wat::core::if acc true
                              (:wat::core::string::contains?
                                (kwargs-type-slot-name
                                  (:wat::core::Option/expect
                                    (:wat::core::get kw-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                    "w2a has-peer-field: type index"))
                                "Peer")))
                          false
                          (:wat::core::range 0 n-kw-fields))
         ;; mint-coords?: the checker itself is a kwargs defn (has `& [...]`) so it re-enters this
         ;; branch — the `is-check` suffix guard stops the infinite mint (no Coords/checker for a
         ;; `::kwargs-check`). Otherwise gate on being a dialing (Peer-bearing) work-fn.
         mint-coords? (:wat::core::if is-check false has-peer-field)
         ;; ── arc 170 C2 D: the CAPABILITY-swapped argvec — (Peer :- [S R]) → (TypedCapability :- [S R]) ──
         ;; A SECOND head-swap, parallel to `swapped-ch` (Address) but targeting the combined
         ;; `(:wat::capability::TypedCapability :- [S R])` surface (capability.wat) instead. This is
         ;; the checker's OWN param typing (so `bracket/uses` passes RAW HANDLES typed as
         ;; TypedCapability — caught by the bodiless-edge assignability check — never erased,
         ;; never a bare Address). `swapped-ch`/`swapped-argvec` (Address) is UNCHANGED and
         ;; still used only for `::Coords`'s field TYPES (the pure crossing carrier). The needle
         ;; is the FULL "wat::kernel::Peer" (not bare "Peer"): Address shares Peers
         ;; `wat::kernel::` namespace so the bare swap works there, but TypedCapability lives in
         ;; `wat::capability::` — the whole qualified head must relocate, not just the tail.
         capswapped-ch (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) j <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                            (:wat::core::let
                              [child   (:wat::core::Option/expect (:wat::core::get kw-ch j) "w2d capswapped-ch index")
                               is-type (:wat::core::= (:wat::core::i64::mod j 3) 2)
                               nm      (:wat::core::if is-type (kwargs-type-slot-name child) "")
                               is-peer (:wat::core::if is-type (:wat::core::string::contains? nm "Peer") false)
                               swapped (:wat::core::if is-peer
                                         (kwargs-type-slot-swap-head child "wat::kernel::Peer" "wat::capability::TypedCapability")
                                         child)]
                              (:wat::core::conj acc swapped)))
                          (:wat::core::Vector :wat::WatAST)
                          (:wat::core::range 0 kw-len))
         capswapped-argvec (:wat::core::with-children kw-argvec capswapped-ch)
         ;; ── ::GrantHandles — the impure, is-peer-FILTERED parent-local carrier ──────────────
         ;; A `defstruct` (impure-permitting, like `::Kwargs` itself) of ONLY the service fields,
         ;; each typed `(TypedCapability :- [Si Ri])` (capswapped). Data fields never enter it — they
         ;; carry no capability to grant. Read `kw-ch`'s ORIGINAL (unswapped) type per field —
         ;; same is-peer test as `has-peer-field`, applied per-field here.
         grant-handles-ty-str (:wat::core::string::interpolate "{name-base}::GrantHandles" :name-base name-base)
         grant-handles-kw     (:wat::core::keyword-node (:wat::core::string::interpolate ":{grant-handles-ty-str}" :grant-handles-ty-str grant-handles-ty-str))
         ;; arc 294 9a kwargs flip: positional ctor of this just-minted aggregate moves to the prime.
         grant-handles-prime-kw (:wat::core::keyword-node (:wat::core::string::concat ":" (:wat::core::string::concat grant-handles-ty-str "'")))
         gh-field-triples (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                               (:wat::core::let
                                 [fname-node (:wat::core::Option/expect (:wat::core::get fname-nodes i) "w2d gh-field: fname index")
                                  orig-ty    (:wat::core::Option/expect
                                               (:wat::core::get kw-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                               "w2d gh-field: type index")
                                  is-peer    (:wat::core::string::contains? (kwargs-type-slot-name orig-ty) "Peer")
                                  cap-ty     (:wat::core::Option/expect
                                               (:wat::core::get capswapped-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                               "w2d gh-field: capswapped type index")]
                                 (:wat::core::if is-peer
                                   (:wat::core::conj (:wat::core::conj (:wat::core::conj acc fname-node) arrow-sym) cap-ty)
                                   acc)))
                             (:wat::core::Vector :wat::WatAST)
                             (:wat::core::range 0 n-kw-fields))
         grant-handles-field-vec (:wat::core::with-children kw-argvec gh-field-triples)
         grant-handles-def (:wat::core::if mint-coords?
                              `(:wat::core::defstruct ~grant-handles-kw ~grant-handles-field-vec)
                              `(:wat::core::do nil))
         ;; ── the Coords record + checker forms (guarded) ──
         ;; GUARD NO-OP = (do nil), NOT an empty (do): an empty `(:wat::core::do)` is ILLEGAL —
         ;; "do form requires at least one form; got zero". `(do nil)` has one form, evaluates to
         ;; nil, is discarded as a harmless top-level form.
         coords-def (:wat::core::if mint-coords?
                      `(:wat::core::defrecord ~coords-kw ~swapped-argvec)
                      `(:wat::core::do nil))
         ;; The checker's body now builds BOTH carriers and returns them as a Tuple:
         ;; (::Coords field-1 …) — a service field is coord'd off the TypedCapability param
         ;; (`TypedCapability/coord`) before entering the pure Address-typed Coords record; a
         ;; data field passes through unchanged (same as before).
         ;; (::GrantHandles svc-field-1 …) — the RAW TypedCapability-typed params, direct, no
         ;; coord call — these stay live/granted-through, never erased.
         ;; Reuses `fname-nodes` (the SAME symbol-node objects bound as the checker's OWN param
         ;; names via `capswapped-argvec` — hygienic by construction, mirrors the $impl
         ;; let-binder reuse above). The gate (param types are now `(TypedCapability :- [S R])` → a
         ;; swapped handle TypeMismatches) and the carrier-assembly (this body) are ONE act.
         coords-ctor-args (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                               (:wat::core::let
                                 [fname-node (:wat::core::Option/expect (:wat::core::get fname-nodes i) "w2d coords-ctor-args: fname index")
                                  orig-ty    (:wat::core::Option/expect
                                               (:wat::core::get kw-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                               "w2d coords-ctor-args: type index")
                                  is-peer    (:wat::core::string::contains? (kwargs-type-slot-name orig-ty) "Peer")
                                  arg-form   (:wat::core::if is-peer
                                               `(:wat::capability::TypedCapability/coord ~fname-node)
                                               fname-node)]
                                 (:wat::core::conj acc arg-form)))
                             (:wat::core::Vector :wat::WatAST)
                             (:wat::core::range 0 n-kw-fields))
         gh-ctor-args (:wat::core::foldl
                        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                          (:wat::core::let
                            [fname-node (:wat::core::Option/expect (:wat::core::get fname-nodes i) "w2d gh-ctor-args: fname index")
                             orig-ty    (:wat::core::Option/expect
                                          (:wat::core::get kw-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                          "w2d gh-ctor-args: type index")
                             is-peer    (:wat::core::string::contains? (kwargs-type-slot-name orig-ty) "Peer")]
                            (:wat::core::if is-peer (:wat::core::conj acc fname-node) acc)))
                        (:wat::core::Vector :wat::WatAST)
                        (:wat::core::range 0 n-kw-fields))
         pair-ty-str  (:wat::core::string::concat "("
                        (:wat::core::string::concat coords-ty-str
                          (:wat::core::string::concat "," (:wat::core::string::concat grant-handles-ty-str ")"))))
         pair-ty-kw   (:wat::core::keyword-node (:wat::core::string::interpolate ":{pair-ty-str}" :pair-ty-str pair-ty-str))
         kwargs-check-def (:wat::core::if mint-coords?
                            `(:wat::core::defn ~kwargs-check-kw [& ~capswapped-argvec] -> ~pair-ty-kw
                               (:wat::core::Tuple (~coords-prime-kw ~@coords-ctor-args) (~grant-handles-prime-kw ~@gh-ctor-args)))
                            `(:wat::core::do nil))
         ;; ── <fqdn>::grant-worker / revoke-worker — unrolled typed grant|revoke over the
         ;; literal service-field list of ::GrantHandles. `handles-sym`/`pid-sym` are reused
         ;; (by identity) between the defn's own param binders and the body's call forms —
         ;; hygienic, mirrors `grantable-self-sym`/`grantable-pids-sym` in wat/service.wat.
         grant-worker-name-str  (:wat::core::string::interpolate "{name-base}::grant-worker" :name-base name-base)
         revoke-worker-name-str (:wat::core::string::interpolate "{name-base}::revoke-worker" :name-base name-base)
         grant-worker-kw  (:wat::core::keyword-node (:wat::core::string::interpolate ":{grant-worker-name-str}" :grant-worker-name-str grant-worker-name-str))
         revoke-worker-kw (:wat::core::keyword-node (:wat::core::string::interpolate ":{revoke-worker-name-str}" :revoke-worker-name-str revoke-worker-name-str))
         gw-handles-sym (:wat::core::symbol-node "handles")
         gw-pid-sym     (:wat::core::symbol-node "pid")
         grant-calls (:wat::core::foldl
                       (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                         (:wat::core::let
                           [fname-node (:wat::core::Option/expect (:wat::core::get fname-nodes i) "w2d grant-calls: fname index")
                            orig-ty    (:wat::core::Option/expect
                                         (:wat::core::get kw-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                         "w2d grant-calls: type index")
                            is-peer    (:wat::core::string::contains? (kwargs-type-slot-name orig-ty) "Peer")
                            fname-str  (:wat::core::ast-name fname-node)
                            acc-kw     (:wat::core::keyword-node
                                         (:wat::core::string::concat ":"
                                           (:wat::core::string::concat grant-handles-ty-str
                                             (:wat::core::string::concat "/" fname-str))))
                            call-form  `(:wat::capability::TypedCapability/grant (~acc-kw ~gw-handles-sym) (:wat::core::Vector :wat::core::i64 ~gw-pid-sym))]
                           (:wat::core::if is-peer (:wat::core::conj acc call-form) acc)))
                       (:wat::core::Vector :wat::WatAST)
                       (:wat::core::range 0 n-kw-fields))
         revoke-calls (:wat::core::foldl
                        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                          (:wat::core::let
                            [fname-node (:wat::core::Option/expect (:wat::core::get fname-nodes i) "w2d revoke-calls: fname index")
                             orig-ty    (:wat::core::Option/expect
                                          (:wat::core::get kw-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                          "w2d revoke-calls: type index")
                             is-peer    (:wat::core::string::contains? (kwargs-type-slot-name orig-ty) "Peer")
                             fname-str  (:wat::core::ast-name fname-node)
                             acc-kw     (:wat::core::keyword-node
                                          (:wat::core::string::concat ":"
                                            (:wat::core::string::concat grant-handles-ty-str
                                              (:wat::core::string::concat "/" fname-str))))
                             call-form  `(:wat::capability::TypedCapability/revoke (~acc-kw ~gw-handles-sym) (:wat::core::Vector :wat::core::i64 ~gw-pid-sym))]
                            (:wat::core::if is-peer (:wat::core::conj acc call-form) acc)))
                        (:wat::core::Vector :wat::WatAST)
                        (:wat::core::range 0 n-kw-fields))
         grant-worker-def (:wat::core::if mint-coords?
                            `(:wat::core::defn ~grant-worker-kw [~gw-handles-sym <- ~grant-handles-kw ~gw-pid-sym <- :wat::core::i64] -> :wat::core::nil
                               (:wat::core::do ~@grant-calls))
                            `(:wat::core::do nil))
         revoke-worker-def (:wat::core::if mint-coords?
                              `(:wat::core::defn ~revoke-worker-kw [~gw-handles-sym <- ~grant-handles-kw ~gw-pid-sym <- :wat::core::i64] -> :wat::core::nil
                                 (:wat::core::do ~@revoke-calls))
                              `(:wat::core::do nil))
         ;; ── <fqdn>::assemble — typed Coords → Kwargs (thread bracket Setup).
         ;; Same field fold as process-work-forms' generated dial-runner: Peer
         ;; fields connect', data fields copy. Minted HERE so the thread locus
         ;; can apply a companion that already lives in this universe (service
         ;; thread launch applies init/serve the same way). Process still
         ;; generates its own assemble into shipped source — separate memory.
         assemble-name-str (:wat::core::string::interpolate "{name-base}::assemble" :name-base name-base)
         assemble-kw       (:wat::core::keyword-node
                             (:wat::core::string::interpolate ":{assemble-name-str}" :assemble-name-str assemble-name-str))
         assemble-deps-sym (:wat::core::symbol-node "deps")
         assemble-p-sym    (:wat::core::symbol-node "p")
         assemble-c-sym    (:wat::core::symbol-node "c")
         kwargs-prime-kw   (:wat::core::keyword-node
                             (:wat::core::string::interpolate "{kwargs-ty-colon-str}'" :kwargs-ty-colon-str kwargs-ty-colon-str))
         assemble-ctor-args
         (:wat::core::foldl
           (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64]
             -> (:wat::core::Vector :- [:wat::WatAST])
             (:wat::core::let
               [fname-node (:wat::core::Option/expect
                             (:wat::core::get fname-nodes i) "assemble-ctor-args: fname index")
                orig-ty    (:wat::core::Option/expect
                             (:wat::core::get kw-ch (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                             "assemble-ctor-args: type index")
                is-peer    (:wat::core::string::contains? (kwargs-type-slot-name orig-ty) "Peer")
                fname-str  (:wat::core::ast-name fname-node)
                acc-kw     (:wat::core::keyword-node
                             (:wat::core::string::concat ":"
                               (:wat::core::string::concat coords-ty-str
                                 (:wat::core::string::concat "/" fname-str))))
                read-form  `(~acc-kw ~assemble-deps-sym)
                form       (:wat::core::if is-peer
                             `(:wat::core::match (:wat::kernel::connect ~read-form)
                                ((:wat::kernel::ConnectOutcome::Connected ~assemble-p-sym) ~assemble-p-sym)
                                ((:wat::kernel::ConnectOutcome::Refused ~assemble-c-sym)
                                  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message ~assemble-c-sym) :wat::core::None :wat::core::None))
                                ((:wat::kernel::ConnectOutcome::Rejected ~assemble-c-sym)
                                  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message ~assemble-c-sym) :wat::core::None :wat::core::None))
                                ((:wat::kernel::ConnectOutcome::Failed ~assemble-c-sym)
                                  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message ~assemble-c-sym) :wat::core::None :wat::core::None)))
                             read-form)]
               (:wat::core::conj acc form)))
           (:wat::core::Vector :wat::WatAST)
           (:wat::core::range 0 n-kw-fields))
         assemble-def (:wat::core::if mint-coords?
                        `(:wat::core::defn ~assemble-kw
                           [~assemble-deps-sym <- ~coords-kw] -> ~kwargs-ty-node
                           (~kwargs-prime-kw ~@assemble-ctor-args))
                        `(:wat::core::do nil))]
        ;; Arc 260.1b: emit record-def + $impl fn (under :<name>$impl) + companion defmacro (:name)
        ;; The companion macro is a THIN FORWARDER to :wat::core::kwargs-lower (Part B dedup).
        ;; Values baked in at defn-expansion time via ~ (depth-1 unquotes from the outer quasiquote):
        ;;   ~n-pos, ~name-str, ~impl-name-node, ~kwargs-ty-colon-str, ~field-names-ast-vec.
        ;; The companion's program-body (let) binds these baked values and builds the
        ;; kwargs-lower call form at companion-invocation time; kwargs-lower macro expands it.
        ;;
        ;; HYGIENE NOTE: All symbol binders inside this quasiquote use ~(symbol-node "name")
        ;; (depth-1 unquote that fires at defn-expansion time, producing a Symbol value).
        ;; Literal Symbol nodes at binder positions would trip the defn macro's own
        ;; check_quasiquote_for_literal_binders gate (ProgramBodyIntroducesName). The Unquote
        ;; form `~(...)` is a List node at check time — not a Symbol — so it passes the gate.
        `(:wat::core::do
           ~record-def
           (:wat::core::def ~impl-name-node
             (:wat::core::fn ~reshaped-params -> ~ret-type
               (:wat::core::let ~let-binders-vec ~@body-forms)))
           (:wat::core::defmacro ~name-base-node
             [& ~call-args-sym <- (:wat::core::Vector :- [:wat::WatAST])]
             -> :wat::WatAST
             ;; ── Thin forwarder to :wat::core::kwargs-lower ───────────────────────
             ;; Baked-in constants (substituted at defn-expansion time via depth-1 ~):
             ;;   _kl-impl: keyword node for the $impl fn
             ;;   _kl-kty:  keyword node for the Kwargs type
             ;;   _kl-fvec: the field-names Vector AST node (via quote, for ast->children)
             ;;   _kl-np:   i64 literal: count of positional params
             ;;   _kl-ns:   keyword node for function namespace (for pascal->kebab-in)
             (:wat::core::let
               [~(:wat::core::symbol-node "_kl-impl") ~impl-call-node
                ~(:wat::core::symbol-node "_kl-kty")  (:wat::core::keyword-node ~kwargs-ty-colon-str)
                ~(:wat::core::symbol-node "_kl-fvec") (:wat::core::quote ~field-names-ast-vec)
                ~(:wat::core::symbol-node "_kl-np")   ~n-pos
                ~(:wat::core::symbol-node "_kl-ns")   (:wat::core::keyword-node (:wat::core::string::concat ":" ~name-base))]
               `(:wat::core::kwargs-lower ~_kl-impl ~_kl-kty ~_kl-fvec ~_kl-np ~_kl-ns ~@call-args)))
           ~coords-def                  ;; ← W2 record redirect: <fqdn>::Coords (before the checker refs it).
           ~grant-handles-def           ;; ← C2 D: <fqdn>::GrantHandles (before the checker refs it).
           ~kwargs-check-def            ;; ← W2a/C2 D. Order-independent (refs only literal Coords/GrantHandles types).
           ~grant-worker-def            ;; ← C2 D: <fqdn>::grant-worker.
           ~revoke-worker-def           ;; ← C2 D: <fqdn>::revoke-worker.
           ~assemble-def))              ;; ← Coords → Kwargs (thread Setup).
      ;; ── BACKWARD-COMPAT PASS-THROUGH (no kwargs section) ────────────────────
      `(:wat::core::def ~name (:wat::core::fn ~@rest)))))

;; Restrictions live as a :restricted-to key in the metadata-map on def/defn
;; (e.g. {:restricted-to [<prefix-kw>…]}); the substrate enforces it.

;; ─── Threading macros `->` / `->>` ───────────────────────────────
;;
;; Thread-first `->`: inject acc as the FIRST arg of each step.
;;   (-> x (f a b) g)  =>  (g (f x a b))
;; A list step `(f a…)` => `(f acc a…)`; a bare symbol/keyword step `f` => `(f acc)`.
;; Empty-list step `()`: Option/expect on (first ()) fires "-> step has no head"
;;   as a panic_any(AssertionPayload) at macro-expansion time (during startup).
(:wat::core::defmacro :wat::core::->
  [acc <- :wat::WatAST & steps <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) 
          `(~(:wat::core::first step) ~a ~@(:wat::core::rest step))
          `(~step ~a)))
    acc
    steps))

;; Thread-last `->>`: inject acc as the LAST arg of each step.
;;   (->> x (f a b) g)  =>  (g (f a b x))
;; A list step `(f a…)` => `(f a… acc)`; a bare symbol/keyword step `f` => `(f acc)`.
;; Empty-list step `()`: ~@() splices nothing, yielding `(acc)` — expansion succeeds
;;   but eval rejects the integer-head form with MalformedForm at runtime.
(:wat::core::defmacro :wat::core::->>
  [acc <- :wat::WatAST & steps <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) 
          `(~@step ~a)
          `(~step ~a)))
    acc
    steps))

;; Arc 258 Stone 258.2a — cond reborn as a wat macro over bare if.
;; (cond (test body) … (:else body)) expands to nested bare (:wat::core::if …).
;; The legacy annotated form (cond -> :T arm…) is also accepted: the first
;; clause is the symbol `->` (not a List), so we strip -> and :T and
;; re-expand the remainder as a bare cond.
;;
;; cond is TOTAL: a terminal (:else body) arm is required.
;;
;; EMPTY clause list → expansion-time MacroError via keyword/from-string:
;;   (:wat::core::keyword/from-string ":else ...") rejects ':'-prefixed input with a
;;   RuntimeError (via EvalBreak::Diagnostic), which propagates as StartupError::Macro
;;   rather than panic_any, so run_err can capture it. The error message contains ":else".
;; :else arm → emit its body unconditionally (terminal).
;; test arm → (if test body (cond rest…)) and re-expand to fixpoint.
;;
;; Detecting :else: compare head structurally with (first `(:else)) — both are
;; Value::wat__WatAST, so (= head (first `(:else))) is safe for any non-List head
;; (returns false for integers/symbols, true only for the :else keyword form).
;;
;; empty? is checked FIRST (before any Option/expect) so the empty-clause case
;; goes through the RuntimeError channel, not panic_any.
(:wat::core::defmacro :wat::core::cond
  [& clauses <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::empty? clauses)
    ;; empty clause list — non-exhaustive / no terminal :else. Arc 258 Stone 258.2b: use the
    ;; first-class macro-error primitive to abort with a clean diagnostic. This replaces the
    ;; old keyword-sentinel hack (keyword/from-string with a diagnostic name) which carried a
    ;; near-theoretical slip if every arm body was itself a keyword. macro-error returns Err
    ;; directly — the macro engine wraps it into a catchable MacroError without panic or noise.
    (:wat::core::macro-error "cond: non-exhaustive — needs a terminal :else arm")
    (:wat::core::if (:wat::core::List? (:wat::core::first clauses))
      ;; First clause is a List — bare form: (cond (test body) … (:else body))
      (:wat::core::let [arm  (:wat::core::first clauses)
                        head (:wat::core::first arm)]
        (:wat::core::if (:wat::core::List? head)
          ;; test arm — head is a sub-list like (= 1 2): (if head body (cond rest…))
          `(:wat::core::if
              ~head
              ~(:wat::core::second arm)
              (:wat::core::cond ~@(:wat::core::rest clauses)))
          ;; non-List head — detect :else by structural comparison with the :else keyword form.
          ;; (first `(:else)) returns bare WatAST::Keyword(":else") after arc-278 flip.
          ;; = on two Value::wat__WatAST nodes uses structural PartialEq (safe for any variant pair).
          (:wat::core::if (:wat::core::= head (:wat::core::first `(:else)))
            ;; :else terminal arm — emit body unconditionally
            (:wat::core::second arm)
            ;; other non-List head — treat as test arm (v1 fallback for malformed input)
            `(:wat::core::if
                ~head
                ~(:wat::core::second arm)
                (:wat::core::cond ~@(:wat::core::rest clauses))))))
      ;; First clause is NOT a List (it is the -> symbol) — annotated form.
      ;; Strip -> and :T (first two elements) and re-expand as bare cond.
      `(:wat::core::cond ~@(:wat::core::rest (:wat::core::rest clauses))))))

;; ─── keyword/of — RETIRED (STONE-defservice-emits-the-binder, arc 109) ─────────────
;;
;; `keyword/of` built the parametric keyword `:Head<arg1,arg2>` from keyword args —
;; its ENTIRE purpose was minting the now-retired angle spelling. There is no surviving
;; version to emit: the replacement, `(Head :- [args])`, is a compound FORM (a List), not
;; a keyword atom, so this macro cannot be "fixed" to return one without changing its
;; return contract from `:wat::WatAST`-as-keyword to `:wat::WatAST`-as-list, and nothing
;; in the stdlib actually needs that — every real minting site (`wat/service.wat`) already
;; mints the reference FORM directly via quasiquote (`` `(~base-kw :- [~@args])` ``), never
;; through this door. Its one caller was a test fixture exercising macro-in-template-position
;; firing (`tests/macros/probe_arc249_4_rehome_in_wat_kw_of_tmpl.wat`) — moved to a local
;; test-only macro that keeps testing the SAME property without minting an angle keyword.
;; See `tests/macros/probe_arc249_4_rehome_in_wat.rs`'s `keyword_of_fires_in_template_position`.

;; Stone 245.8 — Polymorphic ordering defclauses RETIRED.
;; `<`/`>`/`<=`/`>=` are now a relational check-side intrinsic (`infer_ordering`
;; in src/check.rs), the sibling of `infer_equality`. The runtime dispatch arms
;; in `dispatch_keyword_head_value` (src/runtime.rs) route directly to `eval_compare`.
;; The per-Type leaves (`:wat::core::i64::<`, `:wat::core::f64::<`, etc.) remain
;; as the type-locked tier in Rust.

;; ─── Instinct-faithful ordering surface (Arc 251 Stone) ──────────────────────
;;
;; `sort'` is the Rust primitive (comparator-sort engine; fn-first `(sort' cmp xs)`).
;; `sort` and `sort-by` are Clojure-exact multi-arity defclauses over `sort'` + `<`.
;; Dispatch is purely by arity (sort: 1 vs 2; sort-by: 2 vs 3).
;; All clauses auto-generalize over bare type-vars T and K (Arc 256 / Stone 251.7).

(:wat::core::defclause :wat::core::sort
  ;; 1-ary: natural ascending — default comparator is <
  ;; T auto-generalizes (bare uppercase type-var, Arc 256 / Stone 251.7).
  ([coll <- (:wat::core::Vector :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::sort'
      (:wat::core::fn [a <- :T b <- :T] -> :wat::core::bool
        (:wat::core::< a b))
      coll))
  ;; 2-ary: user-supplied boolean less-than comparator (fn-first, Clojure idiom).
  ;; Cmp is a bare type-var that unifies with the caller's [T T :-> bool].
  ([cmp  <- :Cmp
    coll <- (:wat::core::Vector :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::sort' cmp coll)))

(:wat::core::defclause :wat::core::sort-by
  ;; 2-ary: key function only — default comparator is < on the keys.
  ;; Keyfn is a bare type-var that unifies with the caller's [T :-> K].
  ([keyfn <- :Keyfn
    coll  <- (:wat::core::Vector :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::sort'
      (:wat::core::fn [a <- :T b <- :T] -> :wat::core::bool
        (:wat::core::< (keyfn a) (keyfn b)))
      coll))
  ;; 3-ary: key function + comparator on keys.
  ;; Keyfn and Cmp are bare type-vars.
  ([keyfn <- :Keyfn
    cmp   <- :Cmp
    coll  <- (:wat::core::Vector :- [T])] -> (:wat::core::Vector :- [T])
    (:wat::core::sort'
      (:wat::core::fn [a <- :T b <- :T] -> :wat::core::bool
        (cmp (keyfn a) (keyfn b)))
      coll)))

;; ── nth-spec — the wat ORACLE for the native `:wat::core::nth` (stone 118.B4-0) ────────────────
;;
;; ⚠ BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — this header used to call
;; `nth` "the positional, TOTAL accessor" in the same breath as "RAISING on out-of-range", which
;; is a contradiction a reader would trust: the CONTRACT reads as total ("there IS an i-th
;; element; give it or fail" — never an `Option`, never a caller-visible undefined case), but the
;; FUNCTION itself is partial by this codebase's own definition of `total` (an ordinary value on
;; every input, no raise) — it raises via `Option/expect` on out-of-range. `get` is the genuinely
;; total one (`[(Vec :- [T]) i64 :-> (Option :- [T])]`, `None` on out-of-range, never raises).
;;
;; `Vector/get` is the associative, nil-safe form. `nth` is Clojure's positional idiom: the i-th
;; element returned as `T`, RAISING on out-of-range — "there IS an i-th element; give it or
;; fail." Sugar over `Option/expect (Vector/get …)`.
;;
;; ── B4-i widened nth to (Seqable :- [T]) (arc 118); B4-iii — THE WALL closes it again ────────
;;
;; The header's argument above is unchanged: nth's CONTRACT still reads as total, its FUNCTION is
;; still partial. B4-i widened the receiver set with a fourth, O(n) `(Seqable :- [T])` arm reached only
;; by Stream (Vector/PersistentVector/List all resolve to an earlier, O(1) arm first) — walking
;; via `nth-spec-walk`/`:wat::stream::next`. Three O(1) arms, once per container that has `get`
;; (byte-identical modulo receiver type — the "eager indexable container" gap the 294 seam already
;; records for `reduce`'s three eager arms; not collapsed here).
;;
;; Stone 118.B4-iii — THE WALL removes that fourth arm (and `nth-spec-walk`, its sole caller):
;; `(nth s i)` on a Stream was O(i) via the walk, identical syntax to the O(1) Vector case — a
;; complexity lie. `nth-spec` must classify the SAME receiver set the native `nth` now accepts
;; (`StreamContainer::nth_indexable()`, `seq_container.rs`) or the oracle and the native disagree
;; about what they cover and the differential test silently stops proving anything about Stream.
;; Positional access on a lazy seq is spelled `(drop s i)` then `next` now — which is what it does.
;;
;; ── B4-0 renamed this to `nth-spec`; the public `:wat::core::nth` is now a Rust intrinsic ─────
;;
;; What moved is only its NAME and its role: `:wat::core::nth` (`src/runtime.rs`, `eval_nth`) is
;; the fast native kernel; this clause is the ORACLE that keeps it honest via a differential test
;; (`wat-tests/core/core-nth-differential.wat`). Same shape as `:wat::rete::insert-all-spec`
;; (`wat/rete.wat:1508`): "the native kernel is the fast impl, the spec keeps it honest."
;; ⚠ `nth-spec` MUST NEVER delegate to `nth` — a spec that calls its subject proves nothing
;; (`[[feedback_an_oracle_must_be_written_in_the_other_language]]`).
;; The native `nth` is NOT promoted from calling this clause: `nth`'s existing callers
;; (`wat/bracket.wat`, `wat/fix.wat`, `wat/service.wat`, …) keep saying `nth` and now silently
;; reach the native — that is the point of the rename, not an accident.
(:wat::core::defclause :wat::core::nth-spec
  ([v <- (:wat::core::Vector :- [T]) i <- :wat::core::i64] -> :T
    (:wat::core::Option/expect (:wat::core::get v i) "nth: index out of range"))
  ([v <- (:wat::core::PersistentVector :- [T]) i <- :wat::core::i64] -> :T
    (:wat::core::Option/expect (:wat::core::get v i) "nth: index out of range"))
  ([v <- (:wat::core::List :- [T]) i <- :wat::core::i64] -> :T
    (:wat::core::Option/expect (:wat::core::get v i) "nth: index out of range")))

;; ─── format — opinionated named-template printf (arc 279) ────────────────────
;;
;; `(:wat::core::format "{greeting}, {name}!" :name "ada" :greeting "hello")`
;;   → "hello, ada!"
;;
;; It is a MACRO (the kwargs doctrine): the template is parsed at EXPAND TIME and
;; the form compiles to a lean `(:wat::core::string::concat <static> (:wat::core::str val) …)`.
;; The template, placeholder names, and kwarg labels EVAPORATE — zero runtime template cost.
;;
;; Rules (strict, no config):
;;   - Placeholders: `{name}` — named, never positional.
;;   - Trailing kwargs: `:name val` pairs (out-of-order OK).
;;   - Rendered UNQUOTED via (:wat::core::str val): String→itself, i64→digits, etc.
;;   - Every `{name}` MUST have a matching `:name` kwarg (else macro-error).
;;   - Every `:name` MUST appear in the template (else macro-error).
;;   - Template must be a string LITERAL (not a variable) — static parse.
;;   - Literal braces: `{{` → `{`, `}}` → `}` (doubled-brace escape, Rust/Python convention).
;;     Collapsed at EXPAND TIME by the macro — zero runtime cost.
;;     A lone `{` or `}` that is not part of a placeholder or a double is a macro-error.
;;   - Template must not contain `"` characters (macro-error guard).
;;
;; Implementation (two-pass char-walk tokenizer, arc 279.1):
;;   1. Extract template string literal from first arg via ast-kind/ast-name.
;;   2. Fold trailing `:name val` pairs into a (HashMap :- [String WatAST]) (kwargs-map).
;;   3. Pass 1 — tokenize: build char vector via (map subs (range 0 length)), then foldl over
;;      a Tuple(mode, pending, buf, segments) accumulator per the state-machine transition table.
;;      Finalize: error on lone brace / unclosed name, flush final text segment.
;;   4. Pass 2 — emit: foldl segments → pieces (Vector :- [WatAST]) + used-set (HashMap :- [String bool]).
;;      kind=="text" → String literal AST node; kind=="slot" → (:wat::core::str val-ast).
;;   5. Strict check: every kwarg key in used-set (else macro-error).
;;   6. Emit: (:wat::core::string::concat piece …).
;;
;; Static text nodes: produced via
;;   (Option/expect (first (ast->children (read-string (string::concat "\"" text "\"")))))
;; at expand time. read-string returns List([WatAST::String(text)]); we extract the String node.
;; Limitation: template text segments must not contain `"` (guard above catches this).
;;
;; See wat/service.wat ~55–110 for the kwargs-fold pattern.
;;
(:wat::core::defmacro :wat::core::format
  [tmpl <- :wat::WatAST
   & opts <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    ;; ── 1. Extract the template string literal ───────────────────────
    [tmpl-str   (:wat::core::if
                  (:wat::core::= (:wat::core::ast-kind tmpl) "string")
                  
                  (:wat::core::ast-name tmpl)
                  (:wat::core::macro-error
                    "format: first argument must be a string literal"))
     ;; Guard: template segments must not contain `"` (read-string would produce broken source).
     _no-quotes (:wat::core::if
                  (:wat::core::string::contains? tmpl-str "\"")
                  
                  (:wat::core::macro-error
                    "format: template must not contain quote characters")
                  nil)

     ;; ── 2. Fold trailing :name val pairs into a kwargs map ───────────
     ;; opts is the rest Vector: [:name val :name2 val2 …].
     opts-len    (:wat::core::length opts)
     n-pairs     (:wat::core::i64::/ opts-len 2)
     ;; Even-length guard.
     _even-check (:wat::core::if
                   (:wat::core::= (:wat::core::i64::* n-pairs 2) opts-len)
                   
                   nil
                   (:wat::core::macro-error
                     "format: trailing kwargs must be :name value pairs — odd count"))
     ;; Build kwargs-map: (HashMap :- [String WatAST]) (kwarg-name-string → value AST node).
     kwargs-map  (:wat::core::foldl
                   (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::String :wat::WatAST])
                                    i <- :wat::core::i64]
                     -> (:wat::core::HashMap :- [:wat::core::String :wat::WatAST])
                     (:wat::core::let
                       [k     (:wat::core::i64::* i 2)
                        k-ast (:wat::core::Option/expect  
                                 (:wat::core::get opts k)
                                 "format: kwargs pair key missing")
                        key   (:wat::core::if
                                (:wat::core::= (:wat::core::ast-kind k-ast) "keyword")
                                
                                (:wat::core::keyword/to-string k-ast)
                                (:wat::core::macro-error
                                  "format: kwargs key must be a keyword (e.g. :name)"))
                        val   (:wat::core::Option/expect  
                                 (:wat::core::get opts (:wat::core::i64::+ k 1))
                                 "format: kwargs pair value missing")]
                       (:wat::core::HashMap/assoc m key val)))
                   (:wat::core::HashMap :wat::core::String :wat::WatAST)
                   (:wat::core::range 0 n-pairs))

     ;; ── 3. Pass 1 — tokenize chars → segment list ───────────────────
     ;; Build char vector: each element is a single-char String.
     tmpl-len    (:wat::core::string::length tmpl-str)
     ;; Arc 118.2a — was `(:wat::core::map ...)`. `map` flipped LAZY (returns a `Stream`, not
     ;; a `Vector`); `format` is itself a macro invoked from inside OTHER macros' bodies at
     ;; macro-expansion time (e.g. `wat/lint.wat`), so `chars` must stay a concrete
     ;; `(Vector :- [String])` RIGHT NOW, without depending on any wat-defined eager materializer
     ;; (untested at this bootstrap phase — see `crate::stream::NativeLazyCell`'s doc).
     ;; `foldl`+`conj` stay Rust-native and eager, unaffected by the flip.
     chars       (:wat::core::foldl
                   (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
                     (:wat::core::conj acc (:wat::core::string::subs tmpl-str i (:wat::core::i64::+ i 1))))
                   (:wat::core::Vector :wat::core::String)
                   (:wat::core::range 0 tmpl-len))

     ;; Accumulator: Tuple(mode, pending, buf, segments)
     ;;   mode    : String — "text" | "name"
     ;;   pending : String — "none" | "open" | "close"
     ;;   buf     : String — accumulated chars (text or placeholder name)
     ;;   segments: (Vector :- [Tuple(kind,payload)]) — emitted tokens
     ;;
     ;; Transition table (verbatim from DESIGN-279.1-escape.md):
     ;;
     ;; mode=="text", pending=="open":
     ;;   c=="{" → {{ literal: (text,"none",buf+"{",segs)
     ;;   c=="}" → macro-error "format: empty placeholder {} in template"
     ;;   else   → flush buf as text-seg if non-empty, ("name","none",c,segs')
     ;;
     ;; mode=="text", pending=="close":
     ;;   c=="}" → }} literal: (text,"none",buf+"}",segs)
     ;;   else   → macro-error "format: lone '}' in template — use '}}' for a literal brace"
     ;;
     ;; mode=="text", pending=="none":
     ;;   c=="{" → (text,"open",buf,segs)   (defer)
     ;;   c=="}" → (text,"close",buf,segs)  (defer)
     ;;   else   → (text,"none",buf+c,segs)
     ;;
     ;; mode=="name" (pending always "none"):
     ;;   c=="}" → emit Tuple("slot",buf), (text,"none","",segs')
     ;;   c=="{" → macro-error "format: '{' inside placeholder name — unclosed '{'?"
     ;;   else   → ("name","none",buf+c,segs)

     ;; Accumulator layout: Tuple(Tuple(mode, pending), Tuple(buf, segs))
     ;; — a nested pair-of-pairs, since first/second/third cover indices 0-2
     ;; on a Tuple but there is no fourth accessor and `last` requires a Vector.
     ;; Access: mode=(first (first acc)), pending=(second (first acc)),
     ;;         buf=(first (second acc)), segs=(second (second acc)).
     tok-state   (:wat::core::foldl
                   (:wat::core::fn [acc <- :wat::core::Tuple
                                    c   <- :wat::core::String]
                     -> :wat::core::Tuple
                     (:wat::core::let
                       [mp      (:wat::core::first acc)
                        bs      (:wat::core::second acc)
                        mode    (:wat::core::first mp)
                        pending (:wat::core::second mp)
                        buf     (:wat::core::first bs)
                        segs    (:wat::core::second bs)]
                       (:wat::core::if
                         (:wat::core::= mode "text")
                         
                         (:wat::core::if
                           (:wat::core::= pending "open")
                           
                           ;; mode=="text", pending=="open"
                           (:wat::core::if
                             (:wat::core::= c "{")
                             
                             ;; {{ → literal {
                             (:wat::core::Tuple
                               (:wat::core::Tuple "text" "none")
                               (:wat::core::Tuple (:wat::core::string::concat buf "{") segs))
                             (:wat::core::if
                               (:wat::core::= c "}")
                               
                               ;; {} → error
                               (:wat::core::macro-error
                                 "format: empty placeholder {} in template")
                               ;; { followed by other char → open placeholder
                               ;; flush buf as text segment if non-empty, then start name
                               (:wat::core::let
                                 [segs-after (:wat::core::if
                                               (:wat::core::String/empty? buf)
                                               
                                               segs
                                               (:wat::core::conj segs
                                                 (:wat::core::Tuple "text" buf)))]
                                 (:wat::core::Tuple
                                   (:wat::core::Tuple "name" "none")
                                   (:wat::core::Tuple c segs-after)))))
                           (:wat::core::if
                             (:wat::core::= pending "close")
                             
                             ;; mode=="text", pending=="close"
                             (:wat::core::if
                               (:wat::core::= c "}")
                               
                               ;; }} → literal }
                               (:wat::core::Tuple
                                 (:wat::core::Tuple "text" "none")
                                 (:wat::core::Tuple (:wat::core::string::concat buf "}") segs))
                               ;; lone } → error
                               (:wat::core::macro-error
                                 "format: lone '}' in template — use '}}' for a literal brace"))
                             ;; mode=="text", pending=="none"
                             (:wat::core::if
                               (:wat::core::= c "{")
                               
                               (:wat::core::Tuple
                                 (:wat::core::Tuple "text" "open")
                                 (:wat::core::Tuple buf segs))
                               (:wat::core::if
                                 (:wat::core::= c "}")
                                 
                                 (:wat::core::Tuple
                                   (:wat::core::Tuple "text" "close")
                                   (:wat::core::Tuple buf segs))
                                 (:wat::core::Tuple
                                   (:wat::core::Tuple "text" "none")
                                   (:wat::core::Tuple (:wat::core::string::concat buf c) segs))))))
                         ;; mode=="name" (pending always "none")
                         (:wat::core::if
                           (:wat::core::= c "}")
                           
                           ;; close placeholder: emit slot segment
                           (:wat::core::Tuple
                             (:wat::core::Tuple "text" "none")
                             (:wat::core::Tuple ""
                               (:wat::core::conj segs (:wat::core::Tuple "slot" buf))))
                           (:wat::core::if
                             (:wat::core::= c "{")
                             
                             ;; { inside name → error
                             (:wat::core::macro-error
                               "format: '{' inside placeholder name — unclosed '{'?")
                             ;; accumulate name char
                             (:wat::core::Tuple
                               (:wat::core::Tuple "name" "none")
                               (:wat::core::Tuple (:wat::core::string::concat buf c) segs)))))))
                   (:wat::core::Tuple
                     (:wat::core::Tuple "text" "none")
                     (:wat::core::Tuple "" (:wat::core::Vector :wat::core::Tuple)))
                   chars)

     ;; ── Finalization: inspect tok-state, error on bad endings ────────
     ;; Extract final accumulator fields from the nested pair-of-pairs.
     fin-mp      (:wat::core::first tok-state)
     fin-bs      (:wat::core::second tok-state)
     fin-mode    (:wat::core::first fin-mp)
     fin-pending (:wat::core::second fin-mp)
     fin-buf     (:wat::core::first fin-bs)
     fin-segs    (:wat::core::second fin-bs)

     ;; Check for trailing lone brace or unclosed name.
     _fin-check  (:wat::core::if
                   (:wat::core::= fin-pending "open")
                   
                   (:wat::core::macro-error
                     "format: trailing lone '{' — use '{{' for a literal brace")
                   (:wat::core::if
                     (:wat::core::= fin-pending "close")
                     
                     (:wat::core::macro-error
                       "format: trailing lone '}' — use '}}' for a literal brace")
                     (:wat::core::if
                       (:wat::core::= fin-mode "name")
                       
                       (:wat::core::macro-error
                         (:wat::core::string::concat
                           "format: unclosed placeholder {"
                           fin-buf))
                       nil)))

     ;; Flush final text segment if non-empty.
     segments    (:wat::core::if
                   (:wat::core::String/empty? fin-buf)
                   
                   fin-segs
                   (:wat::core::conj fin-segs (:wat::core::Tuple "text" fin-buf)))

     ;; ── 4. Pass 2 — segments → pieces (Vector :- [WatAST]) + used-set ───
     ;; Helper: build a WatAST String-literal node from a text string.
     ;; (Option/expect (first (ast->children (read-string (concat "\"" text "\"")))))
     ;; The `"` guard above guarantees text never contains `"`, so the re-wrap is safe.

     pass2-result (:wat::core::foldl
                    (:wat::core::fn [acc2 <- :wat::core::Tuple
                                     seg  <- :wat::core::Tuple]
                      -> :wat::core::Tuple
                      (:wat::core::let
                        [ps2   (:wat::core::first acc2)
                         used2 (:wat::core::second acc2)
                         kind  (:wat::core::first seg)
                         pay   (:wat::core::second seg)]
                        (:wat::core::if
                          (:wat::core::= kind "text")
                          
                          ;; text segment → String literal AST node
                          (:wat::core::Tuple
                            (:wat::core::conj ps2
                              (:wat::core::first
                                (:wat::core::ast->children
                                  (:wat::core::match (:wat::core::read-string
                                    (:wat::core::string::concat
                                      "\""
                                      (:wat::core::string::concat pay "\"")))
                                    ((:wat::core::ReadOutcome::Forms __forms) __forms)
                                    ;; EXPAND-TIME site — hand-written, not the codemod's uniform
                                    ;; arm. `assertion-failed!` is a kernel head that DIVERGES, so
                                    ;; the F5 default-deny gate refuses it inside a program-body
                                    ;; macro, and rightly: expand-time failures belong on the
                                    ;; macro-error channel (EvalBreak::Diagnostic), not the panic
                                    ;; one. Blessing assertion-failed! to make a codemod's output
                                    ;; fit would be widening the gate to suit the tool.
                                    ((:wat::core::ReadOutcome::Malformed __cause)
                                      (:wat::core::macro-error
                                        (:wat::core::string::concat
                                          "string::interpolate: text segment did not parse: "
                                          (:wat::core::Error/message __cause))))))))
                            used2)
                          ;; slot segment → validate kwarg, emit (:wat::core::str val-ast)
                          (:wat::core::let
                            [_vn     (:wat::core::if
                                       (:wat::core::HashMap/contains-key? kwargs-map pay)
                                       
                                       nil
                                       (:wat::core::macro-error
                                         (:wat::core::string::concat
                                           "format: placeholder {"
                                           (:wat::core::string::concat pay
                                             "} has no matching kwarg"))))
                             val-ast (:wat::core::Option/expect  
                                        (:wat::core::HashMap/get kwargs-map pay)
                                        "format: internal — kwargs-map get post-contains?")]
                            (:wat::core::Tuple
                              (:wat::core::conj ps2 `(:wat::core::str ~val-ast))
                              (:wat::core::HashMap/assoc used2 pay true))))))
                    (:wat::core::Tuple
                      (:wat::core::Vector :wat::WatAST)
                      (:wat::core::HashMap :wat::core::String :wat::core::bool))
                    segments)

     pieces      (:wat::core::first pass2-result)
     used-set    (:wat::core::second pass2-result)

     ;; ── 5. Strict check: every kwarg must be consumed ───────────────
     kwarg-keys  (:wat::core::HashMap/keys kwargs-map)
     _unused-chk (:wat::core::foldl
                   (:wat::core::fn [_ <- :wat::core::nil key <- :wat::core::String]
                     -> :wat::core::nil
                     (:wat::core::if
                       (:wat::core::HashMap/contains-key? used-set key)
                       
                       nil
                       (:wat::core::macro-error
                         (:wat::core::string::concat "format: kwarg :"
                           (:wat::core::string::concat key
                             (:wat::core::string::concat " is unused — no {"
                               (:wat::core::string::concat key "} in template")))))))
                   nil
                   kwarg-keys)]

    ;; ── 6. Emit (:wat::core::string::concat piece …) ─────────────────
    ;; Empty template → "". Single piece → unwrap. Multiple → concat.
    (:wat::core::if
      (:wat::core::empty? pieces)
      
      `""
      (:wat::core::if
        (:wat::core::= (:wat::core::length pieces) 1)
        
        (:wat::core::first pieces)
        `(:wat::core::string::concat ~@pieces)))))

;; ─── Arc 293.2-parity: defstruct as a thin macro over structtype ──────────────
;;
;; Mirror: :wat::core::Record::def (macro) → :wat::core::recordtype (Rust primitive).
;;         :wat::core::defstruct (macro) → :wat::core::structtype (Rust primitive).
;;
;; :wat::core::defstruct is now a macro that splices ALL its args straight through
;; to :wat::core::structtype (name + optional metadata-map + field-vector — same
;; 2-arg or 3-arg shape). No code-gen here — struct method synthesis stays in
;; register_struct_methods (Rust, unchanged). The sole win: defstruct is now a
;; macro, enabling a uniform /from-map companion macro (like defrecord) in a
;; later arc.
;;
;; Load-order note: structtype is a Rust type-registration head, always known
;; before any macro expansion runs. No ordering gap.
;;
;; Arc 294 item 9a — CONSTRUCTION ERGONOMICS FLIP (same shape/rationale as
;; `:wat::core::defrecord` / `:wat::holon::defrecord` in wat/Record.wat — see there for the
;; full comment). `args` is `[name-kw, meta-map?, fields-vec]` (2 or 3 items, per
;; `parse_structtype`/`parse_aggregate`); the field-vector is always the LAST arg. Same
;; splice-field known gap as the record macros (this extraction runs at defstruct's own
;; macro-expansion time, before `~@:Surface` splices resolve at type-registration).
(:wat::core::defmacro :wat::core::defstruct
  [& args <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    [fqdn         (:wat::core::first args)
     fields       (:wat::core::Option/expect (:wat::core::last args) "defstruct: missing field-vector")
     field-ch     (:wat::core::ast->children fields)
     field-len    (:wat::core::length field-ch)
     n-fields     (:wat::core::i64::/ field-len 3)
     fname-nodes  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                      (:wat::core::conj acc
                        (:wat::core::Option/expect
                          (:wat::core::get field-ch (:wat::core::i64::* i 3))
                          "defstruct kwargs companion: fname index")))
                    (:wat::core::Vector :wat::WatAST)
                    (:wat::core::range 0 n-fields))
     field-names-ast-vec (:wat::core::with-children fields fname-nodes)
     fqdn-str      (:wat::core::keyword/to-string fqdn)
     ;; Arc 294 item 9a — a GENERIC type name (`:ns::T<A,B>`) registers its kwargs
     ;; companion macro + references its positional prime under the BARE name
     ;; (`:ns::T` / `:ns::T'`), matching register_aggregate_methods (runtime.rs:
     ;; `format!("{}'", agg.name)`, params dropped). The `<…>` rides ONLY on the
     ;; structtype registration (`~@args` below), which carries the params through.
     fqdn-bare-str (:wat::core::first (:wat::core::string::split fqdn-str "<"))
     fqdn-bare-kw  (:wat::core::keyword-node (:wat::core::string::interpolate ":{fqdn-bare-str}" :fqdn-bare-str fqdn-bare-str))
     ;; Arc 294 item (C) — the bare `:T` keyword STRING for the live `kwargs-construct`.
     bare-kw-str   (:wat::core::string::interpolate ":{fqdn-bare-str}" :fqdn-bare-str fqdn-bare-str)
     prime-kw-str  (:wat::core::string::concat ":" (:wat::core::string::concat fqdn-bare-str "'"))
     ns-parts      (:wat::core::string::split fqdn-bare-str "::")
     n-ns-parts    (:wat::core::length ns-parts)
     ns-lead       (:wat::core::foldl
                     (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
                       (:wat::core::conj acc
                         (:wat::core::Option/expect (:wat::core::get ns-parts i) "defstruct kwargs companion: ns-part index")))
                     (:wat::core::Vector :wat::core::String)
                     (:wat::core::range 0 (:wat::core::i64::- n-ns-parts 1)))
     ns-joined     (:wat::core::string::join "::" ns-lead)
     ns-colon-str  (:wat::core::string::concat ":" (:wat::core::string::concat ns-joined "::"))
     call-args-sym (:wat::core::symbol-node "call-args")]
    `(:wat::core::do
       (:wat::core::structtype ~@args)
       (:wat::core::defmacro ~fqdn-bare-kw
         [& ~call-args-sym <- (:wat::core::Vector :- [:wat::WatAST])]
         -> :wat::WatAST
         ;; Arc 294 item (C) — LIVE `kwargs-construct` over the bare `:T` (see Record.wat's BASE macro).
         (:wat::core::let
           [~(:wat::core::symbol-node "_kc-type") (:wat::core::keyword-node ~bare-kw-str)]
           `(:wat::core::kwargs-construct ~_kc-type ~@call-args))))))

;; ─── Arc 293 K5: extend-surface — default method impls over both pair tiers ────
;;
;; Takes a surface keyword :S plus N typeless method forms (m [binders] body) and emits
;; one extend-type per PAIR backing tier ($core-record and $holon-record). The user writes
;; BODY ONLY — extend-type already fills the method's types from the surface (the
;; 293.4e-pre.iii capability), so the macro needs no reflection seam. Pure form-production.
;;
;; Per the K5 decision (option A, 2026-06-30): the default rides BOTH pair tiers, so a
;; to-record'd value at either tier inherits it for free.
(:wat::core::defmacro :wat::core::extend-surface
  [surf <- :wat::WatAST  & methods <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    [surf-str   (:wat::core::keyword/to-string surf)            ;; "k5::HasX" (no leading colon)
     core-kw    (:wat::core::keyword/from-string
                  (:wat::core::string::interpolate "{surf-str}$core-record" :surf-str surf-str))
     holon-kw   (:wat::core::keyword/from-string
                  (:wat::core::string::interpolate "{surf-str}$holon-record" :surf-str surf-str))]
    `(:wat::core::do
       (:wat::core::extend-type ~core-kw  ~surf ~@methods)
       (:wat::core::extend-type ~holon-kw ~surf ~@methods))))

;; ─── Arc 296: :wat::kernel::Location — moving the source of truth to wat ──
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration; the Rust side is meant to
;; become generated FROM this form rather than hand-maintained alongside it.
;;
;; A point in a source file: populated by `:wat::kernel::run-sandboxed` when
;; a panic carries a PanicInfo location, and by assertion primitives whose
;; failure-payload needs to cite file:line:col.
;;
;; Placed here, near the top of core.wat and before :wat::core::Error below:
;; the :wat::core::Error surface's `location` feature is typed
;; :wat::kernel::Location, so core.wat genuinely depends on this type — that
;; measured dependency edge is why Location lives here rather than alongside
;; its seven kernel-diagnostics siblings in wat/kernel/diagnostics.wat.
(:wat::core::defrecord :wat::kernel::Location
  [file <- :wat::core::String
   line <- :wat::core::i64
   col  <- :wat::core::i64])

;; ─── Arc 296 S3: :wat::core::Error stdlib surface ────────────────────────────
;;
;; The canonical contract for error records: a message (human-readable String),
;; a source location (kernel::Location — the call site or origin), and a
;; recursive causes chain ((Vector :- [Error]) — zero or more contributing errors).
;;
;; :nature :wat::core::Record — pure; the surface and its backing records may
;; appear as field types in other pure aggregates (defrecord, defsurface)
;; without ImpureFieldInPureAggregate.
;;
;; Recursive self-reference in `causes` is unblocked by arc 296 S1 (is_pure_type
;; for Record-natured surfaces) + S2 (infer_list_constructor surface path).
;;
;; Load-order: :wat::core::String and :wat::core::Vector are available at the
;; top of this file; :wat::kernel::Location is a Rust builtin registered before
;; any stdlib wat loads — all three dependencies are satisfied here.
(:wat::core::defsurface :wat::core::Error
  :nature :wat::core::Record
  :features [message  <- :wat::core::String
             location <- :wat::kernel::Location
             causes   <- (:wat::core::Vector :- [:wat::core::Error])])

;; ─── Arc 296 S3: :wat::core::Fault — canonical minimal error record ──────────
;;
;; The simplest concrete error: a human-readable message string, the call-site
;; location, and an empty causes chain. Structurally satisfies :wat::core::Error
;; (all three floor fields — message, location, causes are present and typed
;; identically) so it may be passed to any [e <- :wat::core::Error] param.
;;
;; Smart constructor: :wat::core::Fault/of captures the CALL SITE location via
;; (:wat::kernel::here) spliced into the expansion — it is a MACRO (not a fn)
;; precisely so the (here) form fires at the caller's source coordinate, not at
;; the constructor's own location.
(:wat::core::defrecord :wat::core::Fault
  [message  <- :wat::core::String
   location <- :wat::kernel::Location
   causes   <- (:wat::core::Vector :- [:wat::core::Error])])

(:wat::core::defmacro :wat::core::Fault/of
  [msg <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::Fault :message ~msg :location (:wat::kernel::here) :causes (:wat::core::Vector :wat::core::Error)))

;; ─── Arc 296: :wat::core::EvalError — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration; the Rust side is meant to
;; become generated FROM this form rather than hand-maintained alongside it.
;;
;; Populated in the Err slot of a :Result returned by the eval-family forms
;; (:wat::eval-ast! / eval-edn! / eval-digest! / eval-signed!) when dynamic
;; evaluation fails. Carries a `kind` discriminator (short machine-readable
;; variant name, e.g. "verification-failed", "parse-failed", "type-mismatch")
;; and a `message` diagnostic (human-readable detail).
(:wat::core::defstruct :wat::core::EvalError
  [kind    <- :wat::core::String
   message <- :wat::core::String])

;; ─── Arc 296: :wat::core::Span — moving the source of truth to wat ────────
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; The leaf source location an error's `:location` floor key carries (arc 278
;; "errors first-class EDN"). `end` is `(Option :- [Pos])` — `None` for point-spans.
(:wat::core::defrecord :wat::core::Span
  [file <- :wat::core::String
   line <- :wat::core::i64
   col  <- :wat::core::i64
   end  <- (:wat::core::Option :- [:wat::core::Pos])])
