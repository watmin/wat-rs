;; Probe — print the Slot set. A green over zero Slots proves nothing.
(:wat::core::defn :user::show-slot [s <- :wat::fmt::Slot] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::interpolate "  SLOT head={h} glued={g}"
      :h (:wat::fmt::Slot/head s)
      :g (:wat::i64::to-string (:wat::fmt::Slot/glued s)))))

(:wat::core::defn :user::is-fn-slot? [s <- :wat::fmt::Slot] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::fmt::Slot/head s) ":wat::core::fn")
    (:wat::i64::= (:wat::fmt::Slot/glued s) 3)
    false))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [slots   (:wat::fmt::slots-from-registry)
     n       (:wat::core::length slots)
     fn-hits (:wat::core::length
               (:wat::core::into (:wat::core::PersistentVector :- [:wat::fmt::Slot])
                 (:wat::core::filter :user::is-fn-slot? slots)))
     refused (:wat::fmt::slot-of-syntax "(:wat::core::fn <x>+ -> :T y)")]
    (:wat::core::do
      (:wat::kernel::println
        (:wat::string::interpolate "SLOTS={n}" :n (:wat::i64::to-string n)))
      (:wat::core::run! :user::show-slot slots)
      (:wat::kernel::println
        (:wat::string::interpolate "FN_SLOT={f}  REFUSAL={r}"
          :f (:wat::core::if (:wat::i64::= fn-hits 1) "true" "false")
          :r (:wat::core::if (:wat::core::empty? refused) "true" "false"))))))
