;; tests/wat_lang/wat_core_try.wat — co-located fixture.
;; :wat::core::Result/try — error-propagation form. Runtime tests only.
;; Check-error tests use separate *.wat.bad files.

;; test1: try on Ok extracts inner value → Result(Ok(42))
(:wat::core::defn :t::test1-try-ok [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Result::Ok {:value (:wat::core::Result/try (:wat::core::Result::Ok {:value 42}))}))

;; test2: try on Err propagates → Result(Err("boom"))
(:wat::core::defn :t::test2-try-err-prop [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Result::Ok {:value (:wat::core::Result/try (:wat::core::Result::Err {:error "boom"}))}))

;; helper for test3
(:wat::core::defn :t::app-unwrap-or-propagate
  [r <- (:wat::core::Result :- [:wat::core::i64 :wat::core::String])]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Result::Ok {:value (:wat::core::Result/try r)}))

;; test3: try propagates across helper function → Result(Err("from-helper"))
(:wat::core::defn :t::test3-try-helper [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:t::app-unwrap-or-propagate (:wat::core::Result::Err {:error "from-helper"})))

;; test4: try chains two bindings in let → Result(Ok(42))
(:wat::core::defn :t::test4-try-let-chain [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [a (:wat::core::Result/try (:wat::core::Result::Ok {:value 10}))
     b (:wat::core::Result/try (:wat::core::Result::Ok {:value 32}))]
    (:wat::core::Result::Ok {:value (:wat::i64::+ a b)})))

;; test5: try short-circuits let on first err → Result(Err("early"))
(:wat::core::defn :t::test5-try-let-short-circuit [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [a (:wat::core::Result/try (:wat::core::Result::Err {:error "early"}))
     b (:wat::core::Result/try (:wat::core::Result::Ok {:value 99}))]
    (:wat::core::Result::Ok {:value (:wat::i64::+ a b)})))

;; helper for test6
(:wat::core::defn :t::app-describe
  [o <- (:wat::core::Option :- [(:wat::core::Result :- [:wat::core::i64 :wat::core::String])])]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::match o 
    [:wat::core::Option::Some {:value r} (:wat::core::Result::Ok {:value (:wat::core::Result/try r)})]
    [:wat::core::Option::None {}    (:wat::core::Result::Err {:error "missing"})]))

;; test6: try inside match arm propagates → Result(Err("inner-boom"))
(:wat::core::defn :t::test6-try-match-arm [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:t::app-describe (:wat::core::Option::Some {:value (:wat::core::Result::Err {:error "inner-boom"})})))

;; test7: try inside Result-returning fn propagates to fn → Result(Err("fn-err"))
(:wat::core::defn :t::test7-try-in-fn-scope [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [f (:wat::core::fn
         [r <- (:wat::core::Result :- [:wat::core::i64 :wat::core::String])]
         -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
         (:wat::core::Result::Ok {:value (:wat::core::Result/try r)}))]
    (f (:wat::core::Result::Err {:error "fn-err"}))))
