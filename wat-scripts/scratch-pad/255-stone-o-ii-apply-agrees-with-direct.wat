;; Scratch probe — arc 255 Stone O-ii, acceptance row 1.
;;
;; THE PROPERTY UNDER TEST: for a defclause head, `(apply f [args])` and `(f args)` are
;; IDENTICAL — same value, same string rendering — not just "both succeed." Six defclauses,
;; spanning past arithmetic (`sort`, `into`, `filterv`), plus the zero-arg identity
;; `(apply + [])` -> 0 and 3-arg variadic cases.
;;
;; Heterogeneous-arg verbs (`into`, `filterv`) use apply's LEADING-ARGS form
;; `(apply f a1 [spread])` — the spread vector only ever needs to be homogeneous with
;; itself, never with the leading args, so a fn/accumulator arg sits outside it exactly as
;; it does in the corpus (`wat/spawn.wat:502`, `arc109-2iii-fn-bracket-destinations.wat`).

(:wat::core::defn :probe::outcome [r <- (:wat::core::Result :- [:wat::core::Value :wat::core::EvalError])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "err:" (:wat::core::EvalError/message e)))))

(:wat::core::defn :probe::agree [name   <- :wat::core::String
                                 direct <- :wat::WatAST
                                 thru   <- :wat::WatAST]
  -> :wat::core::nil
  (:wat::core::let
    [d (:probe::outcome (:wat::eval-ast! direct))
     a (:probe::outcome (:wat::eval-ast! thru))
     tag (:wat::core::if (:wat::core::= d a) "MATCH" "MISMATCH")]
    (:wat::kernel::println
      (:wat::string::concat name "  DIRECT=" d "  APPLY=" a "  [" tag "]"))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; zero-arg identity — the defclause's own identity element, through apply.
     _01 (:probe::agree "+  zero-arg identity   "
           (:wat::core::quote (:wat::core::+))
           (:wat::core::quote (:wat::core::apply :wat::core::+ (:wat::core::Vector :- [:wat::core::i64]))))

     ;; 3-arg variadic — the reason apply exists.
     _02 (:probe::agree "+  3-arg variadic      "
           (:wat::core::quote (:wat::core::+ 1 2 3))
           (:wat::core::quote (:wat::core::apply :wat::core::+ (:wat::core::Vector :- [:wat::core::i64] 1 2 3))))

     _03 (:probe::agree "*  3-arg variadic      "
           (:wat::core::quote (:wat::core::* 2 3 4))
           (:wat::core::quote (:wat::core::apply :wat::core::* (:wat::core::Vector :- [:wat::core::i64] 2 3 4))))

     _04 (:probe::agree "-  left-fold           "
           (:wat::core::quote (:wat::core::- 10 1 2))
           (:wat::core::quote (:wat::core::apply :wat::core::- (:wat::core::Vector :- [:wat::core::i64] 10 1 2))))

     ;; past arithmetic — sort, into, filterv.
     _05 (:probe::agree "sort  1-ary            "
           (:wat::core::quote (:wat::core::sort (:wat::core::Vector :- [:wat::core::i64] 3 1 2)))
           (:wat::core::quote (:wat::core::apply :wat::core::sort
             (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::core::i64])] (:wat::core::Vector :- [:wat::core::i64] 3 1 2)))))

     _06 (:probe::agree "into  (leading + spread)"
           (:wat::core::quote (:wat::core::into (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::Vector :- [:wat::core::i64] 1 2 3)))
           (:wat::core::quote (:wat::core::apply :wat::core::into (:wat::core::Vector :- [:wat::core::i64])
             (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::core::i64])] (:wat::core::Vector :- [:wat::core::i64] 1 2 3)))))

     _07 (:probe::agree "filterv  (leading + spread)"
           (:wat::core::quote (:wat::core::filterv
             (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::> x 1))
             (:wat::core::Vector :- [:wat::core::i64] 1 2 3)))
           (:wat::core::quote (:wat::core::apply :wat::core::filterv
             (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::> x 1))
             (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::core::i64])] (:wat::core::Vector :- [:wat::core::i64] 1 2 3)))))]
    nil))
