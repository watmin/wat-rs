;; tests/types/probe_arc293_k3_to_record.wat — co-located fixture (arc 293 K3)
;;
;; THE THREE PROJECTION VERBS — project a satisfier's EDN attributes into a backing record at the
;; tier the CALLER names (the floor governs satisfaction, NOT projection — 2026-06-29 co-design).
;; ONE shared extraction; the three verbs differ only in the target holder:
;;   (:wat::core::to-struct  p :S) -> :S$struct        (in-locus; type forbids crossing comms)
;;   (:wat::core::to-record  p :S) -> :S$core-record   (portable EDN data)
;;   (:wat::holon::to-record p :S) -> :S$holon-record  (portable EDN data + a derived hologram)
;;
;; A surface emits ALL THREE backing records (same fields; holder is the only variance). `$` is a
;; legal keyword char (K2, confirmed). Each accessor `:S$<tier>/<field>` reads a projected field.
;;
;; RED at HEAD: neither `to-struct` nor `to-record` exists, and K2 emits only `:S$record` (not the
;; triple) — so the three projections and their `$struct`/`$core-record`/`$holon-record` accessors
;; do not resolve and the world fails to type-check. GREEN after K3.

(:wat::core::defstruct :k3::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defsurface :k3::Planar :holder :wat::core::Struct
  :features [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :k3::demo [] -> :wat::core::i64
  (:wat::core::let
    [p  (:k3::Pt 3 4)
     st (:wat::core::to-struct  p :k3::Planar)    ; -> :k3::Planar$struct       {x 3 y 4}
     cr (:wat::core::to-record  p :k3::Planar)    ; -> :k3::Planar$core-record  {x 3 y 4}
     hr (:wat::holon::to-record p :k3::Planar)]   ; -> :k3::Planar$holon-record {x 3 y 4} + hologram
    (:wat::core::i64::+
      (:k3::Planar$struct/x st)                   ; 3   — read off the struct projection
      (:wat::core::i64::+
        (:k3::Planar$core-record/y cr)            ; 4   — read off the core-record projection
        (:k3::Planar$holon-record/x hr)))))       ; 3   — read off the holon-record projection  => 3+4+3 = 10
