(:wat::core::defn :fix::kwargs-pos
  [f <- [:wat::core::keyword
         :wat::core::keyword
         :wat::core::keyword
         :wat::core::keyword
         (:wat::core::Vector :- [:wat::core::i64])
         :wat::core::keyword
         (:wat::core::Vector :- [:wat::core::i64])
         :wat::core::keyword
         (:wat::core::Vector :- [:wat::core::i64])
         :-> :wat::core::nil]]
  -> :wat::core::nil
  (f :wat-tests::recorder :satisfies :wat-tests::Recorder :durable [] :ephemeral [] :impls []))
