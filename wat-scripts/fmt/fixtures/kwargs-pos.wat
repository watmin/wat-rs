(:wat::core::defn :fix::kwargs-pos
  [f <- [:wat::core::keyword
         :wat::core::keyword
         :wat::core::keyword
         :wat::core::keyword
         (wat.type/Vector :- [wat.type/i64])
         :wat::core::keyword
         (wat.type/Vector :- [wat.type/i64])
         :wat::core::keyword
         (wat.type/Vector :- [wat.type/i64])
         :-> wat.type/nil]]
  -> wat.type/nil
  (f :wat-tests::recorder :satisfies :wat-tests::Recorder :durable [] :ephemeral [] :impls []))
