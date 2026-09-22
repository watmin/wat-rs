;; Stone 255.12, sites 3 and 4 — the two RUNTIME equalities, reached from wat.
;;
;; ⚠ Neither of these is visible to `--check`. Site 3 is the EDN→typed-value
;; coerce table (`edn::render::edn_to_typed_value_inner`), reached here through
;; `:wat::edn::validate`; site 4 is the defclause dispatcher's runtime type match
;; (`function::subsume::value_matches_type_by_name`). Both compared a stored
;; `TypeExpr::Path` against a hardcoded `:wat::core::…` literal with `==`, so a
;; `wat.type/`-spelled annotation — which stores `:wat::type::…` — missed.
;;
;; Both produced a diagnostic that contradicted itself, because `format_type`
;; DENOTES on the way out while the comparison did not:
;;   site 3: Validation.Invalid {:expected ":wat::core::i64" :got "Integer"}
;;   site 4: NoMatchingClause … expected :wat::core::i64, got wat::core::i64
;;
;; This is the 8d-ii STOP, and it is reachable with NOTHING in the tree converted.

;; ─── site 3 — the coerce table ──────────────────────────────────────────
(:wat::core::defn :p255_12::coerce-core-ok [] -> :wat::edn::Validation
  (:wat::edn::validate 42 :wat::core::i64))

(:wat::core::defn :p255_12::coerce-type-ok [] -> :wat::edn::Validation
  (:wat::edn::validate 42 :wat::type::i64))

;; NON-VACUITY — a value of the WRONG type is still refused, with the diagnostic.
(:wat::core::defn :p255_12::coerce-type-wrong [] -> :wat::edn::Validation
  (:wat::edn::validate "not-an-integer" :wat::type::i64))

;; The PARAMETRIC half — head and argument both `wat.type`-spelled.
(:wat::core::defn :p255_12::coerce-type-parametric-ok [] -> :wat::edn::Validation
  (:wat::edn::validate [1 2 3] (:wat::type::Vector :- [:wat::type::i64])))

(:wat::core::defn :p255_12::coerce-type-parametric-wrong [] -> :wat::edn::Validation
  (:wat::edn::validate ["a"] (:wat::type::Vector :- [:wat::type::i64])))

;; ─── site 4 — defclause runtime dispatch ────────────────────────────────
(:wat::core::defclause :p255_12::pick-type
  ([x <- :wat::type::i64] -> :wat::core::i64 x))

(:wat::core::defn :p255_12::dispatch-type-ok [] -> :wat::core::i64
  (:p255_12::pick-type 42))

;; NON-VACUITY — the cure must not turn the matcher into a WILDCARD. Two clauses
;; of one name, one keyed `wat.type`-spelled and one `wat.core`-spelled, at two
;; different types: each call must still select ITS OWN clause. A denotation that
;; over-collapsed would make the first clause swallow both.
(:wat::core::defclause :p255_12::discriminate
  ([x <- :wat::type::i64]     -> :wat::core::String "i64-clause")
  ([s <- :wat::core::String]  -> :wat::core::String "string-clause"))

(:wat::core::defn :p255_12::dispatch-picks-i64 [] -> :wat::core::String
  (:p255_12::discriminate 42))

(:wat::core::defn :p255_12::dispatch-picks-string [] -> :wat::core::String
  (:p255_12::discriminate "x"))
