;; RELAND-1 over-reach detector — a derive-marker bound must be ACCEPTED.
;; Pattern from tests/types/probe_arc237_derive_verb.wat. :t::Marker is a
;; subtype_edges VALUE, never a types key. Store 4 is the only reason this is 0.
(:wat::core::defrecord :t::A [])
(:wat::core::derive :t::A :t::Marker)
(:wat::core::defn :user::take-marker [m <- :t::Marker] -> :wat::core::i64 42)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
