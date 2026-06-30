;; tests/types/probe_arc293_holder_ladder.wat — co-located fixture (arc 293 K1a)
;;
;; The contravariant holder ladder for surface satisfaction. A required `:holder` is a FLOOR:
;;   :holder :wat::core::Struct  accepts struct + record + holon   (widest)
;;   :holder :wat::core::Record  accepts record + holon
;;   :holder :wat::holon::Record accepts holon only                (narrowest)
;;
;; RED at HEAD: satisfaction does an EXACT holder match (check.rs:14698 `agg_holder == req`), so a
;; core RECORD is rejected by a `:holder :Struct` surface and a HOLON by a `:holder :Record` surface.
;; GREEN after K1a: `agg_holder.rank() >= req.rank()` (Struct -1 < Record 0 < HolonRecord +1).

(:wat::core::defsurface :lad::Named   :holder :wat::core::Struct  :features [name <- :wat::core::String])
(:wat::core::defsurface :lad::Stamped :holder :wat::core::Record  :features [at   <- :wat::core::i64])

(:wat::core::defrecord  :lad::Person [name <- :wat::core::String])   ; core record
(:wat::holon::defrecord :lad::Event  [at   <- :wat::core::i64])      ; holon record

(:wat::core::defn :lad::greet [x <- :lad::Named]   -> :wat::core::String (:lad::Named/name x))
(:wat::core::defn :lad::when  [x <- :lad::Stamped] -> :wat::core::i64    (:lad::Stamped/at x))

(:wat::core::defn :lad::demo [] -> :wat::core::String
  (:wat::core::string::concat
    (:lad::greet (:lad::Person "alice"))                  ; record → :holder :Struct  (ladder accepts down)
    " @ "
    (:wat::core::str (:lad::when (:lad::Event 100)))))     ; holon  → :holder :Record  (ladder accepts down)
