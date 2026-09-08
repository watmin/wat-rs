;; Positive control for every program under wat-scripts/grep/.
;; Driven by grep_programs_still_match.rs. Parse only — wat --grep does not type-check.
(:wat::core::defn :smoke::dup [] -> :wat::core::nil
  nil)

(:wat::core::defn :smoke::dup [] -> :wat::core::nil
  nil)

(:wat::core::defn :smoke::hits [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::first [])
    (:wat::core::Option/expect
      (:wat::core::HashMap/get (:wat::core::HashMap) "k")
      "missing")
    (:wat::core::Some {:value 1})
    (:wat::core::Ok {:value 1})
    (:wat::core::Err {:error "e"})
    (:wat::core::i64::+ 1 2)
    (:wat::rete::core::i64::+ 1 2)
    nil))
