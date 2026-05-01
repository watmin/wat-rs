;; wat/holon.wat — loose verbs in the :wat::holon::* namespace.
;;
;; The wat/holon/ subdirectory ships one named PascalCase holon per
;; file (wat/holon/Subtract.wat → :wat::holon::Subtract, etc.). This
;; top-level file is the home for :wat::holon::* verbs that DON'T
;; constitute their own named holon — closures, factories, and
;; convenience functions that operate on existing holon primitives.
;;
;; Parallels wat/stream.wat (loose :wat::stream::* HOFs) and other
;; namespace-level top-level files per § G's filesystem-path-mirrors-
;; FQDN doctrine. Future additions: any :wat::holon::* verb whose
;; name doesn't match a substrate-defined holon type lands here.
;;
;; Currently shipped: three Hologram/get filter factories (originally
;; wat/holon/Filter.wat — moved 2026-05-01 because the file shipped
;; verbs, not a Filter type; per gaze ward's reading, the file's
;; basename was lying about what it housed).
;;
;; ─── Hologram/get filter factories ──────────────────────────────────
;;
;; The arc-074 Hologram/get takes a user-supplied filter
;; `:fn(:wat::core::f64) -> :wat::core::bool` that decides whether the
;; highest-cosine candidate is "close enough" to return. The substrate
;; ships three opinionated factories so consumers don't have to
;; hand-roll the canonical thresholds.
;;
;; Each factory takes the encoder dim `d` and returns a closure with
;; the floor baked in. Pass the closure to Hologram/get's filter
;; parameter.
;;
;; Usage:
;;
;;   ;; strict — only return when cosine clears the coincident floor
;;   (:wat::holon::Hologram/get store pos probe
;;     (:wat::holon::filter-coincident 10000))
;;
;;   ;; looser — return when there's any presence above the noise floor
;;   (:wat::holon::Hologram/get store pos probe
;;     (:wat::holon::filter-present 10000))
;;
;;   ;; pure population readout — no gating; whatever scored highest wins
;;   (:wat::holon::Hologram/get store pos probe
;;     (:wat::holon::filter-accept-any))
;;
;; Why factories rather than plain functions: the floor depends on `d`
;; (the encoding dimension), and `d` is a per-store constant. Baking
;; `d` into the closure at construction time is honest — the filter
;; carries the same threshold the store was built against.
;;
;; Why these aren't substrate primitives in Rust: they're three
;; closures over the f64 floor accessors that already are primitives.
;; Wat can express them; substrate doesn't earn its keep here.

;; ─── filter-coincident — strict, "same point on the algebra grid" ─
;;
;; Returns true iff `(1 - cos) < coincident-floor(d)`. Matches the
;; semantics of `:wat::holon::coincident?` but works on a raw cosine
;; value instead of two HolonAST inputs.
;;
;; Arc 076: d is read from the ambient `:wat::config::dim-count` rather
;; than passed by the caller. The filter captures the floor at the
;; call site's ambient d; pass through `Hologram/make` once and the
;; entire store carries the same threshold.
(:wat::core::define
  (:wat::holon::filter-coincident
    -> :fn(wat::core::f64)->wat::core::bool)
  (:wat::core::let*
    (((floor :wat::core::f64)
      (:wat::holon::coincident-floor (:wat::config::dim-count))))
    (:wat::core::lambda ((cos :wat::core::f64) -> :wat::core::bool)
      (:wat::core::< (:wat::core::- 1.0 cos) floor))))

;; ─── filter-present — looser, "signal detected above noise" ───────
;;
;; Returns true iff `cos > presence-floor(d)`. Matches the semantics
;; of `:wat::holon::presence?` but works on a raw cosine value.
;; Use when the cache is acting as a "best-known reasonable answer"
;; lookup rather than "did I see this exact form before."
;;
;; Arc 076: d is read from the ambient `:wat::config::dim-count`.
(:wat::core::define
  (:wat::holon::filter-present
    -> :fn(wat::core::f64)->wat::core::bool)
  (:wat::core::let*
    (((floor :wat::core::f64)
      (:wat::holon::presence-floor (:wat::config::dim-count))))
    (:wat::core::lambda ((cos :wat::core::f64) -> :wat::core::bool)
      (:wat::core::> cos floor))))

;; ─── filter-accept-any — null gate, returns whatever scored best ──
;;
;; Returns true unconditionally. Useful when the consumer wants the
;; population's nearest neighbor without any floor — e.g., taking the
;; cell's argmax for a soft scoring loop where the consumer applies
;; their own gate downstream.
(:wat::core::define
  (:wat::holon::filter-accept-any
    -> :fn(wat::core::f64)->wat::core::bool)
  (:wat::core::lambda ((_ :wat::core::f64) -> :wat::core::bool) true))
