;; Two questions the mint depends on:
;;  1. Is the classifier space OPEN — does a user's own record carry its class name? If yes, `is?`
;;     (the string-taking general form) reaches shapes the nine named predicates never can.
;;  2. Does the family PARTITION? `nil()` is `classified("Symbol","nil")` by construction, so
;;     `is-Nil?` and `is-Symbol?` should BOTH answer true for nil. A rule keyed on `is-Symbol?`
;;     would then also catch every nil.
(:wat::holon::defrecord :probe::Order [id <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rec (:probe::Order :id 7)
     h   (:wat::holon::to-holon rec)
     n   #holon nil]
    (:wat::core::do
      (:wat::kernel::println "classifier of a holonic record's holon:")
      (:wat::kernel::println (:wat::holon::extract-classifier h))
      (:wat::kernel::println "is? h \"probe::Order\":")
      (:wat::kernel::println (:wat::holon::is? h "probe::Order"))
      (:wat::kernel::println "-- the partition question --")
      (:wat::kernel::println "is-Nil?    of nil:") (:wat::kernel::println (:wat::holon::is-Nil? n))
      (:wat::kernel::println "is-Symbol? of nil:") (:wat::kernel::println (:wat::holon::is-Symbol? n))
      (:wat::kernel::println "is-Map?    of nil:") (:wat::kernel::println (:wat::holon::is-Map? n)))))
