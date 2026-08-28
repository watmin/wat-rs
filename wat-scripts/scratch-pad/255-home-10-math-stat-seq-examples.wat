;; Scratch probe — Stone HOME-10. Verifies the exact doc @example expressions
;; for the twelve math/stat/seq verbs BEFORE they ship in intrinsic doc comments.
;; Mirrors `wat/doctest.wat`'s own verify-examples mechanism (quote the expr and
;; the expected form, `:wat::eval-ast!` BOTH, compare with `=`) rather than
;; running them as ordinary statically-checked top-level forms.
;;
;; MEASURED (this probe): `:wat::core::Vector`'s constructor requires an
;; EXPLICIT leading type keyword even through the eval-ast! door (the runtime
;; ctor in collection/eval.rs enforces the same rule check.rs's
;; infer_list_constructor does) — `(:wat::core::Vector 1 2 3)` fails with
;; "first argument must be a type keyword" at EVAL time, not just at
;; static-check time. `:wat::core::Tuple` and `:wat::core::List` do NOT need
;; one (their eval ctors infer per-element, no bracket required). Every
;; @example below spells `:wat::core::Vector` with its type accordingly.
;; Not a permanent fixture.

(:wat::core::defn :probe::check [name <- :wat::core::String
                                  expr <- :wat::WatAST
                                  expected <- :wat::WatAST]
  -> :wat::core::nil
  (:wat::core::match (:wat::eval-ast! expr)
    ((:wat::core::Ok got)
      (:wat::core::match (:wat::eval-ast! expected)
        ((:wat::core::Ok want)
          (:wat::core::if (:wat::core::= got want)
            (:wat::kernel::println (:wat::string::concat "PASS " name))
            (:wat::kernel::println (:wat::string::concat "FAIL(mismatch) " name))))
        ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "FAIL(expected-eval) " name " " (:wat::core::EvalError/message e))))))
    ((:wat::core::Err e) (:wat::kernel::println (:wat::string::concat "FAIL(expr-eval) " name " " (:wat::core::EvalError/message e))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_01 (:probe::check "math/ln" (:wat::core::quote (:wat::math::ln 1.0)) (:wat::core::quote 0.0))
     _02 (:probe::check "math/exp" (:wat::core::quote (:wat::math::exp 0.0)) (:wat::core::quote 1.0))
     _03 (:probe::check "math/sqrt" (:wat::core::quote (:wat::math::sqrt 16.0)) (:wat::core::quote 4.0))
     _04 (:probe::check "math/sin" (:wat::core::quote (:wat::math::sin 0.0)) (:wat::core::quote 0.0))
     _05 (:probe::check "math/cos" (:wat::core::quote (:wat::math::cos 0.0)) (:wat::core::quote 1.0))
     _06 (:probe::check "math/pi" (:wat::core::quote (:wat::math::pi)) (:wat::core::quote 3.141592653589793))

     _07 (:probe::check "stat/mean" (:wat::core::quote (:wat::stat::mean (:wat::core::Vector :wat::core::f64 2.0 4.0))) (:wat::core::quote (:wat::core::Some 3.0)))
     _08 (:probe::check "stat/variance" (:wat::core::quote (:wat::stat::variance (:wat::core::Vector :wat::core::f64 2.0 4.0))) (:wat::core::quote (:wat::core::Some 1.0)))
     _09 (:probe::check "stat/stddev" (:wat::core::quote (:wat::stat::stddev (:wat::core::Vector :wat::core::f64 2.0 4.0))) (:wat::core::quote (:wat::core::Some 1.0)))

     ;; seq — Vector input (the doc examples)
     _10 (:probe::check "seq/zip vector"
           (:wat::core::quote (:wat::seq::zip (:wat::core::Vector :wat::core::i64 1 2 3) (:wat::core::Vector :wat::core::i64 4 5 6)))
           (:wat::core::quote (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple 1 4) (:wat::core::Tuple 2 5) (:wat::core::Tuple 3 6))))
     _11 (:probe::check "seq/window vector"
           (:wat::core::quote (:wat::seq::window (:wat::core::Vector :wat::core::i64 1 2 3 4) 2))
           (:wat::core::quote (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::Vector :wat::core::i64 1 2) (:wat::core::Vector :wat::core::i64 2 3) (:wat::core::Vector :wat::core::i64 3 4))))
     _12 (:probe::check "seq/remove-at vector"
           (:wat::core::quote (:wat::seq::remove-at (:wat::core::Vector :wat::core::i64 1 2 3) 1))
           (:wat::core::quote (:wat::core::Vector :wat::core::i64 1 3)))

     ;; seq — List input (row 2: Seqable survives the carve)
     _13 (:probe::check "seq/zip list"
           (:wat::core::quote (:wat::seq::zip (:wat::core::List 1 2 3) (:wat::core::List 4 5 6)))
           (:wat::core::quote (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple 1 4) (:wat::core::Tuple 2 5) (:wat::core::Tuple 3 6))))
     _14 (:probe::check "seq/window list"
           (:wat::core::quote (:wat::seq::window (:wat::core::List 1 2 3 4) 2))
           (:wat::core::quote (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::Vector :wat::core::i64 1 2) (:wat::core::Vector :wat::core::i64 2 3) (:wat::core::Vector :wat::core::i64 3 4))))
     _15 (:probe::check "seq/remove-at list"
           (:wat::core::quote (:wat::seq::remove-at (:wat::core::List 1 2 3) 1))
           (:wat::core::quote (:wat::core::Vector :wat::core::i64 1 3)))
    ]
    nil))
