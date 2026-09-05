;; VIGILIA experiri — the same phantom in the arm that is NOT taken (the reflection
;; fixtures' shipped shape). Must return 5.
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::let [o (:wat::core::Some 5)]
    (:wat::core::match o
      ((:wat::core::Some x) x)
      (:wat::core::None (:wat::kernel::abort "the reporting arm was NOT taken")))))
