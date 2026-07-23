;; Co-located fixture for wat_dispatch_193a.rs — slurped via startup_beside(file!()).
;; Three compute fns (one per passing test). The negative startup test uses wat_dispatch_193a.wat.bad.

(:wat::core::use! :rust::test::MathUtils)

(:wat::core::defn :my::compute-add [] -> :wat::core::i64
  (:rust::test::MathUtils::add 40 2))

(:wat::core::defn :my::compute-some [] -> :wat::core::i64
  (:wat::core::match (:rust::test::MathUtils::maybe_double 21) 
    ((:wat::core::Some v) v)
    (:wat::core::None -1)))

(:wat::core::defn :my::compute-none [] -> :wat::core::i64
  (:wat::core::match (:rust::test::MathUtils::maybe_double 0) 
    ((:wat::core::Some v) v)
    (:wat::core::None -1)))

