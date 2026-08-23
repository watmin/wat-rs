;; tests/function/probe_arc241_stone5_defclause_rest_dispatch.wat
;; Arc 241 Stone 241.5 — defclause & rest-binder runtime dispatch.
;; Co-located fixture, slurped via startup_beside(file!()).
;; Startup-fail negative cases are in sibling *.wat.bad files.

;; Contracts 1-2 — variadic-min with rest; also empty-rest case
(:wat::core::defclause :c12::sum-all
  ([first <- :wat::core::i64
    & rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc n))
      first
      rest)))
(:wat::core::defn :user::c01-variadic [] -> :wat::core::i64 (:c12::sum-all 1 2 3 4))
(:wat::core::defn :user::c02-empty-rest [] -> :wat::core::i64 (:c12::sum-all 42))

;; Contract 3-4 — rest-only clause (0+ args)
(:wat::core::defclause :c34::count-args
  ([& rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::length rest)))
(:wat::core::defn :user::c03-rest-only [] -> :wat::core::i64 (:c34::count-args 10 20 30))
(:wat::core::defn :user::c04-rest-only-empty [] -> :wat::core::i64 (:c34::count-args))

;; Contract 8 — mixed clause set (first fixed clause arity-mismatches → second rest clause matches)
(:wat::core::defclause :c8::flex
  ([x <- :wat::core::i64] -> :wat::core::i64 x)
  ([first <- :wat::core::i64
    & rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc n))
      first
      rest)))
(:wat::core::defn :user::c08-mixed [] -> :wat::core::i64 (:c8::flex 10 20 30))
