;; tests/rete/probe_arc278_2a_alpha_match.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :user::Temp record used by the alpha-match tests.

(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])

;; alpha-match's registered TypeScheme is `(Option :- [(PersistentMap :- [String V])])` (check.rs, arc 278
;; Stone 2a) — V is unconstrained by the params (the map is heterogeneous at runtime); the
;; call-site annotation below pins V to i64 (the ?t binding's field type in these probes).

;; Condition: a Temp whose :value binds ?t and must be > 20.
;; MATCH: 25 binds ?t and 25 > 20 holds → Some({"?t": 25}); PersistentMap/get "?t" → Some(25).
(:wat::core::defn :user::match-binds-and-constrains [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::map::get
    (:wat::core::Option/expect
      (:wat::rete::alpha-match
        (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
        (:user::Temp :value 25)) "matched")
    "?t"))

;; 15 binds ?t but 15 > 20 is false → None (no-error, not a raise).
(:wat::core::defn :user::match-rejects-failed-constraint []
  -> (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])])
  (:wat::rete::alpha-match
    (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
    (:user::Temp :value 15)))

;; Condition head :user::Other ≠ fact type :user::Temp → None.
(:wat::core::defn :user::match-rejects-wrong-type []
  -> (:wat::core::Option :- [(:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])])
  (:wat::rete::alpha-match
    (:wat::core::quote (:user::Other (?t <- :value)))
    (:user::Temp :value 25)))
