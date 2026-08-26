;; tests/types/probe_arc293_k3_to_record.wat — co-located fixture (arc 293 K3-revise)
;;
;; THE PAIR of projection verbs — ONE-WAY UP to the pure tier the caller names.
;; Projection never descends to struct (already in locus; impure tier).
;;   (:wat::core::to-record  p :S) → :S$core-record   (portable EDN data)
;;   (:wat::holon::to-record p :S) → :S$holon-record  (portable EDN data + derived hologram)
;;
;; A surface emits the PAIR of backing records (same fields; nature is the only variance).
;; `$` is a legal keyword char. Each accessor `:S$<tier>/<field>` reads a projected field.
;;
;; RETIRED 293 K3-revise: `to-struct` + `:S$struct` — projection is ONE-WAY UP; the impure
;; tier is never a projection target (see AGGREGATE-MODEL.md § to-record, 2026-06-29).
;;
;; GREEN after K3-revise. Returns 7 = y from $core-record (4) + x from $holon-record (3).

(:wat::core::defstruct :k3::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defsurface :k3::Planar :nature :wat::core::Struct
  :features [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :k3::demo [] -> :wat::core::i64
  (:wat::core::let
    [p  (:k3::Pt :x 3 :y 4)
     cr (:wat::core::to-record  p :k3::Planar)    ; -> :k3::Planar$core-record  {x 3 y 4}
     hr (:wat::holon::to-record p :k3::Planar)]   ; -> :k3::Planar$holon-record {x 3 y 4} + hologram
    (:wat::i64::+
      (:k3::Planar$core-record/y cr)              ; 4   — read y off the core-record projection
      (:k3::Planar$holon-record/x hr))))          ; 3   — read x off the holon-record projection => 7
