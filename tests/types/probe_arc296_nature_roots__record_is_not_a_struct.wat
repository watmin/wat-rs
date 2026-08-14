;; Arc 296 — THE LOAD-BEARING RED. A record must NOT satisfy a `:wat::core::Struct` slot.
;;
;; This is the row that proves the CAUSE was fixed and not the symptom. `register_builtin`
;; derives each type's subtype edge from `nature.root_keyword()`, so a Struct-natured
;; `:wat::core::Record` umbrella emitted `:wat::core::Record <: :wat::core::Struct` — making
;; EVERY record in wat a subtype of Struct, a claim nobody ever declared. With the nature
;; correct, `child == root` and the guard skips: no edge, and this call is a TypeMismatch.
;;
;; If this file ever FREEZES CLEAN again, the spurious edge is back.

(:wat::core::defrecord :t::Pt [x <- :wat::core::i64])
(:wat::core::defn :t::takes-struct [s <- :wat::core::Struct] -> :wat::core::i64 1)
(:wat::core::defn :t::main [] -> :wat::core::i64 (:t::takes-struct (:t::Pt :x 1)))
