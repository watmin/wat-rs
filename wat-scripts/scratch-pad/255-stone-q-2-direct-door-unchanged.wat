;; Scratch probe — arc 255 Stone Q-2 acceptance row 3: THE AST DOOR IS UNTOUCHED.
;;
;; Sibling of `255-stone-q-2-the-threaded-span-must-be-used.wat`, which drives the same seven
;; arithmetic families through `:wat::core::apply` (the VALUE door — Q-2's target). This probe
;; drives the identical seven families through a DIRECT call — `(:wat::i64::+ 1 "x")`-shaped,
;; not `apply` — via `:wat::eval-ast!` so the type checker cannot refuse it statically and the
;; RUNTIME arm actually fires.
;;
;; The direct door calls `eval_i64_arith`/`i64_add_op` (and the f64/bigint/rational
;; equivalents) — NOT `arith_i64_i64_inner`/`arith_f64_f64_inner`/`arith_bigint_bigint_inner`/
;; `arith_rational_rational_inner`, which Q-2 touched. STOP-3 in `BRIEF-STONE-Q-2` names this
;; as the thing that must stay byte-identical: "Only the value door was span-less."
;;
;; MEASURED: this probe's stdout is BYTE-IDENTICAL between HEAD (git clone before Stone Q) and
;; the Stone Q-2 tree — `diff` on the two captures is empty. See BRIEF-STONE-Q-2's row 3 for the
;; transcript. Re-run it as the acceptance instrument; do not rewrite it.

(:wat::core::defn :probe::show
  [tag <- :wat::core::String r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat tag ": " (:wat::edn::write r))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_1 (:probe::show "i64-type"
          (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 1 "x"))))
     _2 (:probe::show "i64-overflow"
          (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 9223372036854775807 1))))
     _3 (:probe::show "div-zero"
          (:wat::eval-ast! (:wat::core::quote (:wat::i64::/ 5 0))))
     _4 (:probe::show "f64-type"
          (:wat::eval-ast! (:wat::core::quote (:wat::f64::+ 1.0 "x"))))
     _5 (:probe::show "rational-type"
          (:wat::eval-ast! (:wat::core::quote (:wat::rational::+ (:wat::i64::to-rational 1) "x"))))
     _6 (:probe::show "bigint-type"
          (:wat::eval-ast! (:wat::core::quote (:wat::bigint::+ (:wat::i64::to-bigint 1) "x"))))
     _7 (:probe::show "arity"
          (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 1))))]
    nil))
