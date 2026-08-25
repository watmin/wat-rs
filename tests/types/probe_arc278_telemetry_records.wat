;; Arc 278 — telemetry records acceptance probe (RED until wat/telemetry.wat ships :wat::telemetry').
;; main CONSTRUCTS a Metric splicing Scope, then reads a SPLICED field (namespace) and an OWN field (name).
;; Because main runs, the construction is EXERCISED — RED at HEAD (Metric/Scope/Numeric/Unit undefined);
;; GREEN when the records ship, splice expands (Metric/namespace minted), and the unified ctor accepts the
;; 8 positional fields (4 spliced Scope + 4 own). Construction syntax the shadowdancer confirms/adjusts.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [tags (:wat::core::HashMap :wat::core::keyword :wat::core::String)   ;; empty Tags
     ;; Arc 294 item (C) — spliced records are constructed via KWARGS at the bare name
     ;; (order-free; the whole point of surface-splice legibility). Bare-positional is
     ;; retired (positional belongs to the prime `:T'`).
     m    (:wat::telemetry::Metric
            :namespace     "market-eval"                     ;; spliced from Scope
            :uuid          (:wat::uuid::nil)            ;; spliced
            :tags          tags                              ;; spliced
            :time-ns       123                               ;; spliced
            :start-time-ns 100                               ;; own
            :name          :requests                         ;; own
            :value         (:wat::telemetry::Numeric::I64 7) ;; own
            :unit          :wat::telemetry::Unit::Count)]    ;; own (bare unit variant)
    (:wat::kernel::println (:wat::telemetry::Metric/namespace m))   ;; a SPLICED accessor
    (:wat::kernel::println (:wat::telemetry::Metric/name m))))      ;; an OWN accessor
