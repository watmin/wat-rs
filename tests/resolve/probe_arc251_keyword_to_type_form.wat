(:wat::core::defn :user::c01a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::core::i64"))))
(:wat::core::defn :user::c01b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::holon::HolonAST"))))
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::core::Vector<wat::core::i64>"))))
(:wat::core::defn :user::c03 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::core::Vector<wat::core::Vector<wat::core::i64>>"))))
(:wat::core::defn :user::c04 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::core::Vector<T>"))))
(:wat::core::defn :user::c05 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::core::HashMap<wat::core::String,wat::core::i64>"))))
(:wat::core::defn :user::c06 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":(wat::core::i64,wat::core::String)"))))
(:wat::core::defn :user::c07 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":()"))))
(:wat::core::defn :user::c08 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node ":(wat::core::Vector<T>,wat::core::i64)"))))
;; c09: proves the tuple form parses back as a type — including this declaration makes startup the gate
(:wat::core::defn :user::c09-f [t :- (wat.type/Tuple wat.type/i64 wat.type/String)] -> :wat::core::nil nil)
