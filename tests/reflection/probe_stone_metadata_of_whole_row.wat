;; tests/reflection/probe_stone_metadata_of_whole_row.wat
;; just-eval fixtures for probe_stone_metadata_of_whole_row.rs.
;;
;; Registry branch: :wat::rete::step-payload (five @args, ≥1 example).
;; Wat branch:      :wat::core::sort (a defclause with a doc metadata-map).
;; Alias row:       :wat::rete::i64::> (@alias, still carries the target's axes).
(:wat::core::defn :user::step-payload-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])])
  (:wat::runtime::metadata-of :wat::rete::step-payload))

(:wat::core::defn :user::sort-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])])
  (:wat::runtime::metadata-of :wat::core::sort))

(:wat::core::defn :user::alias-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])])
  (:wat::runtime::metadata-of :wat::rete::i64::>))
