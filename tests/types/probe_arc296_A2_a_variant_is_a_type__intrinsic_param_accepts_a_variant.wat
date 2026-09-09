;; SUBJECT — an INTRINSIC's parameter must accept a narrowed variant, exactly as a
;; user defn's parameter already does. Today they disagree about the same pair.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::Option/expect (:wat::core::Option::Some {:value 42}) "boom")))
