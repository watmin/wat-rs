;; Does a RETE-spelled `if` — a Form row that re-dispatches to core `if`'s genuine runtime arm —
;; actually FIRE inside a `(:wat::rete::where ...)`?
;;
;; This is the load-bearing control for the `cond` question. A `where` body is never
;; macro-expanded (defrule quotes :when verbatim; eval_test_core calls runtime::eval_inner on the
;; raw AST), so a MACRO cannot survive there. A runtime special form can. If this prints hits=1,
;; then "expand rete cond into rete if" is a correct TARGET and macro-expansion of the where body
;; is the ONLY remaining blocker — rather than there being a second problem underneath.

(:wat::core::defrecord :probe::Req [a <- :wat::core::bool])
(:wat::core::defrecord :probe::Hit [a <- :wat::core::bool])

(:wat::rete::defrule :probe::r1
  :when
  [(:probe::Req (?a <- :a))
   (:wat::rete::where (:wat::rete::core::if ?a true false))]
  :then
  [(:probe::Hit :a ?a)])

(:wat::rete::defquery :probe::q-Hit
  :params []
  :when [(?fact <- :probe::Hit)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::core::PersistentVector (:probe::r1))
     staged  (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q-Hit))) (:probe::Req :a true))
     fired   (:wat::rete::fire-rules staged)
     hits    (:wat::rete::query fired (:probe::q-Hit))]
    (:wat::kernel::println
      (:wat::core::string::concat "hits=" (:wat::core::str (:wat::core::length hits))))))
