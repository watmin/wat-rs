;; Arc 278 — telemetry records acceptance probe (RED until wat/telemetry.wat ships :wat::telemetry').
;; main CONSTRUCTS a Metric splicing Scope, then reads a SPLICED field (namespace) and an OWN field (name).
;; Because main runs, the construction is EXERCISED — RED at HEAD (Metric/Scope/Numeric/Unit undefined);
;; GREEN when the records ship, splice expands (Metric/namespace minted), and the unified ctor accepts the
;; 8 positional fields (4 spliced Scope + 4 own). Construction syntax the shadowdancer confirms/adjusts.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [tags (:wat::core::HashMap :wat::core::keyword :wat::core::String)   ;; empty Tags
     m    (:wat::telemetry'::Metric
            "market-eval"                          ;; namespace  (spliced from Scope)
            (:wat::core::Uuid/nil)                 ;; uuid       (spliced)
            tags                                   ;; tags       (spliced)
            123                                    ;; time-ns    (spliced)
            100                                    ;; start-time-ns (own)
            :requests                              ;; name       (own)
            (:wat::telemetry'::Numeric::I64 7)     ;; value      (own)
            :wat::telemetry'::Unit::Count)]        ;; unit       (own, bare unit variant)
    (:wat::kernel::println (:wat::telemetry'::Metric/namespace m))   ;; a SPLICED accessor
    (:wat::kernel::println (:wat::telemetry'::Metric/name m))))      ;; an OWN accessor
