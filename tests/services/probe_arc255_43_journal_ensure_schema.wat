;; Stone 255.43 — a store that refuses schema setup. journal' :init must
;; face that RecvOutcome. Pre-stone the result is bound as _es and dropped,
;; and /start succeeds. Post-stone /start raises the schema failure.
;;
;; The store handle is bound in the OUTER let so it stays alive while
;; journal' dials it. A tail-position start would drop the handle first.
(:wat::service::defservice :probe::badstore
  :satisfies :wat::query::Store
  :durable [rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])]
  :impls
  [(ensure-schema [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:wat::query::Store::EnsureSchemaResponse.Fatal
                 {:err (:wat::query::Fatal
                         :reason (:wat::query::Fault :message "SCHEMA-SETUP-FAILED"))})}))
   (put [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:wat::query::Store::PutResponse.Fatal
                 {:err (:wat::query::Fatal
                         :reason (:wat::query::Fault :message "SCHEMA-SETUP-FAILED"))})}))
   (scan [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:wat::query::Store::ScanResponse.Fatal
                 {:err (:wat::query::Fatal
                         :reason (:wat::query::Fault :message "SCHEMA-SETUP-FAILED"))})}))
   (scan-index [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:wat::query::Store::ScanIndexResponse.Fatal
                 {:err (:wat::query::Fatal
                         :reason (:wat::query::Fault :message "SCHEMA-SETUP-FAILED"))})}))])

(:wat::core::defn :user::compute [] -> :wat::telemetry::journal::Handle
  (:wat::core::let
    [sh (:probe::badstore/start :locus (:wat::spawn::thread)
          :record (:probe::badstore::Record :rows (:wat::core::PersistentVector)))]
    (:wat::core::let
      [jh (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
            :record (:wat::telemetry::journal::Record)
            :store-addr (:probe::badstore::Handle/addr sh))]
      jh)))
