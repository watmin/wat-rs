;; rune:lint(red-by-design) — :wat::kernel::abort does not resolve on this tree even though the calling arm is never TAKEN at runtime; this tree's resolver checks every match arm's call heads unconditionally
;; VIGILIA experiri — the same phantom in the arm that is NOT taken (the reflection
;; fixtures' shipped shape). Must return 5.
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::let [o (:wat::core::Option.Some {:value 5})]
    (:wat::core::match o
      [:wat::core::Option.Some {:value x} x]
      [:wat::core::Option.None {} (:wat::kernel::abort "the reporting arm was NOT taken")])))
