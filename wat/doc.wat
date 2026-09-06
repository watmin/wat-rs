;; :wat::doc::Row — the runtime doc-row record. pprintln of a Row is
;; `#wat.doc/Row { … }` (the container a record already prints). Prose
;; strings and :examples layout are scoped by this type, not a flag.
;;
;; Fields are `:wat::core::Value` because they are lifted from a
;; `metadata-of` map (heterogeneous). The pretty-printer keys off the
;; record class and the field names.

(:wat::core::defrecord :wat::doc::Row
  [doc          <- :wat::core::Value
   added        <- :wat::core::Value
   args         <- :wat::core::Value
   ret          <- :wat::core::Value
   examples     <- :wat::core::Value
   see          <- :wat::core::Value
   purity       <- :wat::core::Value
   determinism  <- :wat::core::Value
   totality     <- :wat::core::Value
   expand-time  <- :wat::core::Value
   category     <- :wat::core::Value
   deprecated   <- (:wat::core::Option :- [:wat::core::Value])
   alias        <- (:wat::core::Option :- [:wat::core::Value])])

(:wat::core::defn :wat::doc::from-map
  [m <- (:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])]
  -> :wat::doc::Row
  (:wat::doc::Row
    :doc          (:wat::core::Option/expect (:wat::core::get m :doc) "doc")
    :added        (:wat::core::Option/expect (:wat::core::get m :added) "added")
    :args         (:wat::core::Option/expect (:wat::core::get m :args) "args")
    :ret          (:wat::core::Option/expect (:wat::core::get m :ret) "ret")
    :examples     (:wat::core::Option/expect (:wat::core::get m :examples) "examples")
    :see          (:wat::core::Option/expect (:wat::core::get m :see) "see")
    :purity       (:wat::core::Option/expect (:wat::core::get m :purity) "purity")
    :determinism  (:wat::core::Option/expect (:wat::core::get m :determinism) "determinism")
    :totality     (:wat::core::Option/expect (:wat::core::get m :totality) "totality")
    :expand-time  (:wat::core::Option/expect (:wat::core::get m :expand-time) "expand-time")
    :category     (:wat::core::Option/expect (:wat::core::get m :category) "category")
    :deprecated   (:wat::core::get m :deprecated)
    :alias        (:wat::core::get m :alias)))

(:wat::core::defn :wat::doc::of
  [name <- :wat::core::keyword]
  -> :wat::doc::Row
  (:wat::doc::from-map
    (:wat::core::Option/expect
      (:wat::runtime::metadata-of name)
      "metadata-of")))
