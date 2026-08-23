;; tests/wat_lang/wat_core_try.wat — co-located fixture.
;; :wat::core::Result/try — error-propagation form. Runtime tests only.
;; Check-error tests use separate *.wat.bad files.

;; test1: try on Ok extracts inner value → Result(Ok(42))
(:wat::core::defn :t::test1-try-ok [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Ok (:wat::core::Result/try (:wat::core::Ok 42))))

;; test2: try on Err propagates → Result(Err("boom"))
(:wat::core::defn :t::test2-try-err-prop [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Ok (:wat::core::Result/try (:wat::core::Err "boom"))))

;; helper for test3
(:wat::core::defn :t::app-unwrap-or-propagate
  [r <- (:wat::core::Result :- [:wat::core::i64 :wat::core::String])]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Ok (:wat::core::Result/try r)))

;; test3: try propagates across helper function → Result(Err("from-helper"))
(:wat::core::defn :t::test3-try-helper [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:t::app-unwrap-or-propagate (:wat::core::Err "from-helper")))

;; test4: try chains two bindings in let → Result(Ok(42))
(:wat::core::defn :t::test4-try-let-chain [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [a (:wat::core::Result/try (:wat::core::Ok 10))
     b (:wat::core::Result/try (:wat::core::Ok 32))]
    (:wat::core::Ok (:wat::core::i64::+ a b))))

;; test5: try short-circuits let on first err → Result(Err("early"))
(:wat::core::defn :t::test5-try-let-short-circuit [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [a (:wat::core::Result/try (:wat::core::Err "early"))
     b (:wat::core::Result/try (:wat::core::Ok 99))]
    (:wat::core::Ok (:wat::core::i64::+ a b))))

;; helper for test6
(:wat::core::defn :t::app-describe
  [o <- (:wat::core::Option :- [(:wat::core::Result :- [:wat::core::i64 :wat::core::String])])]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::match o 
    ((:wat::core::Some r) (:wat::core::Ok (:wat::core::Result/try r)))
    (:wat::core::None    (:wat::core::Err "missing"))))

;; test6: try inside match arm propagates → Result(Err("inner-boom"))
(:wat::core::defn :t::test6-try-match-arm [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:t::app-describe (:wat::core::Some (:wat::core::Err "inner-boom"))))

;; test7: try inside Result-returning fn propagates to fn → Result(Err("fn-err"))
(:wat::core::defn :t::test7-try-in-fn-scope [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [f (:wat::core::fn
         [r <- (:wat::core::Result :- [:wat::core::i64 :wat::core::String])]
         -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
         (:wat::core::Ok (:wat::core::Result/try r)))]
    (f (:wat::core::Err "fn-err"))))
