;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Filter)
;;
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
;; Currently shipped: three filter factories for Hologram/get. These
;; are verbs, not a Filter type — hence they live here rather than in
;; a dedicated Filter.wat file whose basename would misrepresent what
;; it houses.
;;
;; ─── Hologram/get filter factories ──────────────────────────────────
;;
;; `Hologram/get` takes a 2-arg call `(:wat::holon::Hologram/get store probe)`.
;; The filter `:wat::core::Fn(:wat::core::f64) -> :wat::core::bool` is bound
;; at construction via `Hologram/make` and decides whether the
;; highest-cosine candidate is "close enough" to return. The substrate
;; ships three opinionated factories so consumers don't have to
;; hand-roll the canonical thresholds.
;;
;; Usage:
;;
;;   ;; build the store once — filter is bound at construction
;;   (def store (:wat::holon::Hologram/make (:wat::holon::filter-coincident)))
;;
;;   ;; strict — only return when cosine clears the coincident floor
;;   (:wat::holon::Hologram/get store probe)
;;
;;   ;; looser store — build with filter-present instead
;;   (def store (:wat::holon::Hologram/make (:wat::holon::filter-present)))
;;   (:wat::holon::Hologram/get store probe)
;;
;;   ;; pure population readout — no gating; whatever scored highest wins
;;   (def store (:wat::holon::Hologram/make (:wat::holon::filter-accept-any)))
;;   (:wat::holon::Hologram/get store probe)
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
;; d is read from the ambient `:wat::config::dim-count` rather than
;; passed by the caller. The filter captures the floor at the call
;; site's ambient d; pass through `Hologram/make` once and the entire
;; store carries the same threshold.
(:wat::core::defn :wat::holon::filter-coincident [] -> :wat::core::Fn(wat::core::f64)->wat::core::bool
  (:wat::core::let
      [floor
        (:wat::holon::coincident-floor (:wat::config::dim-count))]
      (:wat::core::fn [cos <- :wat::core::f64] -> :wat::core::bool
        (:wat::core::< (:wat::core::- 1.0 cos) floor))))

;; ─── filter-present — looser, "signal detected above noise" ───────
;;
;; Returns true iff `cos > presence-floor(d)`. Matches the semantics
;; of `:wat::holon::presence?` but works on a raw cosine value.
;; Use when the cache is acting as a "best-known reasonable answer"
;; lookup rather than "did I see this exact form before."
;;
;; d is read from the ambient `:wat::config::dim-count`.
(:wat::core::defn :wat::holon::filter-present [] -> :wat::core::Fn(wat::core::f64)->wat::core::bool
  (:wat::core::let
      [floor
        (:wat::holon::presence-floor (:wat::config::dim-count))]
      (:wat::core::fn [cos <- :wat::core::f64] -> :wat::core::bool
        (:wat::core::> cos floor))))

;; ─── filter-accept-any — null gate, returns whatever scored best ──
;;
;; Returns true unconditionally. Useful when the consumer wants the
;; population's nearest neighbor without any floor — e.g., taking the
;; cell's argmax for a soft scoring loop where the consumer applies
;; their own gate downstream.
(:wat::core::defn :wat::holon::filter-accept-any [] -> :wat::core::Fn(wat::core::f64)->wat::core::bool (:wat::core::fn [_ <- :wat::core::f64] -> :wat::core::bool true))
