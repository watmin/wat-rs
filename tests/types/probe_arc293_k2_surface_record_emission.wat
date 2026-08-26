;; tests/types/probe_arc293_k2_surface_record_emission.wat — co-located fixture (arc 293 K2)
;;
;; A `defsurface` EMITS THREE concrete backing aggregates from its `:features` ATTRIBUTES (Field
;; members only — methods are behavior, never data, excluded). All three share the same fields;
;; nature is the only variance: `$struct` (Struct), `$core-record` (Record), `$holon-record`
;; (HolonRecord). (`$` is a legal keyword char — confirmed empirically.)
;;
;; Arc 293 K2 originally emitted a single `$record` (nature = surface's `:nature`). K3 changed
;; the emit to the fixed triple; this fixture updated to use `$core-record` (nature = Record,
;; matching the surface's `:nature :wat::core::Record` — the portable EDN data tier).
;;
;; RED at HEAD: `defsurface` emits only the SurfaceDef; `:k2::Pt$core-record` does not exist,
;; so neither the ctor `(:k2::Pt$core-record 3 4)` nor the accessor `:k2::Pt$core-record/x` resolves.
;; GREEN after K3 (K2 now trivially satisfied since K3 subsumes K2).

(:wat::core::defsurface :k2::Pt :nature :wat::core::Record
  :features [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :k2::demo [] -> :wat::core::i64
  (:wat::i64::+
    (:k2::Pt$core-record/x (:k2::Pt$core-record' 3 4))     ; construct the emitted backing record + read x
    (:k2::Pt$core-record/y (:k2::Pt$core-record' 3 4))))   ; … + read y   ⇒ 3 + 4 = 7
