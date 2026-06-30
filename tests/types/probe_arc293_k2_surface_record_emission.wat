;; tests/types/probe_arc293_k2_surface_record_emission.wat — co-located fixture (arc 293 K2)
;;
;; A `defsurface` EMITS a concrete backing record `:S$record` (a real, registered AggregateDef) from
;; its `:features` ATTRIBUTES (Field members only — methods are behavior, never data, excluded). The
;; emitted record's holder = the surface's `:holder`; its fields = the surface's attribute members.
;; This is `to-record`'s (K3) return type. (`$` is a legal keyword char — confirmed empirically.)
;;
;; RED at HEAD: `defsurface` emits only the SurfaceDef; `:k2::Pt$record` does not exist, so neither the
;; ctor `(:k2::Pt$record 3 4)` nor the accessor `:k2::Pt$record/x` resolves. GREEN after K2.

(:wat::core::defsurface :k2::Pt :holder :wat::core::Record
  :features [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :k2::demo [] -> :wat::core::i64
  (:wat::core::i64::+
    (:k2::Pt$record/x (:k2::Pt$record 3 4))     ; construct the emitted backing record + read x
    (:k2::Pt$record/y (:k2::Pt$record 3 4))))   ; … + read y   ⇒ 3 + 4 = 7
