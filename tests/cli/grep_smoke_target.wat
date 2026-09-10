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
    ;; Census bait for wat-scripts/grep/bare-variant-constructors.wat —
    ;; that program asks for the RETIRED head spelling. Parse-only; not type-checked.
    (:wat::core::Some 1)
    (:wat::core::Ok 1)
    (:wat::core::Err "e")
    (:wat::core::Option.Some {:value 1})
    (:wat::core::Result.Ok {:value 1})
    (:wat::core::Result.Err {:error "e"})
    (:wat::core::i64::+ 1 2)
    (:wat::rete::core::i64::+ 1 2)
    nil))
