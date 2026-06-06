;; vigilatum: 2026-06-06T04:56:04Z — UPDATED-vigilia spec/DSL 5-spell guard
;; L1+L2=0 (cernere [CONVERGED 0+0: full vocabulary table, every expand-time
;; head verified on the pure-total allow-list] + probare [all 12 forms (16 at first earn; 4 ordering defclauses retired by Stone 245.8)
;; Expressed] + conferre [all 17 header claims verified; 6 USER-GUIDE
;; divergences fixed spec-side] + exigere [CONVERGED 0+0] + circumspicere
;; LAST [the false loads-early rationale killed at both sites; empty-step
;; behaviors documented + witnessed at their empirical failure shapes]).
;; Witness corpus: deftest-green(core-arithmetic + core-equality +
;; core-threading + core-collection-aliases + option-expect + record-def +
;; result-expect + struct-to-form + list-fold-aliases); corpus 236/0/53;
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

;; ─── Threading macros `->` / `->>` ───────────────────────────────
;;
;; Thread-first `->`: inject acc as the FIRST arg of each step.
;;   (-> x (f a b) g)  =>  (g (f x a b))
;; A list step `(f a…)` => `(f acc a…)`; a bare symbol/keyword step `f` => `(f acc)`.
;; Empty-list step `()`: Option/expect on (first ()) fires "-> step has no head"
;;   as a panic_any(AssertionPayload) at macro-expansion time (during startup).
(:wat::core::defmacro :wat::core::->
  [acc <- :wat::holon::HolonAST & steps <- :AST<wat::holon::Holons>]
  -> :AST<wat::holon::HolonAST>
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) -> :AST<wat::holon::HolonAST>
          `(~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first step) "-> step has no head") ~a ~@(:wat::core::rest step))
          `(~step ~a)))
    acc
    steps))

;; Thread-last `->>`: inject acc as the LAST arg of each step.
;;   (->> x (f a b) g)  =>  (g (f a b x))
;; A list step `(f a…)` => `(f a… acc)`; a bare symbol/keyword step `f` => `(f acc)`.
;; Empty-list step `()`: ~@() splices nothing, yielding `(acc)` — expansion succeeds
;;   but eval rejects the integer-head form with MalformedForm at runtime.
(:wat::core::defmacro :wat::core::->>
  [acc <- :wat::holon::HolonAST & steps <- :AST<wat::holon::Holons>]
  -> :AST<wat::holon::HolonAST>
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) -> :AST<wat::holon::HolonAST>
          `(~@step ~a)
          `(~step ~a)))
    acc
    steps))

;; ─── keyword/of — parametric keyword construction ─────────────────
;;
;; keyword/of — build the parametric keyword `:Head<arg1,arg2>` from keyword args
;; (head + args, leading colons stripped). Pure-total program over forms.
;; Arc 249 Stone 249.4a — promoted from construct_keyword_of (expand.rs).
;; Zero args: string::join "" [] = "", yielding `:Head<>` (empty angle brackets).
(:wat::core::defmacro :wat::core::keyword/of
  [head <- :wat::holon::HolonAST & args <- :AST<wat::holon::Holons>]
  -> :AST<wat::holon::HolonAST>
  (:wat::core::let [head-text (:wat::core::keyword/to-string head)
                    arg-texts (:wat::core::map
                                (:wat::core::fn [a <- :wat::holon::HolonAST] -> :wat::core::String
                                   (:wat::core::keyword/to-string a))
                                args)
                    joined (:wat::core::string::join "," arg-texts)
                    full (:wat::core::string::concat head-text
                           (:wat::core::string::concat "<"
                             (:wat::core::string::concat joined ">")))]
    `~(:wat::core::keyword/from-string full)))

;; Stone 245.8 — Polymorphic ordering defclauses RETIRED.
;; `<`/`>`/`<=`/`>=` are now a relational check-side intrinsic (`infer_ordering`
;; in src/check.rs), the sibling of `infer_equality`. The runtime dispatch arms
;; in `dispatch_keyword_head_value` (src/runtime.rs) route directly to `eval_compare`.
;; The per-Type leaves (`:wat::core::i64::<`, `:wat::core::f64::<`, etc.) remain
;; as the type-locked tier in Rust.
