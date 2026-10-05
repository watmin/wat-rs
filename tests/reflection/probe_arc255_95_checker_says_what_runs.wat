;; The consumer is a defclause: a defn stores the annotation and does not
;; re-check it at run time. defclause dispatch does, through
;; value_matches_type_by_name. The parameter is the type the checker states.
(:wat::core::defclause :user::take-symbol
  ([k <- wat.type/symbol] -> wat.type/symbol
    k))

(:wat::core::defn :user::consume-compose []
  -> wat.type/symbol
  (:user::take-symbol
    (:wat::runtime::compose-variant :wat::core::Option :Some)))

(:wat::core::defn :user::consume-parent []
  -> wat.type/symbol
  (:user::take-symbol
    (:wat::core::Option/expect
      (:wat::runtime::variant-parent-of :wat::core::Option.Some)
      "Option.Some has a parent")))

(:wat::core::defn :user::consume-type-name []
  -> wat.type/symbol
  (:user::take-symbol
    (:wat::runtime::TypeInfo/name (:wat::runtime::type-of :wat::core::Option))))
