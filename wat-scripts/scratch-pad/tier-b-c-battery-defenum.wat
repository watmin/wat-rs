;; Tier B step C battery — door: unit_variant / typeenv.
;; A user defenum; variants are keywords a unit_variant lookup could hit.
(:wat::core::defenum :battery::Color :wat::enum::Pure :Red :Blue :Green)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a :battery::Color::Red
     b :battery::Color::Blue]
    (:wat::kernel::println
      (:wat::core::if (:wat::core::= a b) "same" "diff"))))
