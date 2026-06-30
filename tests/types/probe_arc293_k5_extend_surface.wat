;; tests/types/probe_arc293_k5_extend_surface.wat — co-located fixture (arc 293 K5, the LAST tool)
;;
;; `extend-surface` is a wat defmacro: the user writes a TYPELESS body; the macro fills the method
;; types FROM the surface's declared sigs and expands to `extend-type`. "WHERE ARE THE TYPES? the
;; contract." The default attaches to every backing tier that SATISFIES S's floor (the same K3 line:
;; a sub-floor projection keeps its data but loses surface behavior). This surface is STRUCT-floored
;; (`:holder :wat::core::Struct`, the widest), so ALL THREE backing tiers satisfy and the default
;; rides every one of them — Option A exercised honestly. The default body reads `(:S/n self)` (the
;; surface accessor), which type-checks because each tier satisfies the Struct floor.
;;
;; RED at HEAD: `extend-surface` does not exist (no macro, no surface-method-sig reflection seam) —
;; the form fails to expand, so the default is never registered and the backing records do not
;; satisfy `:k5::Adder` (missing `add`), so `:k5::Adder/add` rejects them. GREEN after K5.

(:wat::core::defsurface :k5::Adder :holder :wat::core::Struct
  :features [n <- :wat::core::i64                                                    ; attribute (data)
             (add [self <- :k5::Adder  x <- :wat::core::i64] -> :wat::core::i64)])   ; method sig — self a normal binder

;; the DEFAULT impl — body only; types come from the surface; rides all three Struct-floor-satisfying tiers:
(:wat::core::extend-surface :k5::Adder
  (add [self x] (:wat::core::i64::+ (:k5::Adder/n self) x)))

;; call through the SURFACE dispatch on each tier's backing record (all satisfy the Struct floor):
(:wat::core::defn :k5::demo [] -> :wat::core::i64
  (:wat::core::i64::+
    (:k5::Adder/add (:k5::Adder$struct 5) 3)            ; struct: 5 + 3 =  8
    (:wat::core::i64::+
      (:k5::Adder/add (:k5::Adder$core-record 10) 3)    ; core:  10 + 3 = 13
      (:k5::Adder/add (:k5::Adder$holon-record 20) 3))))   ; holon: 20 + 3 = 23   => 8+13+23 = 44
