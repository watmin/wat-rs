;; VIGILIA experiri — `:wat::kernel::abort`, the phantom shipped in five green
;; tests/reflection fixtures, driven in the arm that IS taken. Same match shape as
;; tests/reflection/wat_arc201_holon_ast_accessors_first_head.wat:11.
(:wat::core::defn :vph::none [] -> (:wat::core::Option :- [:wat::core::i64])
  :wat::core::None)
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::match (:vph::none)
    ((:wat::core::Some x) x)
    (:wat::core::None (:wat::kernel::abort "the reporting arm was TAKEN"))))
