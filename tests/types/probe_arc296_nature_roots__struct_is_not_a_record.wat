;; Arc 296 — the WIRE WALL, which must survive the nature fix untouched.
;;
;; A struct may hold impure values, so it can never cross a comms boundary. It must
;; therefore be REJECTED from the `:wat::core::Record` umbrella ("records are always
;; wire friendly.. they are holders of edn" — builder, 2026-08-15).
;;
;; Green here would mean the fix widened the wall instead of correcting the lattice.

(:wat::core::defstruct :t::S [x <- :wat::core::i64])
(:wat::core::defn :t::takes-record [r <- :wat::core::Record] -> :wat::core::i64 1)
(:wat::core::defn :t::main [] -> :wat::core::i64 (:t::takes-record (:t::S :x 1)))
