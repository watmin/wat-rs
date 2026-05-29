;; wat/core.wat — :wat::core::* dispatches.
;;
;; Substrate dispatches that route polymorphic-name primitives to
;; per-Type impls. Per arc 146 DESIGN: one entity-kind (dispatch) for
;; genuinely-polymorphic primitives; per-Type impls live in Rust as
;; clean rank-1 schemes registered in `register_builtins`.
;;
;; Each declaration uses arc 146's `:wat::core::define-dispatch`
;; (slice 1). Loads BEFORE `wat/runtime.wat` so the dispatches are
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
;; names work; both are honest. The alias machinery (arc 143's
;; :wat::runtime::define-alias) expands at registration time into a
;; delegating user-define whose head copies the target's signature
;; with the alias name substituted.

;; arc 237 Stone 237.7c — `:wat::core::assoc` is now a Rust ∀T intrinsic with custom inference
;; arm spanning HashMap + Record; see `src/check.rs::infer_assoc` + `src/runtime.rs::eval_assoc`.
(:wat::runtime::define-alias :wat::core::dissoc  :wat::core::HashMap/dissoc)
(:wat::runtime::define-alias :wat::core::keys    :wat::core::HashMap/keys)
(:wat::runtime::define-alias :wat::core::values  :wat::core::HashMap/values)
(:wat::runtime::define-alias :wat::core::concat  :wat::core::Vector/concat)

;; ─── Arc 148 slice 4 — Numeric arithmetic ────────────────────────────
;;
;; arc 237 Stone 237.8a — `:wat::core::<op>'2` define-dispatch decls
;; retired under THE DECISION (`feedback_no_implicit_coercion`);
;; same-type arithmetic routes directly through per-Type leaves
;; (`:wat::core::i64::+'2` / `:wat::core::f64::+'2`); cross-type is
;; rejected at check by `infer_arithmetic` (no longer f64-promoting).
;; Mixed-type Rust leaves (`+'i64'f64` etc.) deleted from substrate.
;;
;; Each of `+`, `-`, `*`, `/` remains a polymorphic surface at the
;; variadic level. Two layers remain per THE DECISION:
;;
;;   1. Polymorphic variadic at `:wat::core::<v>` (bare name) — STAYS
;;      as a substrate primitive with same-type-only discipline.
;;      Custom inference (`infer_arithmetic`) rejects mixed numeric
;;      pairs at check time; callers homogenize explicitly via
;;      `:wat::core::i64::to-f64` (or vice versa).
;;
;;   2. Per-Type Rust binary primitives at `:wat::core::<Type>::<v>'2`
;;      — registered in `register_builtins` (src/runtime.rs +
;;      src/check.rs). Reachable per the no-privacy doctrine.
;;      Mixed-type leaves (`+'i64'f64` etc.) DELETED.
;;
;; Same-type variadic wat fns at `:wat::core::<Type>::<v>` (the bare
;; per-Type name) wrap the per-Type binary leaf via arc 150's variadic
;; define + `:wat::core::foldl` — declared after this comment block.

;; ─── Same-type variadic wat fns (8 total) ─────────────────────────────
;;
;; Per-Type variadic wrappers using arc 150's variadic define syntax.
;; Each folds left over the per-Type binary leaf.
;;
;; Lisp/Clojure arity rules per DESIGN § "Arity rules":
;;   `+`/`*` — 0-ary returns identity; 1-ary returns arg unchanged
;;   `-`/`/` — 0-ary errors via 1-arity-min substrate enforcement;
;;             1-ary inserts identity-on-left (negation/reciprocal)
;;
;; The 0-ary case for `:i64::+`/`:i64::*` is expressed as the foldl
;; seed when the variadic surface receives zero rest args. For
;; `-`/`/`, the 0-ary case is enforced by requiring at least one
;; fixed parameter (the variadic accepts >= 1 arg via the (first
;; rest) convention — see DESIGN § "Variadic semantics").

;; i64 same-type variadic — :+/:*/:- / :/  fold over per-Type binary leaf.

(:wat::core::defn :wat::core::i64::+ [& xs <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
  (:wat::core::foldl xs 0
      (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+'2 acc x))))

(:wat::core::defn :wat::core::i64::* [& xs <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
  (:wat::core::foldl xs 1
      (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::*'2 acc x))))

;; `:-` and `:/` require >= 1 arg. Express via fixed first param +
;; rest. 1-ary inserts identity-on-left; 2+-ary folds. The arity
;; checker rejects 0-ary via the fixed-param requirement.

(:wat::core::defn :wat::core::i64::- [first <- :wat::core::i64 & xs <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::i64
      (:wat::core::i64::-'2 0 first)
      (:wat::core::foldl xs first
        (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::-'2 acc x)))))

(:wat::core::defn :wat::core::i64::/ [first <- :wat::core::i64 & xs <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::i64
      (:wat::core::i64::/'2 1 first)
      (:wat::core::foldl xs first
        (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::/'2 acc x)))))

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
(:wat::core::defmacro
  (:wat::core::defn
    (name :AST<wat::core::nil>)
    & (rest :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def ~name (:wat::core::fn ~@rest)))

;; Arc 198 — `:wat::core::defn-restricted` is the named-fn counterpart of
;; `:wat::core::def-restricted`. Same shape as `defn` plus a `:restricted-to`
;; keyword tag + prefix-vec between the name and the fn signature:
;;
;;   (:wat::core::defn-restricted :name :restricted-to [<prefix-kw>...]
;;     [p <- :T q <- :T] -> :Ret body)
;;     ↓ macro-expansion
;;   (:wat::core::def-restricted :name :restricted-to [<prefix-kw>...]
;;     (:wat::core::fn [p <- :T q <- :T] -> :Ret body))
;;
;; Mechanical sugar. The whitelist is a property of the BINDING (not the
;; fn shape); restriction lives on `def-restricted`, the substrate primitive.
;; The `restricted-to-keyword` binder is spliced through as-is; the substrate
;; primitive's parser validates that it is the literal `:restricted-to` keyword.
;; Same rest-binder shape as `defn` per arc 150 variadic-defmacro form.
(:wat::core::defmacro
  (:wat::core::defn-restricted
    (name :AST<wat::core::nil>)
    (restricted-to-keyword :AST<wat::core::nil>)
    (prefixes :AST<wat::core::nil>)
    & (rest :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def-restricted ~name ~restricted-to-keyword ~prefixes (:wat::core::fn ~@rest)))

;; f64 same-type variadic — :+/:*/:- / :/

(:wat::core::defn :wat::core::f64::+ [& xs <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
  (:wat::core::foldl xs 0.0
      (:wat::core::fn [acc <- :wat::core::f64 x <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::+'2 acc x))))

(:wat::core::defn :wat::core::f64::* [& xs <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
  (:wat::core::foldl xs 1.0
      (:wat::core::fn [acc <- :wat::core::f64 x <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::*'2 acc x))))

(:wat::core::defn :wat::core::f64::- [first <- :wat::core::f64 & xs <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::f64
      (:wat::core::f64::-'2 0.0 first)
      (:wat::core::foldl xs first
        (:wat::core::fn [acc <- :wat::core::f64 x <- :wat::core::f64] -> :wat::core::f64
          (:wat::core::f64::-'2 acc x)))))

(:wat::core::defn :wat::core::f64::/ [first <- :wat::core::f64 & xs <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::f64
      (:wat::core::f64::/'2 1.0 first)
      (:wat::core::foldl xs first
        (:wat::core::fn [acc <- :wat::core::f64 x <- :wat::core::f64] -> :wat::core::f64
          (:wat::core::f64::/'2 acc x)))))
