;; tests/types/probe_arc237_derive_verb.wat — co-located fixture
;;
;; Arc 237 follow-on — the user-facing :wat::core::derive verb.

(:wat::core::defrecord :t::A [])
(:wat::core::defrecord :t::B [])

(:wat::core::derive :t::A :t::Marker)
(:wat::core::derive :t::B :t::Marker)

(:wat::core::defn :user::take-marker [m <- :t::Marker] -> :wat::core::i64 42)
(:wat::core::defn :user::go-a [] -> :wat::core::i64 (:user::take-marker (:t::A)))
(:wat::core::defn :user::go-b [] -> :wat::core::i64 (:user::take-marker (:t::B)))
