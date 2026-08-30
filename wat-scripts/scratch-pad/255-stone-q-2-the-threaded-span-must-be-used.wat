;; Scratch probe — arc 255 Stone Q-2 acceptance rows 1 and 2.
;;
;; ⚠ THIS PROGRAM IS EXPECTED TO DIE for every real `<case>` argument. That death IS the
;; finding: each case exercises one arithmetic value-door family through `:wat::core::apply`,
;; and the uncaught `RuntimeError` the process prints carries the diagnostic's `:location`.
;;
;; Run with `./target/release/wat <this file> <case>`. Cases:
;;   i64-type        — arith_i64_i64_inner      TypeMismatch
;;   i64-overflow     — arith_i64_i64_inner      IntegerOverflow
;;   div-zero          — arith_i64_i64_inner      DivisionByZero
;;   f64-type          — arith_f64_f64_inner      TypeMismatch
;;   rational-type     — arith_rational_rational_inner TypeMismatch
;;   bigint-type       — arith_bigint_bigint_inner TypeMismatch
;;   arity             — dispatch_substrate_impl   ArityMismatch
;;   span-a / span-b   — the SAME i64-type error, at two different call-site lines below,
;;                       to show the reported `:location` differs (row 2 — "still synthesized"
;;                       would report the SAME location for both).
;; Any other argument (or none) prints "unknown case" and exits cleanly — `--check` never
;; runs `main`, so this selector never affects the `every_wat_scripts_file_loads` gate.
;;
;; MEASURED at the Stone Q-2 tree (this stone), `./target/release/wat`:
;;   i64-type      -> err TypeMismatch    at THIS FILE's apply call site (line:col below)
;;   i64-overflow  -> err IntegerOverflow at THIS FILE's apply call site
;;   div-zero      -> err DivisionByZero  at THIS FILE's apply call site
;;   f64-type      -> err TypeMismatch    at THIS FILE's apply call site
;;   rational-type -> err TypeMismatch    at THIS FILE's apply call site
;;   bigint-type   -> err TypeMismatch    at THIS FILE's apply call site
;;   arity         -> err ArityMismatch   at THIS FILE's apply call site
;;   span-a vs span-b -> same TypeMismatch KIND, DIFFERENT reported line:col
;;
;; MEASURED at HEAD (git clone before Stone Q), the identical binary+file pair:
;;   every one of the seven families above instead reports a `src/runtime.rs:<line>:<col>`
;;   location — the `rust_caller_span!()` point Q-2 replaced. See BRIEF-STONE-Q-2's row 1
;;   for the full before/after transcript this probe produced.
;;
;; The direct (AST) door is untouched: the same seven families raised through a plain
;; `(:wat::i64::+ 1 "x")`-shaped direct call (via `:wat::eval-ast!`, not `apply`) are
;; byte-identical before and after this stone — see
;; `255-stone-q-2-direct-door-unchanged.wat`, this probe's sibling.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     case (:wat::core::Option/expect (:wat::core::get argv 2) "usage: <this file> <case>")]
    (:wat::core::cond
      ((:wat::core::= case "i64-type")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::i64::+ (:wat::core::Vector :- [:wat::core::Value] 1 "x")))))

      ((:wat::core::= case "i64-overflow")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::i64::+ (:wat::core::Vector :- [:wat::core::Value] 9223372036854775807 1)))))

      ((:wat::core::= case "div-zero")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::i64::/ (:wat::core::Vector :- [:wat::core::Value] 5 0)))))

      ((:wat::core::= case "f64-type")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::f64::+ (:wat::core::Vector :- [:wat::core::Value] 1.0 "x")))))

      ((:wat::core::= case "rational-type")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::rational::+ (:wat::core::Vector :- [:wat::core::Value] (:wat::i64::to-rational 1) "x")))))

      ((:wat::core::= case "bigint-type")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::bigint::+ (:wat::core::Vector :- [:wat::core::Value] (:wat::i64::to-bigint 1) "x")))))

      ((:wat::core::= case "arity")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::i64::+ (:wat::core::Vector :- [:wat::core::Value] 1)))))

      ;; row 2 — two call sites, two spans. Same error KIND, different SOURCE LINE.
      ((:wat::core::= case "span-a")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::i64::+ (:wat::core::Vector :- [:wat::core::Value] 1 "x")))))

      ((:wat::core::= case "span-b")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::i64::+ (:wat::core::Vector :- [:wat::core::Value] 1 "x")))))

      (:else
       (:wat::kernel::println (:wat::string::concat "unknown case: " case))))))
