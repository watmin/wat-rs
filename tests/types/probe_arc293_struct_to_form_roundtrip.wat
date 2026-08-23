;; B1 regression probe: struct->form + eval-ast! roundtrip — NON-CONCURRENT (no run-thread).
;;
;; Arc 293.R2 collapsed three repr variants into Value::Aggregate; R2.3 dropped
;; :T/new in favour of the bare :T constructor. eval_struct_to_form (runtime.rs)
;; was NOT updated — it still emitted ":{}/new" for the WatAST constructor keyword.
;;
;; RED before B1 fix: struct->form emits (:probe::Pair/new 7 9) — unregistered after R2.3;
;;   eval-ast! returns Err(UnknownFunction :probe::Pair/new).
;; GREEN after B1 fix: struct->form emits (:probe::Pair 7 9) — bare ctor, registered;
;;   eval-ast! returns Ok(Value::Aggregate{ class:"probe::Pair", fields:[i64(7), i64(9)] }).
;;
;; The Rust test (.rs sibling) unwraps the Result and verifies field `a` reads back as 7.
;; NON-CONCURRENT: no run-thread wrapper (that is what got wat-tests/core/struct-to-form.wat
;; #[ignore]'d for arc-170).

(:wat::core::defstruct :probe::Pair [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::core::defn :probe::roundtrip [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::core::let
    [p    (:probe::Pair :a 7 :b 9)
     form (:wat::core::struct->form p)]
    (:wat::eval-ast! form)))
