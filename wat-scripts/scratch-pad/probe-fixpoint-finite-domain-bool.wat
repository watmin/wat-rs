;; MEASUREMENT — a computed head whose field type has a domain of TWO.
;; `flag <- bool`, `:then` derives `(not ?flag)`. The fact domain is {F(true), F(false)}.
;; It CANNOT diverge: two facts is the whole universe. Does the verifier refuse it anyway?
(:wat::core::defrecord :fd::F [flag <- :wat::core::bool])

(:wat::rete::defrule :fd::flip
  :when  [(:fd::F (?b <- :flag))]
  :then  [(:fd::F :flag (:wat::rete::core::not ?b))])

(:wat::rete::defquery :fd::q :params [] :when [(?fact <- :fd::F)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :fd)
         session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fd::q)))
         session (:wat::rete::insert session (:fd::F :flag true))
         fired   (:wat::rete::fire-rules session)]
        (:wat::rete::query fired (:fd::q))))))
