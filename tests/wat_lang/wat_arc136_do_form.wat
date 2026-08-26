;; tests/wat_lang/wat_arc136_do_form.wat — co-located fixture for the sibling probe (.rs).
;; Covers positive tests for :wat::core::do. Each test calls its own named fn via eval_in_frozen.
;; Negative tests use separate *.wat.bad files via startup_from_file.

; test 2: single form — (do x) = x
(:wat::core::defn :t::test2-single [] -> :wat::core::i64
  (:wat::core::do 42))

; test 3: multi form — side effects discarded; final value returned
(:wat::core::defn :t::test3-multi [] -> :wat::core::i64
  (:wat::core::do
    (:wat::i64::+ 1 0)
    (:wat::i64::+ 2 0)
    99))

; test 4: recipient unification — probe returns i64 via do's final form
(:wat::core::defn :t::probe4 [] -> :wat::core::i64
  (:wat::core::do
    (:wat::i64::+ 1 1)
    42))
(:wat::core::defn :t::test4-recipient [] -> :wat::core::i64 (:t::probe4))

; test 6: non-final type unconstrained — String ok as non-final form
(:wat::core::defn :t::test6-non-final [] -> :wat::core::i64
  (:wat::core::do
    "string-not-unit"
    (:wat::i64::+ 1 1)
    42))

; test 7: reflection — signature-of-defn :wat::core::do renders variadic sketch
(:wat::core::defn :t::test7-signature [] -> :wat::core::String
  (:wat::core::let
    [sig-opt (:wat::runtime::signature-of-defn :wat::core::do)
     rendered (:wat::edn::write sig-opt)]
    rendered))

; test 8: tail-call — do in tail position preserves TCO
(:wat::core::defn :t::countdown8 [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::<= n 0)
    
    n
    (:wat::core::do
      (:wat::i64::+ n 0)
      (:t::countdown8 (:wat::i64::- n 1)))))
(:wat::core::defn :t::test8-tail [] -> :wat::core::i64
  (:t::countdown8 100000))

; test 9: nested do — inner result discarded; outer returns 2
(:wat::core::defn :t::test9-nested [] -> :wat::core::i64
  (:wat::core::do
    (:wat::core::do
      (:wat::i64::+ 0 0)
      1)
    2))

; test 10: do inside let body — let scope visible to do
(:wat::core::defn :t::test10-let-body [] -> :wat::core::i64
  (:wat::core::let [x 7]
    (:wat::core::do
      (:wat::i64::+ x 1)
      x)))
