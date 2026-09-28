;; :wat::doc::Row — the runtime doc-row record. pprintln of a Row is
;; `#wat.doc/Row { … }` (the container a record already prints). Prose
;; strings and :examples layout are scoped by this type, not a flag.
;;
;; Fields are `:wat::core::Value` because they are lifted from a
;; `metadata-of` map (heterogeneous). The pretty-printer keys off the
;; record class and the field names.

(:wat::core::defrecord :wat::doc::Row
  [doc          <- wat.type/Value
   added        <- wat.type/Value
   args         <- wat.type/Value
   ret          <- wat.type/Value
   examples     <- wat.type/Value
   see          <- wat.type/Value
   purity       <- wat.type/Value
   determinism  <- wat.type/Value
   totality     <- wat.type/Value
   expand-time  <- wat.type/Value
   category     <- wat.type/Value
   deprecated   <- (:wat::core::Option :- [wat.type/Value])
   alias        <- (:wat::core::Option :- [wat.type/Value])])

(:wat::core::defn :wat::doc::from-map
  [m <- (wat.type/HashMap :- [wat.type/keyword wat.type/Value])]
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
  [name <- wat.type/keyword]
  -> :wat::doc::Row
  (:wat::doc::from-map
    (:wat::core::Option/expect
      (:wat::runtime::metadata-of name)
      "metadata-of")))
