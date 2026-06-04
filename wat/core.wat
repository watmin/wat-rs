;; vigilatum: 2026-06-04T04:01:56Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(core-arithmetic)
;;
;; wat/core.wat — the :wat::core::* stdlib surface: short-name aliases plus the
;; polymorphic arithmetic and ordering defclauses.
;;
;; Loads early in the stdlib so these forms are visible to the later
;; files that reference them.

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
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc n))
      (:wat::core::i64::+ x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::+ acc n))
      (:wat::core::f64::+ x y)
      rest)))

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
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::- acc n))
      (:wat::core::i64::- x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::- acc n))
      (:wat::core::f64::- x y)
      rest)))

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
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::* acc n))
      (:wat::core::i64::* x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::* acc n))
      (:wat::core::f64::* x y)
      rest)))

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
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::/ acc n))
      (:wat::core::i64::/ x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::/ acc n))
      (:wat::core::f64::/ x y)
      rest)))

;; ─── Named-function binding ───────────────────────────────────────
;;
;; defn just binds a function value to a name: it macro-expands to
;; (:wat::core::def :name (:wat::core::fn …)). :wat::core::fn is the one and
;; only function constructor; defn forwards the argspec/arrow/ret/body to it
;; unchanged via rest-binder splicing, and an optional metadata-map threads
;; through too — the substrate peels binding-level metadata from the fn-form,
;; so the macro template stays metadata-blind and UNCHANGED.
(:wat::core::defmacro :wat::core::defn
  [name <- :AST<wat::core::nil>
   & rest <- :AST<wat::core::Vector<wat::WatAST>>]
  -> :AST<wat::core::nil>
  `(:wat::core::def ~name (:wat::core::fn ~@rest)))

;; Restrictions live as a :restricted-to key in the metadata-map on def/defn
;; (e.g. {:restricted-to [<prefix-kw>…]}); the substrate enforces it.

;; ─── Polymorphic ordering defclauses ──────────────────────────────
;;
;; 2-ary per-Type (i64 / f64), NaN-correct for f64; cross-type rejected by
;; clause absence, same as arithmetic.

(:wat::core::defclause :wat::core::<
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::< x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::bool (:wat::core::f64::< x y)))

(:wat::core::defclause :wat::core::>
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::bool (:wat::core::f64::> x y)))

(:wat::core::defclause :wat::core::<=
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::<= x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::bool (:wat::core::f64::<= x y)))

(:wat::core::defclause :wat::core::>=
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::>= x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::bool (:wat::core::f64::>= x y)))
