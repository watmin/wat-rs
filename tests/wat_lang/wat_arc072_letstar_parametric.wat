;; tests/wat_lang/wat_arc072_letstar_parametric.wat — co-located fixture.
;; Arc 072 regression — parametric type keywords with <> lex cleanly.

;; test1: (Result :- [i64 String]) no whitespace — simple payload → returns i64 43
(:wat::core::defn :t::test1-result-simple [] -> :wat::core::i64
  (:wat::core::let
    [wrapped   (:wat::core::Result::Ok {:value 42})
     extracted (:wat::core::match wrapped 
                 [:wat::core::Result::Ok {:value n} (:wat::i64::+ n 1)]
                 [:wat::core::Result::Err {:error _} -1])]
    extracted))

;; test2: (Result :- [(Tuple :- [i64 i64]) i64]) — tuple payload → returns i64 11
(:wat::core::defn :t::wrap-it [] -> (:wat::core::Result :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64])
  (:wat::core::Result::Ok {:value (:wat::core::Tuple 7 11)}))

(:wat::core::defn :t::test2-result-tuple [] -> :wat::core::i64
  (:wat::core::let
    [wrapped   (:t::wrap-it)
     extracted (:wat::core::match wrapped 
                 [:wat::core::Result::Ok {:value pair} (:wat::core::second pair)]
                 [:wat::core::Result::Err {:error _} -1])]
    extracted))

;; test4: operator < and >= still lex as keywords → returns bool true
(:wat::core::defn :t::test4-operator-lt-ge [] -> :wat::core::bool
  (:wat::core::if (:wat::core::< 1 2) 
    (:wat::core::>= 5 5)
    false))
