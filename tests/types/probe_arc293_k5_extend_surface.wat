;; tests/types/probe_arc293_k5_extend_surface.wat — co-located fixture (arc 293 K5, the LAST tool)
;;
;; `extend-surface` is a wat `defmacro`: the user writes a TYPELESS method body; the macro emits one
;; `extend-type` per PAIR backing tier (`$core-record` + `$holon-record`), forwarding the typeless body.
;; `extend-type` fills the method's types from the surface (the 293.4e-pre.iii capability), so the user
;; writes BODY ONLY — "WHERE ARE THE TYPES? the contract." Per the K5 decision (option A, 2026-06-30) the
;; default attaches to BOTH pair tiers, so a `to-record`'d value at either tier inherits it for free.
;;
;; The chain (proven by hand at the foundation probe): a source satisfies S (data + its own method) →
;; `to-record` lifts the DATA up to a backing record → that backing record needs its OWN method to satisfy
;; S (projection carries data, not behavior) → `extend-surface`'s default supplies it → dispatch fires.
;;
;; RED at HEAD: `extend-surface` is unbound (no macro) — the form fails to expand, so the pair backing
;; records never get `dbl`, so `:k5::HasX/dbl` rejects them (`$core-record`/`$holon-record` don't satisfy
;; `:k5::HasX` — missing the method). GREEN after K5.

(:wat::core::defstruct :k5::Pt [x <- :wat::core::i64])

(:wat::core::defsurface :k5::HasX :nature :wat::core::Struct
  :features [x <- :wat::core::i64                                       ; attribute (data)
             (dbl [self <- :k5::HasX] -> :wat::core::i64)])             ; method sig — self a normal binder

;; the SOURCE's own impl — so :k5::Pt satisfies :k5::HasX and can be `to-record`'d:
(:wat::core::extend-type :k5::Pt :k5::HasX
  (dbl [self] (:wat::i64::* (:k5::HasX/x self) 2)))

;; >>> THE TOOL UNDER TEST <<< — body only; types filled from the surface; rides BOTH pair tiers:
(:wat::core::extend-surface :k5::HasX
  (dbl [self] (:wat::i64::* (:k5::HasX/x self) 2)))

;; to-record into each pair tier; the default `dbl` rides both (Option A) — works ONLY via extend-surface:
(:wat::core::defn :k5::demo [] -> :wat::core::i64
  (:wat::core::let
    [p  (:k5::Pt :x 21)
     cr (:wat::core::to-record  p :k5::HasX)        ; -> :k5::HasX$core-record  {x 21}
     hr (:wat::holon::to-record p :k5::HasX)]       ; -> :k5::HasX$holon-record {x 21}
    (:wat::i64::+
      (:k5::HasX/dbl cr)                             ; 21*2 = 42  (core tier default)
      (:k5::HasX/dbl hr))))                          ; 21*2 = 42  (holon tier default)  => 84
