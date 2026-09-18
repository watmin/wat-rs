;; rune:lint(red-by-design) — :wat::kernel::abort does not resolve on this tree; the arm that calls it IS reachable in :user::go's body, so the refusal is the phantom made visible, not a fixture defect
;; VIGILIA experiri — `:wat::kernel::abort`, the phantom shipped in five green
;; tests/reflection fixtures, driven in the arm that IS taken. Same match shape as
;; tests/reflection/wat_arc201_holon_ast_accessors_first_head.wat:11.
(:wat::core::defn :vph::none [] -> (:wat::core::Option :- [:wat::core::i64])
  :wat::core::Option.None)
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::match (:vph::none)
    [:wat::core::Option.Some {:value x} x]
    [:wat::core::Option.None {} (:wat::kernel::abort "the reporting arm was TAKEN")]))
