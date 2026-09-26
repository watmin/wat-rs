;; Stone 255.44 — a store whose put fails. journal' maps that onto
;; WriteLogsResponse, and span' must report the same arm. Pre-stone the
;; span binds the write as _w and replies LogResponse.Ok.
;;
;; The store handle and the journal handle stay in outer lets. A
;; tail-position start drops the callee before the dial.

(:wat::service::defservice :probe::modestore
  :satisfies :wat::query::Store
  :durable [mode <- :wat::core::String
            rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])]
  :impls
  [(ensure-schema [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:wat::query::Store::EnsureSchemaResponse.Success {})}))
   (put [s ctx req]
     (:wat::core::let
       [mode (:probe::modestore::Record/mode (:probe::modestore::State/durable s))
        reply (:wat::core::cond
                ((:wat::core::= mode "ok")
                  (:wat::query::Store::PutResponse.Success {}))
                ((:wat::core::= mode "fatal")
                  (:wat::query::Store::PutResponse.Fatal
                    {:err (:wat::query::Fatal
                            :reason (:wat::query::Fault :message "SPAN-SINK-FATAL"))}))
                ((:wat::core::= mode "constraint")
                  (:wat::query::Store::PutResponse.Constraint
                    {:err (:wat::query::Constraint
                            :reason (:wat::query::Fault :message "SPAN-SINK-CONSTRAINT"))}))
                ((:wat::core::= mode "transient")
                  (:wat::query::Store::PutResponse.Transient
                    {:err (:wat::query::Transient
                            :reason (:wat::query::Fault :message "SPAN-SINK-TRANSIENT"))}))
                (:else
                  (:wat::kernel::assertion-failed!
                    :message (:wat::string::concat "modestore: unknown mode " mode))))]
       (:wat::service::Outcome.Reply {:state s :reply reply})))
   (scan [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:wat::query::Store::ScanResponse.Success
                 {:rows (:wat::core::Vector :- [:wat::query::Row])
                  :cursor :wat::core::Option.None})}))
   (scan-index [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:wat::query::Store::ScanIndexResponse.Success
                 {:rows (:wat::core::Vector :- [:wat::query::IndexRow])
                  :cursor :wat::core::Option.None})}))])

(:wat::core::defn :user::log-through
  [mode <- :wat::core::String]
  -> :wat::telemetry::Span::LogResponse
  (:wat::core::let
    [sh (:probe::modestore/start :locus (:wat::spawn::thread)
          :record (:probe::modestore::Record
                    :mode mode
                    :rows (:wat::core::PersistentVector)))]
    (:wat::core::let
      [jh (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
            :record (:wat::telemetry::journal::Record)
            :store-addr (:probe::modestore::Handle/addr sh))]
      (:wat::core::let
        [sph (:wat::telemetry::span/start :locus (:wat::spawn::thread)
               :record (:wat::telemetry::span::Record
                         :namespace "probe"
                         :uuid (:wat::uuid::v4)
                         :tags (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
                         :start-time-ns 0
                         :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                         :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples]))
               :sink-addr (:wat::telemetry::journal::Handle/addr jh))]
        (:wat::core::let
          [span (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph))
                  [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
                  [:wat::kernel::ConnectOutcome.Closed {:cause c}
                    (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
                  [:wat::kernel::ConnectOutcome.Undialable {:cause c}
                    (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
                  [:wat::kernel::ConnectOutcome.WrongPeer {:cause c}
                    (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
                  [:wat::kernel::ConnectOutcome.Failed {:cause c}
                    (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
           recv (:wat::telemetry::Span/log span
                  (:wat::telemetry::Span::LogRequest
                    :emitted-from (:wat::kernel::call-site)
                    :level :wat::telemetry::Level.Info
                    :message "probe"))]
          (:wat::core::match recv
            [:wat::kernel::RecvOutcome.Message {:msg m} m]
            [:wat::kernel::RecvOutcome.Lost {:cause cause}
              (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
            [:wat::kernel::RecvOutcome.Stopped {}
              (:wat::kernel::assertion-failed! :message "span log: stopped")]
            [:wat::kernel::RecvOutcome.Closed {}
              (:wat::kernel::assertion-failed! :message "span log: peer closed")]))))))
