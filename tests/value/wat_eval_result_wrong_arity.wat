;; tests/value/wat_eval_result_wrong_arity.wat — NEGATIVE fixture.
;; startup_from_file must return Err (type checker catches arity mismatch at startup).
;; :wat::eval-edn! takes 1 arg; calling with 2 args is a structural arity mismatch.

(:wat::core::defn :t::wrong-arity [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-edn! "foo" "bar-extra"))
