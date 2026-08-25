;; tests/types/probe_arc293_holder_ladder.wat — co-located fixture (arc 293 K1a)
;;
;; The contravariant nature ladder for surface satisfaction. A required `:nature` is a FLOOR:
;;   :nature :wat::core::Struct  accepts struct + record + holon   (widest)
;;   :nature :wat::core::Record  accepts record + holon
;;   :nature :wat::holon::Record accepts holon only                (narrowest)
;;
;; RED at HEAD: satisfaction does an EXACT nature match (check.rs:14698 `agg_nature == req`), so a
;; core RECORD is rejected by a `:nature :Struct` surface and a HOLON by a `:nature :Record` surface.
;; GREEN after K1a: `agg_nature.rank() >= req.rank()` (Struct -1 < Record 0 < HolonRecord +1).

(:wat::core::defsurface :lad::Named   :nature :wat::core::Struct  :features [name <- :wat::core::String])
(:wat::core::defsurface :lad::Stamped :nature :wat::core::Record  :features [at   <- :wat::core::i64])

(:wat::core::defrecord  :lad::Person [name <- :wat::core::String])   ; core record
(:wat::holon::defrecord :lad::Event  [at   <- :wat::core::i64])      ; holon record

(:wat::core::defn :lad::greet [x <- :lad::Named]   -> :wat::core::String (:lad::Named/name x))
(:wat::core::defn :lad::when  [x <- :lad::Stamped] -> :wat::core::i64    (:lad::Stamped/at x))

(:wat::core::defn :lad::demo [] -> :wat::core::String
  (:wat::string::concat
    (:lad::greet (:lad::Person :name "alice"))                  ; record → :nature :Struct  (ladder accepts down)
    " @ "
    (:wat::core::str (:lad::when (:lad::Event :at 100)))))     ; holon  → :nature :Record  (ladder accepts down)
