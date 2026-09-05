;; PROBE — STONE `:wat::eval::walk` faces `:wat::WatAST` (arc 255).
;;
;; The DESIGN/BRIEF's own risk: `holon_to_watast` is a conversion, and a conversion can lose.
;; The pre-existing probe's terminal form was the scalar `5`, which cannot detect loss. This
;; probe's terminal is a COMPOSITION — a bare-list Bundle of two `leaf` holon-constructor calls
;; — recognized by `try_recognize_holon_value` (`src/runtime.rs`) as an already-value shape, so
;; `walk` reaches its terminal in exactly ONE visit (AlreadyTerminal, acc == 1) with ZERO
;; reduction in between.
;;
;; MEASURED, before this stone's fix:
;;   - `(:wat::core::ast->source (:wat::core::first pair))` directly on `walk`'s own terminal did
;;     NOT type-check: "ast->source: parameter #1 expects :wat::WatAST; got :wat::holon::HolonAST"
;;     — the asymmetry the whole stone exists to close, surfaced as a compile error rather than
;;     papered over.
;;   - Calling `:wat::holon::to-wat` (the wat-facing wrapper of the SAME `holon_to_watast` this
;;     stone wires into `walk`'s construction site) manually on the terminal DID type-check under
;;     the old scheme (its param IS `:wat::holon::HolonAST`), and rendered
;;     `("k" "v")` — a 2-element list, both leaves present, nothing dropped or truncated. The
;;     original quoted source read `((:wat::holon::leaf "k") (:wat::holon::leaf "v"))`; the two
;;     renderings differ in SYNTAX (the `leaf` constructor calls are recognized as their own
;;     already-classified values and rendered as the bare literals they denote — an evaluation
;;     normalization, not a loss) while agreeing exactly in STRUCTURE (a 2-element composition,
;;     "k" then "v", in order).
;;
;; MEASURED, after the fix: `ast->source` now works DIRECTLY on `(first pair)` with no manual
;; `to-wat` needed (the manual call now itself becomes a type error the other way — "expects
;; :wat::holon::HolonAST; got :wat::WatAST" — proof the signature genuinely changed) and renders
;; the IDENTICAL `("k" "v")` the manual conversion produced before the fix. Same conversion
;; function (STOP-3: `holon_to_watast`/`to-wat` untouched), now applied at the callee's own
;; construction site instead of by caller-side convention.

(:wat::core::defn :my::test::count-visit
  [acc <- :wat::core::i64 form <- :wat::WatAST step <- :wat::eval::StepResult]
  -> (:wat::eval::WalkStep :- [:wat::core::i64])
  (:wat::eval::WalkStep::Continue (:wat::i64::+ acc 1)))

(:wat::core::defn :my::main-roundtrip [] -> :wat::core::nil
  (:wat::core::let
    [source (:wat::core::quote
              ((:wat::holon::leaf "k")
               (:wat::holon::leaf "v")))]
    (:wat::core::match
      (:wat::eval::walk source 0 :my::test::count-visit)
      ((:wat::core::Ok pair)
        (:wat::core::let
          [before-src (:wat::core::ast->source source)
           after-src (:wat::core::ast->source (:wat::core::first pair))
           acc (:wat::core::second pair)]
          (:wat::kernel::println
            (:wat::core::Tuple acc before-src after-src (:wat::core::= after-src "(\"k\" \"v\")")))))
      ((:wat::core::Err e)
        (:wat::kernel::println e)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:my::main-roundtrip))
