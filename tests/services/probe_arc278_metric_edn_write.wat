;; Co-located fixture for probe_arc278_metric_edn_write.rs — arc 278 T1b.2 groundwork.
;;
;; KNOWN-UNKNOWN #2: journal''s write path must serialize a live `Metric` record into
;; `StoredRow.data` (a `:wat::core::String` of the record's tagged EDN — arc-300 records-are-EDN).
;; The candidate verb is `:wat::edn::write` (src/edn/render.rs eval_edn_write → value_to_edn_with →
;; wat_edn::write). This probe DISCONFIRMS the assumption that it accepts an arbitrary record:
;; build a `Metric` (kwargs ctor, mirroring probe_arc278_journal_surface.wat) and encode it.
;;
;; If this type-checks + runs, journal''s serialize step is `(:wat::edn::write metric)`; the .rs
;; asserts the produced string carries the Metric tag + its field values.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [tags (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     m    (:wat::telemetry::Metric
            :namespace     "probe-ns"
            :uuid          (:wat::uuid::nil)
            :tags          tags
            :time-ns       123
            :start-time-ns 100
            :name          :requests
            :value         (:wat::telemetry::Numeric::I64 7)
            :unit          :wat::telemetry::Unit::Count)]
    (:wat::edn::write m)))
