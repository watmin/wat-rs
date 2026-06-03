;; wat/core.wat — :wat::core::* stdlib forms.
;;
;; Originally used arc 146's `:wat::core::define-dispatch` (slice 1) to route
;; polymorphic-name primitives to per-Type impls. All define-dispatch decls
;; were evacuated to Rust ∀T intrinsics via arc 237 Stones 237.7a/7b/7c/7d/8a.
;; Stone 241.13 retired `:wat::core::define-dispatch` entirely (HARD CUT);
;; `:wat::core::defclause` (Stone 237.2) is the surviving dispatch entity kind.
;;
;; Loads BEFORE `wat/runtime.wat` so aliases and variadic fns are
;; visible to any reflection-driven macro that might reference them.

;; Arc 237 Stone 237.7a — :wat::core::length evacuated to Rust ∀T intrinsic (src/check.rs +
;; src/runtime.rs). define-dispatch decl removed; the per-type leaves (:Vector/length,
;; :HashMap/length, :HashSet/length) and the DispatchRegistry remain for other ops.

;; Arc 146 slice 3 — bundled migration: empty? / contains? / get / conj.
;; Same shape as length above. contains? uses MIXED IMPL VERBS:
;; HashMap tests KEY membership (`contains-key?`); Vector + HashSet
;; test ELEMENT membership (`contains?`). Caller writes
;; `(:contains? c x)` regardless; dispatch picks the arm by container
;; shape and the impl's verb is internal.
;;
;; get's per-arm return type varies (Vec returns :Option<T>; HashMap
;; returns :Option<V>); infer_dispatch_call returns the matched arm's
;; specific Option<_> type per arc 146 DESIGN.
;;
;; conj is 2-arm only (Vector / HashSet); HashMap doesn't conj —
;; HashMap requires key+value pairing, so :wat::core::assoc is the
;; right verb there (DESIGN audit table).

;; Arc 237 Stone 237.7b-i — :wat::core::empty? evacuated to Rust ∀T intrinsic (src/check.rs +
;; src/runtime.rs). define-dispatch decl removed; the per-type leaves (:Vector/empty?,
;; :HashMap/empty?, :HashSet/empty?) and the DispatchRegistry remain for other ops.

;; arc 237 Stone 237.7b-ii — :wat::core::contains? is now a Rust ∀T intrinsic with custom inference arm; see src/check.rs::infer_contains + src/runtime.rs::eval_contains

;; arc 237 Stone 237.7b-iv — `:wat::core::get` is now a Rust ∀T intrinsic with custom inference arm; see `src/check.rs::infer_get` + `src/runtime.rs::eval_get`

;; arc 237 Stone 237.7b-iii — `:wat::core::conj` is now a Rust ∀T intrinsic with custom inference arm; see `src/check.rs::infer_conj` + `src/runtime.rs::eval_conj`

;; Arc 146 slice 4 — :wat::core::* short-name aliases for single-impl
;; ops. Each alias maps a short ergonomic name to its explicit per-Type
;; impl. Per arc 146 DESIGN: single-impl ops are aliases (not
;; dispatches; dispatch is for genuine polymorphism). Both short + long
;; names work; both are honest.
;;
;; Stone 241.12 — migrated from :wat::runtime::define-alias to :wat::core::defalias
;; (native substrate form; one layer, not two).

;; arc 237 Stone 237.7c — `:wat::core::assoc` is now a Rust ∀T intrinsic with custom inference
;; arm spanning HashMap + Record; see `src/check.rs::infer_assoc` + `src/runtime.rs::eval_assoc`.
(:wat::core::defalias :wat::core::dissoc  :wat::core::HashMap/dissoc)
(:wat::core::defalias :wat::core::keys    :wat::core::HashMap/keys)
(:wat::core::defalias :wat::core::values  :wat::core::HashMap/values)
(:wat::core::defalias :wat::core::concat  :wat::core::Vector/concat)

;; ─── Arc 148 slice 4 / Stone 237.8b — Numeric arithmetic (recipe-lock) ──────
;;
;; Stone 237.8b — THE RECIPE locked:
;;
;;   Layer 1 (Rust): per-Type binary primitive — :wat::core::<Type>::<op>
;;                   ALWAYS 2-ary; irreducible; one fn per Type per op.
;;                   '2 suffix DROPPED (Stone 237.8b HARD CUT).
;;
;;   Layer 2 (wat):  polymorphic defclause — :wat::core::<op>
;;                   Clauses dispatch by arity (0/1/2/3+) × arg-Type.
;;                   Per-op identity defaults via Lisp tradition.
;;                   Variadic via 3+-ary clause with & rest-binder folding
;;                   the per-Type binary primitive over rest.
;;
;; Per-Type variadic wat fns (:wat::core::i64::+, :wat::core::f64::+ etc.)
;; DELETED (Stone 237.8b HARD CUT) — absorbed by defclause clauses.
;; infer_arithmetic + eval_arithmetic_variadic + is_numeric DELETED from Rust.
;;
;; Cross-type rejection via CLAUSE ABSENCE — no mixed-type clause exists,
;; so (:wat::core::+ 1 2.0) → :NoMatchingClause (enforced by defclause
;; first-match semantics). No special-case Rust check needed.
;;
;; Lisp/Clojure arity rules (per Stone 237.8b DESIGN):
;;   `+`/`*`: 0-ary → identity (0 / 1); 1-ary → arg unchanged; 2-ary → binary; 3+ → fold
;;   `-`/`/`: 0-ary → :NoMatchingClause (no clause); 1-ary → identity-on-left
;;            (negate / reciprocal); 2-ary → binary; 3+ → fold

;; ─── Polymorphic arithmetic defclauses ────────────────────────────────────────

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

(:wat::core::defclause :wat::core::-
  ;; NO 0-ary clause — :NoMatchingClause fires via 237.4 rich error
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
    (:wat::core::foldl rest (:wat::core::i64::- x y)
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::- acc n))))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl rest (:wat::core::f64::- x y)
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::- acc n)))))

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
    (:wat::core::foldl rest (:wat::core::i64::* x y)
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::* acc n))))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl rest (:wat::core::f64::* x y)
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::* acc n)))))

(:wat::core::defclause :wat::core::/
  ;; NO 0-ary clause — :NoMatchingClause fires via 237.4 rich error
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
    (:wat::core::foldl rest (:wat::core::i64::/ x y)
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::/ acc n))))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl rest (:wat::core::f64::/ x y)
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::/ acc n)))))

;; ─── Named-function binding ───────────────────────────────────────
;;
;; `:wat::core::defn` is the user-facing named-function form. It
;; composes the two foundational primitives:
;;
;;   (:wat::core::defn :name :sig :body)
;;     ↓ macro-expansion
;;   (:wat::core::def :name (:wat::core::fn :sig :body))
;;
;; Per user direction 2026-05-08: `:wat::core::fn` is the ONE AND
;; ONLY function constructor. defn just binds the function value to
;; a name; def just binds any value to a name. Composition over
;; multiplication of primitives.
;;
;; Inherits from `:wat::core::def`:
;; - position rule (top-level OR direct child of top-level do/let body)
;; - strict-default redef-error
;; - recursive name binding (the fn body sees `:name` as bound)
;;
;; Multi-arity overloads are NOT in this form's scope; a separate
;; `defn-clause` form (Erlang-style) ships later.
;;
;; Docstrings are NOT in this form's scope; arc 141 wires docstring
;; extraction broadly across substrate forms; defn extends to take a
;; docstring at that time.

;; Arc 167 slice 2 — flat-shape signature. defn forwards args/arrow/
;; ret/body to fn unchanged via rest-binder splicing. The new fn shape
;; is 5 elements at the form level: head + ARGS-VECTOR + `->` + :RET +
;; BODY. defn takes a name keyword + the same 4 trailing pieces:
;;
;;   (:wat::core::defn :name [p <- :T q <- :T] -> :Ret body)
;;     ↓ macro-expansion
;;   (:wat::core::def :name (:wat::core::fn [p <- :T q <- :T] -> :Ret body))
;;
;; Stone 241.6 — optional metadata-map between name and argspec threads
;; through rest-binder unchanged:
;;
;;   (:wat::core::defn :name {:doc "..."} [p <- :T] -> :Ret body)
;;     ↓ macro-expansion (rest-binder unchanged)
;;   (:wat::core::def :name (:wat::core::fn {:doc "..."} [p <- :T] -> :Ret body))
;;     ↓ substrate fn-embedded-metadata peel (try_parse_fn_shape_def + eval_fn)
;;   binding_metadata[":name"] = {:doc "..."}; fn sees [p <- :T] -> :Ret body
;;
;; The quasiquote-only defmacro body cannot branch on metadata presence;
;; the substrate's fn-peel extracts binding-level metadata from the fn-form
;; transparently. The macro template is UNCHANGED.
;;
;; The rest-binder uses the variadic-defmacro shape per arc 150 / per
;; `wat/test.wat` § :wat::test::program (`:AST<wat::core::Vector<wat::WatAST>>`).
(:wat::core::defmacro :wat::core::defn
  [name <- :AST<wat::core::nil>
   & rest <- :AST<wat::core::Vector<wat::WatAST>>]
  -> :AST<wat::core::nil>
  `(:wat::core::def ~name (:wat::core::fn ~@rest)))

;; Arc 198 defined `:wat::core::defn-restricted` as mechanical sugar over
;; `:wat::core::def-restricted` — both forms retired by Stone 241.14.
;; Restrictions now live as `:restricted-to` key in metadata-map on def/defn:
;;
;;   (:wat::core::defn :name {:restricted-to [<prefix-kw>...]}
;;     [p <- :T q <- :T] -> :Ret body)
;;
;; The HARD-CUT arm at check.rs fires for any residual caller of either retired form.

;; ─── Polymorphic ordering defclauses (Stone 237.8b) ──────────────────────────
;;
;; 2-ary only: no variadic ordering this stone. Each clause calls the
;; per-Type ordering primitive (i64 routes through eval_compare; f64 routes
;; through NaN-correct eval_f64_compare). Cross-type → :NoMatchingClause
;; via clause absence.

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
