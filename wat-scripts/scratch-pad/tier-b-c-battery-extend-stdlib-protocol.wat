;; Tier B step C battery — door: typeenv / extend_regs / subtype_edges.
;; A user type extend-type-satisfies a STDLIB protocol (`Seqable`).
(:wat::core::defstruct :battery::Box [x <- :wat::core::i64])

(:wat::core::extend-type :battery::Box (:wat::core::Seqable :- [:wat::core::i64])
  (seq [self] -> (:wat::stream::Stream :- [:wat::core::i64])
    (:wat::core::seqable->stream
      (:wat::core::Vector :- [:wat::core::i64] (:battery::Box/x self)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [b (:battery::Box 7)]
    (:wat::kernel::println (:wat::i64::to-string (:battery::Box/x b)))))
