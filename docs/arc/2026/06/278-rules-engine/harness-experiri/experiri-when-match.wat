(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :probe::E])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String  ok <- :wat::core::bool])

;; IDENTICAL match expression, three positions. Uncomment one rule at a time.
(:wat::rete::defrule :probe::in-when
  :when  [(:probe::In (?k <- :k) (?v <- :v))
          (:wat::rete::where (:wat::rete::core::match ?v (:probe::E::A true) (:probe::E::B false)))]
  :then  [(:probe::Out :k ?k :ok true)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "loaded"))
