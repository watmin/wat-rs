;; tests/value/wat_eval_result.wat — co-located fixture for the sibling probe (.rs).
;; Slurped via startup_beside(file!()). Each function covers one test case.
;; No :user::main needed — startup_beside loads defns; tests call each fn via eval_in_frozen.

;; ─── Test 1: eval-ast! returns Ok(holon) ─────────────────────────────────────

(:wat::core::defn :t::test1 [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::core::let
    [program (:wat::core::quote (:wat::holon::to-holon "hello"))]
    (:wat::eval-ast! program)))

;; ─── Test 2: eval-ast! mutation form surfaces as Err ─────────────────────────

(:wat::core::defn :t::test2 [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::core::let
    [program
      (:wat::core::quote
        (:wat::core::defstruct :evil::T [x <- :wat::core::i64]))]
    (:wat::eval-ast! program)))

;; ─── Test 3: eval-edn! parse failure surfaces as Err ─────────────────────────

(:wat::core::defn :t::test3 [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-edn! "(:wat::core::i64::+ 1"))

;; ─── Test 4: eval-digest-string! hash mismatch surfaces as Err ───────────────

(:wat::core::defn :t::test4 [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-digest-string!
    "(:wat::holon::to-holon \"x\")"
    :wat::verify::digest-sha256
    :wat::verify::string "0000000000000000000000000000000000000000000000000000000000000000"))

;; ─── Test 5: eval-edn! wrong arity — lives in separate fixture (startup fails) ─
;; See: tests/value/wat_eval_result_wrong_arity.wat

;; ─── Test 6: try propagates eval err through helper ──────────────────────────

(:wat::core::defn :t::test6-run-dynamic [program <- :wat::WatAST] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::core::Ok (:wat::core::Result/try (:wat::eval-ast! program))))

(:wat::core::defn :t::test6 [] -> :wat::core::String
  (:wat::core::let
    [bad
      (:wat::core::quote
        (:wat::core::defstruct :injected::T [x <- :wat::core::i64]))]
    (:wat::core::match (:t::test6-run-dynamic bad) 
      ((:wat::core::Ok _) "should-not-reach")
      ((:wat::core::Err e) (:wat::core::EvalError/kind e)))))

;; ─── Test 7: eval-err exposes both kind and message ─────────────────────────

(:wat::core::defn :t::test7 [] -> (:wat::core::Tuple :- [:wat::core::String :wat::core::String])
  (:wat::core::let
    [bad
      (:wat::core::quote
        (:wat::core::defstruct :injected::T [x <- :wat::core::i64]))
     r
      (:wat::eval-ast! bad)]
    (:wat::core::match r 
      ((:wat::core::Ok _)
        (:wat::core::Tuple "unreachable" "unreachable"))
      ((:wat::core::Err e)
        (:wat::core::Tuple
          (:wat::core::EvalError/kind e)
          (:wat::core::EvalError/message e))))))
