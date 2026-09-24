;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E) — the DRIFT the
;; single-sourcing closes. The 216.5c hand list in `value_is_hashable` named 13 of the 14 variants
;; whose `Hash` arm is `unreachable!()`; it omitted `wat__stream__Stream`. So a BARE Stream — no
;; nesting at all — passed the shallow guard and panicked in `impl Hash for Value`. Laundered
;; through a generic `T` so no call site names the Stream type.
(:wat::core::defn :user::conj-any :- [T] [x <- :T] -> :wat::core::nil
  (:wat::core::let
    [s  (:wat::core::HashSet :- [:T])
     s2 (:wat::hashset::conj s x)]
    (:wat::kernel::println "INSERTED")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::conj-any (:wat::stream::empty)))
