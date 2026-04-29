;; :wat::measure::Event — the substrate-defined enum that
;; WorkUnit/scope ships at scope-close and consumers' Service
;; sinks consume.
;;
;; Two variants, both flat-field (per arc 085's auto-dispatch
;; constraint that variant fields must be primitives or :wat::edn::*
;; newtypes — not nested structs):
;;
;; - Metric — one row per data point. CloudWatch model: a counter
;;   that ends at 7 emits ONE row (metric-value = leaf 7); a
;;   duration timed N times emits N rows (one per sample). Aggregation
;;   (SUM/AVG/PERCENTILE) happens at query time in arc 093's
;;   WorkQuery, the same shape Prometheus + CloudWatch use.
;;
;; - Log — one row per WorkUnit/info / warn / error / debug call.
;;   Slice 5 ships the emission primitives; this slice 4 just
;;   declares the variant.
;;
;; Indexed columns (namespace, metric-name) and the `tags` map
;; are :wat::edn::NoTag — render as bare EDN (`:metrics`,
;; `{:asset :BTC, ...}`) for SQL grep / pattern queries.
;; The `data` field on Log is :wat::edn::Tagged — round-trip-safe
;; so log entries can be parsed back into HolonAST and pattern-matched.
;;
;; Lab-side consumers (arc 091 slice 6) will instantiate
;; `Service<:wat::measure::Event,_>` and pass the result handles
;; to `WorkUnit/scope`. `Sqlite/auto-spawn` over Event derives a
;; two-table schema (one per variant) via arc 085's auto-dispatch.
;;
;; Arc 091 slice 4.

(:wat::core::enum :wat::measure::Event
  (Metric
    (start-time-ns :i64)              ; wu start (wall-clock epoch ns)
    (end-time-ns   :i64)              ; wu end
    (namespace     :wat::edn::NoTag)  ; producing fn's fqdn keyword
    (uuid          :String)           ; from the WorkUnit
    (tags          :wat::measure::Tags) ; HolonAST→HolonAST map
    (metric-name   :wat::edn::NoTag)  ; the counter/duration key
    (metric-value  :wat::edn::NoTag)  ; primitive HolonAST leaf — never a Bundle
    (metric-unit   :wat::edn::NoTag)) ; :count, :seconds, etc.
  (Log
    (time-ns   :i64)                   ; emit moment (wall-clock epoch ns)
    (namespace :wat::edn::NoTag)       ; producing fn's fqdn keyword
    (caller    :wat::edn::NoTag)       ; producer identity
    (level     :wat::edn::NoTag)       ; :info/:warn/:error/:debug
    (uuid      :String)                ; from the WorkUnit
    (tags      :wat::measure::Tags)    ; same map, attached to every log line
    (data      :wat::edn::Tagged)))    ; round-trip-safe message HolonAST
