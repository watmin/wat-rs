;; wat/telemetry/journal.wat — arc 278 STONE T1b.2: :wat::telemetry'::journal' — the telemetry sink.
;;
;; A `:wat::service::defservice` that `:satisfies :wat::telemetry'::Journal` and HOLDS a
;; `:wat::query::Store` peer (S4d `:peers` — the dialed backend, given at start), serializing each
;; `Metric`/`Log` -> `StoredRow` -> `Store/put` through that held peer. Mirrors the s2s peer-holding
;; shape proven in tests/services/probe_arc278_s2s_peer_on_{thread,process} (caller' holding echo').
;;
;; The key shapes (proven in tests/services/probe_arc278_tagged_keys_store):
;;   pk   = #wat.telemetry'/PartitionKey {:namespace … :kind …}   (:wat::edn::write of a record)
;;   sk   = #inst "<constant-width iso8601-nanos>"                 (:wat::time::to-iso8601 … 9, tagged)
;;   data = the record's tagged EDN                               (:wat::edn::write metric/log)
;;   index-keys = { "by-uuid" -> IndexKey{ipk=#uuid, isk=sk} }    (the uuid correlation GSI)
;;
;; Write failures are the store's `put` failures, surfaced PASS-THROUGH into the Journal response
;; (the shared :wat::query:: error vocabulary — derive-is-the-wall, NOT a parallel telemetry one).
;;
;; Loads after wat/telemetry.wat (Journal/Metric/Log/PartitionKey/Kind), wat/query.wat (Store), and
;; wat/service.wat (defservice) — see the src/stdlib.rs manifest slot.

;; ── small pure helpers ──────────────────────────────────────────────────────────
;; sk = #inst "<iso8601 with 9 fixed fractional digits, Z>" — CONSTANT WIDTH, so it sorts
;; lexicographically = chronologically (the store's `sort-by Row/sk` is the range order).
(:wat::core::defn :wat::telemetry::time-sk [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::string::concat
    (:wat::string::concat "#inst \"" (:wat::time::to-iso8601 (:wat::time::at-nanos ns) 9))
    "\""))

;; the uuid correlation GSI's index-keys for a scope uuid + the row's sk.
(:wat::core::defn :wat::telemetry::uuid-index-keys
  [uuid <- :wat::core::Uuid  sk <- :wat::core::String]
  -> (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey])
  (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
    "by-uuid" (:wat::query::IndexKey :ipk (:wat::edn::write uuid) :isk sk)))

;; Metric -> StoredRow (pk = namespace+:Metric; sk = #inst; data = the tagged Metric EDN).
(:wat::core::defn :wat::telemetry::metric->row
  [m <- :wat::telemetry::Metric] -> :wat::query::StoredRow
  (:wat::core::let
    [sk (:wat::telemetry::time-sk (:wat::telemetry::Metric/time-ns m))]
    (:wat::query::StoredRow
      :pk (:wat::edn::write (:wat::telemetry::PartitionKey
                              :namespace (:wat::telemetry::Metric/namespace m)
                              :kind :wat::telemetry::Kind::Metric))
      :sk sk
      :data (:wat::edn::write m)
      :index-keys (:wat::telemetry::uuid-index-keys (:wat::telemetry::Metric/uuid m) sk))))

;; Log -> StoredRow (pk = namespace+:Log; sk = #inst; data = the tagged Log EDN).
(:wat::core::defn :wat::telemetry::log->row
  [l <- :wat::telemetry::Log] -> :wat::query::StoredRow
  (:wat::core::let
    [sk (:wat::telemetry::time-sk (:wat::telemetry::Log/time-ns l))]
    (:wat::query::StoredRow
      :pk (:wat::edn::write (:wat::telemetry::PartitionKey
                              :namespace (:wat::telemetry::Log/namespace l)
                              :kind :wat::telemetry::Kind::Log))
      :sk sk
      :data (:wat::edn::write l)
      :index-keys (:wat::telemetry::uuid-index-keys (:wat::telemetry::Log/uuid l) sk))))

;; ── the service ─────────────────────────────────────────────────────────────────
(:wat::service::defservice :wat::telemetry::journal
  :satisfies :wat::telemetry::Journal
  ;; arc 278 Stone 1b — the per-service hard frame limit FOO (bytes-per-read): the journal accepts
  ;; BULK log writes, so it declares 10 MiB (the 512 KiB default would reject a real batch). Threaded
  ;; to this service's accepted-connection receivers; a frame over this → a reasoned 400 + close, not mute.
  :max-frame-bytes 10485760
  :durable   []
  ;; the dialed backend peer — a client (Peer' :- [Store::Op Store::Reply]), held as a ROOT ephemeral field
  :ephemeral [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  ;; the explicit s2s dependency DAG — set-equal to the ephemeral peer field's surface
  :peers     [:wat::query::Store]
  ;; :init connects to the given store (its Address' is a start operating-input, EDN — crosses a fork),
  ;; then ENSURES the store's schema ONCE: the base table (pk, sk) + the by-uuid correlation GSI.
  ;; journal' owns the schema because the store is domain-blind. A no-op on mem-store'; on
  ;; sqlite-store' this CREATEs the table + index, so the later `put`s succeed (mem hid this need).
  :init (:wat::core::fn
          [record     <- :wat::telemetry::journal::Record
           store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
          -> :wat::telemetry::journal::State
          (:wat::core::let
            ;; arc 278 the connect'-outcome wall — face all four arms. ::Connected → bind
            ;; the store Peer'; ::Refused/::Rejected/::Failed → assertion-failed! (fatal,
            ;; preserving the pre-wall raise-unwind: a service whose store dial fails at
            ;; :init cannot start). Sibling pattern: spawn.wat's recv'/send' fatal arms.
            [store (:wat::core::match (:wat::kernel::connect store-addr)
                     ((:wat::kernel::ConnectOutcome::Connected p) p)
                     ((:wat::kernel::ConnectOutcome::Refused c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                     ((:wat::kernel::ConnectOutcome::Rejected c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                     ((:wat::kernel::ConnectOutcome::Failed c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
             _es   (:wat::query::Store/ensure-schema store
                     (:wat::query::Store::EnsureSchemaRequest
                       :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
                       :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                                  (:wat::query::IndexSchema
                                    :name "by-uuid" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))]
            (:wat::telemetry::journal::State :durable record :store store)))
  :impls
  [(write-metrics [s ctx req]
     (:wat::core::let
       [store (:wat::telemetry::journal::State/store s)
        batch (:wat::telemetry::Journal::WriteMetricsRequest/batch req)
        rows  (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::StoredRow])
                                 m   <- :wat::telemetry::Metric]
                  -> (:wat::core::Vector :- [:wat::query::StoredRow])
                  (:wat::core::conj acc (:wat::telemetry::metric->row m)))
                (:wat::core::Vector :- [:wat::query::StoredRow])
                batch)
        put-resp (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
        wresp (:wat::core::match put-resp
                ((:wat::kernel::RecvOutcome::Message sresp)
                  (:wat::core::match sresp
                    ((:wat::query::Store::PutResponse::Success)
                      (:wat::telemetry::Journal::WriteMetricsResponse::Success))
                    ((:wat::query::Store::PutResponse::Constraint err)
                      (:wat::telemetry::Journal::WriteMetricsResponse::Constraint err))
                    ((:wat::query::Store::PutResponse::Transient err)
                      (:wat::telemetry::Journal::WriteMetricsResponse::Transient err))
                    ((:wat::query::Store::PutResponse::Fatal err)
                      (:wat::telemetry::Journal::WriteMetricsResponse::Fatal err))
                    ;; wire-breach at the store peer propagates outward as our own op's breach.
                    ((:wat::query::Store::PutResponse::RequestTooLarge bytes cap)
                      (:wat::telemetry::Journal::WriteMetricsResponse::RequestTooLarge bytes cap))
                    ((:wat::query::Store::PutResponse::RequestMalformed mpath mexpected mgot)
                      (:wat::telemetry::Journal::WriteMetricsResponse::RequestMalformed mpath mexpected mgot))))
                ;; a lost/closed store peer must NOT kill the shared journal service — map to our own
                ;; Fatal response value and KEEP SERVING (the client-triggerable-DoS arc forbids raise).
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::telemetry::Journal::WriteMetricsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
                ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape
                ;; (the operation cannot complete either way) with the TRUE reason: the
                ;; store peer was alive and the substrate was asked to stop.
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::telemetry::Journal::WriteMetricsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: stop requested mid-call — the store peer was ALIVE"))))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::telemetry::Journal::WriteMetricsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: store peer closed")))))]
       (:wat::service::Outcome::Reply s wresp)))

   (write-logs [s ctx req]
     (:wat::core::let
       [store (:wat::telemetry::journal::State/store s)
        batch (:wat::telemetry::Journal::WriteLogsRequest/batch req)
        rows  (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::StoredRow])
                                 l   <- :wat::telemetry::Log]
                  -> (:wat::core::Vector :- [:wat::query::StoredRow])
                  (:wat::core::conj acc (:wat::telemetry::log->row l)))
                (:wat::core::Vector :- [:wat::query::StoredRow])
                batch)
        put-resp (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
        wresp (:wat::core::match put-resp
                ((:wat::kernel::RecvOutcome::Message sresp)
                  (:wat::core::match sresp
                    ((:wat::query::Store::PutResponse::Success)
                      (:wat::telemetry::Journal::WriteLogsResponse::Success))
                    ((:wat::query::Store::PutResponse::Constraint err)
                      (:wat::telemetry::Journal::WriteLogsResponse::Constraint err))
                    ((:wat::query::Store::PutResponse::Transient err)
                      (:wat::telemetry::Journal::WriteLogsResponse::Transient err))
                    ((:wat::query::Store::PutResponse::Fatal err)
                      (:wat::telemetry::Journal::WriteLogsResponse::Fatal err))
                    ;; wire-breach at the store peer propagates outward as our own op's breach.
                    ((:wat::query::Store::PutResponse::RequestTooLarge bytes cap)
                      (:wat::telemetry::Journal::WriteLogsResponse::RequestTooLarge bytes cap))
                    ((:wat::query::Store::PutResponse::RequestMalformed mpath mexpected mgot)
                      (:wat::telemetry::Journal::WriteLogsResponse::RequestMalformed mpath mexpected mgot))))
                ;; a lost/closed store peer must NOT kill the shared journal service — map to our own
                ;; Fatal response value and KEEP SERVING (the client-triggerable-DoS arc forbids raise).
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::telemetry::Journal::WriteLogsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
                ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape
                ;; (the operation cannot complete either way) with the TRUE reason: the
                ;; store peer was alive and the substrate was asked to stop.
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::telemetry::Journal::WriteLogsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: stop requested mid-call — the store peer was ALIVE"))))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::telemetry::Journal::WriteLogsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: store peer closed")))))]
       (:wat::service::Outcome::Reply s wresp)))

   ;; query-metrics — scan the namespace's Metric partition over [time-lo, time-hi], hydrate each
   ;; stored row back to a Metric (:wat::edn::read off the tag), page via cursor. NO rete.
   (query-metrics [s ctx req]
     (:wat::core::let
       [store (:wat::telemetry::journal::State/store s)
        ns   (:wat::telemetry::Journal::QueryMetricsRequest/namespace req)
        lo   (:wat::telemetry::Journal::QueryMetricsRequest/time-lo req)
        hi   (:wat::telemetry::Journal::QueryMetricsRequest/time-hi req)
        lim  (:wat::telemetry::Journal::QueryMetricsRequest/limit req)
        cur  (:wat::telemetry::Journal::QueryMetricsRequest/cursor req)
        pk   (:wat::edn::write (:wat::telemetry::PartitionKey :namespace ns :kind :wat::telemetry::Kind::Metric))
        resp (:wat::query::Store/scan store
               (:wat::query::Store::ScanRequest :pk pk
                 :sk-lo (:wat::telemetry::time-sk lo) :sk-hi (:wat::telemetry::time-sk hi)
                 :limit lim :cursor cur))
        qresp (:wat::core::match resp
                ((:wat::kernel::RecvOutcome::Message sresp)
                  (:wat::core::match sresp
                    ((:wat::query::Store::ScanResponse::Success rows next-cur)
                      (:wat::telemetry::Journal::QueryMetricsResponse::Success
                        (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::telemetry::Metric]) row <- :wat::query::Row]
                            -> (:wat::core::Vector :- [:wat::telemetry::Metric])
                            (:wat::core::conj acc (:wat::edn::read (:wat::query::Row/data row))))
                          (:wat::core::Vector :- [:wat::telemetry::Metric])
                          rows)
                        next-cur))
                    ((:wat::query::Store::ScanResponse::Transient err)
                      (:wat::telemetry::Journal::QueryMetricsResponse::Transient err))
                    ((:wat::query::Store::ScanResponse::Fatal err)
                      (:wat::telemetry::Journal::QueryMetricsResponse::Fatal err))
                    ;; wire-breach at the store peer propagates outward as our own op's breach.
                    ((:wat::query::Store::ScanResponse::RequestTooLarge bytes cap)
                      (:wat::telemetry::Journal::QueryMetricsResponse::RequestTooLarge bytes cap))
                    ((:wat::query::Store::ScanResponse::RequestMalformed mpath mexpected mgot)
                      (:wat::telemetry::Journal::QueryMetricsResponse::RequestMalformed mpath mexpected mgot))))
                ;; a lost/closed store peer must NOT kill the shared journal service — map to our own
                ;; Fatal response value and KEEP SERVING (the client-triggerable-DoS arc forbids raise).
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::telemetry::Journal::QueryMetricsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
                ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape
                ;; (the operation cannot complete either way) with the TRUE reason: the
                ;; store peer was alive and the substrate was asked to stop.
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::telemetry::Journal::QueryMetricsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: stop requested mid-call — the store peer was ALIVE"))))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::telemetry::Journal::QueryMetricsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: store peer closed")))))]
       (:wat::service::Outcome::Reply s qresp)))

   ;; query-logs — the same for the Log partition.
   (query-logs [s ctx req]
     (:wat::core::let
       [store (:wat::telemetry::journal::State/store s)
        ns   (:wat::telemetry::Journal::QueryLogsRequest/namespace req)
        lo   (:wat::telemetry::Journal::QueryLogsRequest/time-lo req)
        hi   (:wat::telemetry::Journal::QueryLogsRequest/time-hi req)
        lim  (:wat::telemetry::Journal::QueryLogsRequest/limit req)
        cur  (:wat::telemetry::Journal::QueryLogsRequest/cursor req)
        pk   (:wat::edn::write (:wat::telemetry::PartitionKey :namespace ns :kind :wat::telemetry::Kind::Log))
        resp (:wat::query::Store/scan store
               (:wat::query::Store::ScanRequest :pk pk
                 :sk-lo (:wat::telemetry::time-sk lo) :sk-hi (:wat::telemetry::time-sk hi)
                 :limit lim :cursor cur))
        qresp (:wat::core::match resp
                ((:wat::kernel::RecvOutcome::Message sresp)
                  (:wat::core::match sresp
                    ((:wat::query::Store::ScanResponse::Success rows next-cur)
                      (:wat::telemetry::Journal::QueryLogsResponse::Success
                        (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::telemetry::Log]) row <- :wat::query::Row]
                            -> (:wat::core::Vector :- [:wat::telemetry::Log])
                            (:wat::core::conj acc (:wat::edn::read (:wat::query::Row/data row))))
                          (:wat::core::Vector :- [:wat::telemetry::Log])
                          rows)
                        next-cur))
                    ((:wat::query::Store::ScanResponse::Transient err)
                      (:wat::telemetry::Journal::QueryLogsResponse::Transient err))
                    ((:wat::query::Store::ScanResponse::Fatal err)
                      (:wat::telemetry::Journal::QueryLogsResponse::Fatal err))
                    ;; wire-breach at the store peer propagates outward as our own op's breach.
                    ((:wat::query::Store::ScanResponse::RequestTooLarge bytes cap)
                      (:wat::telemetry::Journal::QueryLogsResponse::RequestTooLarge bytes cap))
                    ((:wat::query::Store::ScanResponse::RequestMalformed mpath mexpected mgot)
                      (:wat::telemetry::Journal::QueryLogsResponse::RequestMalformed mpath mexpected mgot))))
                ;; a lost/closed store peer must NOT kill the shared journal service — map to our own
                ;; Fatal response value and KEEP SERVING (the client-triggerable-DoS arc forbids raise).
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::telemetry::Journal::QueryLogsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
                ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape
                ;; (the operation cannot complete either way) with the TRUE reason: the
                ;; store peer was alive and the substrate was asked to stop.
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::telemetry::Journal::QueryLogsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: stop requested mid-call — the store peer was ALIVE"))))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::telemetry::Journal::QueryLogsResponse::Fatal
                    (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: store peer closed")))))]
       (:wat::service::Outcome::Reply s qresp)))

   ;; sift-logs — arc 278 Stone 2: query-logs + server-side filtering. The predicate (a `Sieve`'s
   ;; `::`-source) is compiled ONCE (read-string -> unwrap -> verify
   ;; pure?/deterministic?/total? -> eval-ast!), outside the foldl; applied PER ROW
   ;; inside it. An impure/non-deterministic/partial predicate is REJECTED — `::Fatal`
   ;; with a Fault, never a silent pass (no-hidden-failures). `total?` is the third
   ;; language axis (DESIGN-STONE-total-the-third-axis); journal is a consumer beyond rete.
   (sift-logs [s ctx req]
     (:wat::core::let
       [store    (:wat::telemetry::journal::State/store s)
        pred-src (:wat::core::match (:wat::telemetry::Journal::SiftLogsRequest/sieve req) 
                   ((:wat::query::Sieve::Predicate pred) pred))
        pform    (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string pred-src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))
        purep    (:wat::rete::pure? pform)
        detp     (:wat::rete::deterministic? pform)
        totp     (:wat::rete::total? pform)
        qresp    (:wat::core::if (:wat::core::and purep (:wat::core::and detp totp))
                   (:wat::core::let
                     [pfn  (:wat::core::Result/expect (:wat::eval-ast! pform) "sift-logs: eval predicate")
                      ns   (:wat::telemetry::Journal::SiftLogsRequest/namespace req)
                      lo   (:wat::telemetry::Journal::SiftLogsRequest/time-lo req)
                      hi   (:wat::telemetry::Journal::SiftLogsRequest/time-hi req)
                      lim  (:wat::telemetry::Journal::SiftLogsRequest/limit req)
                      cur  (:wat::telemetry::Journal::SiftLogsRequest/cursor req)
                      pk   (:wat::edn::write (:wat::telemetry::PartitionKey :namespace ns :kind :wat::telemetry::Kind::Log))
                      resp (:wat::query::Store/scan store
                             (:wat::query::Store::ScanRequest :pk pk
                               :sk-lo (:wat::telemetry::time-sk lo) :sk-hi (:wat::telemetry::time-sk hi)
                               :limit lim :cursor cur))]
                     (:wat::core::match resp
                       ((:wat::kernel::RecvOutcome::Message sresp)
                         (:wat::core::match sresp
                           ((:wat::query::Store::ScanResponse::Success rows next-cur)
                             (:wat::telemetry::Journal::SiftLogsResponse::Success
                               (:wat::core::foldl
                                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::telemetry::Log]) row <- :wat::query::Row]
                                   -> (:wat::core::Vector :- [:wat::telemetry::Log])
                                   (:wat::core::let [log (:wat::edn::read (:wat::query::Row/data row))]
                                     (:wat::core::if (:wat::core::apply  pfn log [])
                                       (:wat::core::conj acc log)
                                       acc)))
                                 (:wat::core::Vector :- [:wat::telemetry::Log])
                                 rows)
                               next-cur))
                           ((:wat::query::Store::ScanResponse::Transient err)
                             (:wat::telemetry::Journal::SiftLogsResponse::Transient err))
                           ((:wat::query::Store::ScanResponse::Fatal err)
                             (:wat::telemetry::Journal::SiftLogsResponse::Fatal err))
                           ;; wire-breach at the store peer propagates outward as our own op's breach.
                           ((:wat::query::Store::ScanResponse::RequestTooLarge bytes cap)
                             (:wat::telemetry::Journal::SiftLogsResponse::RequestTooLarge bytes cap))
                           ((:wat::query::Store::ScanResponse::RequestMalformed mpath mexpected mgot)
                             (:wat::telemetry::Journal::SiftLogsResponse::RequestMalformed mpath mexpected mgot))))
                       ;; a lost/closed store peer must NOT kill the shared journal service — map to our own
                       ;; Fatal response value and KEEP SERVING (the client-triggerable-DoS arc forbids raise).
                       ((:wat::kernel::RecvOutcome::Lost cause)
                         (:wat::telemetry::Journal::SiftLogsResponse::Fatal
                           (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
                       ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape
                       ;; (the operation cannot complete either way) with the TRUE reason: the
                       ;; store peer was alive and the substrate was asked to stop.
                       (:wat::kernel::RecvOutcome::Stopped
                         (:wat::telemetry::Journal::SiftLogsResponse::Fatal
                           (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: stop requested mid-call — the store peer was ALIVE"))))
                       (:wat::kernel::RecvOutcome::Closed
                         (:wat::telemetry::Journal::SiftLogsResponse::Fatal
                           (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: store peer closed"))))))
                   (:wat::telemetry::Journal::SiftLogsResponse::Fatal
                     (:wat::query::Fatal :reason
                       (:wat::query::Fault :message "sift-logs: predicate must be pure, deterministic, and total"))))]
       (:wat::service::Outcome::Reply s qresp)))

   ;; sift-metrics — the mechanical twin, over the Metric partition.
   (sift-metrics [s ctx req]
     (:wat::core::let
       [store    (:wat::telemetry::journal::State/store s)
        pred-src (:wat::core::match (:wat::telemetry::Journal::SiftMetricsRequest/sieve req) 
                   ((:wat::query::Sieve::Predicate pred) pred))
        pform    (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string pred-src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))
        purep    (:wat::rete::pure? pform)
        detp     (:wat::rete::deterministic? pform)
        totp     (:wat::rete::total? pform)
        qresp    (:wat::core::if (:wat::core::and purep (:wat::core::and detp totp))
                   (:wat::core::let
                     [pfn  (:wat::core::Result/expect (:wat::eval-ast! pform) "sift-metrics: eval predicate")
                      ns   (:wat::telemetry::Journal::SiftMetricsRequest/namespace req)
                      lo   (:wat::telemetry::Journal::SiftMetricsRequest/time-lo req)
                      hi   (:wat::telemetry::Journal::SiftMetricsRequest/time-hi req)
                      lim  (:wat::telemetry::Journal::SiftMetricsRequest/limit req)
                      cur  (:wat::telemetry::Journal::SiftMetricsRequest/cursor req)
                      pk   (:wat::edn::write (:wat::telemetry::PartitionKey :namespace ns :kind :wat::telemetry::Kind::Metric))
                      resp (:wat::query::Store/scan store
                             (:wat::query::Store::ScanRequest :pk pk
                               :sk-lo (:wat::telemetry::time-sk lo) :sk-hi (:wat::telemetry::time-sk hi)
                               :limit lim :cursor cur))]
                     (:wat::core::match resp
                       ((:wat::kernel::RecvOutcome::Message sresp)
                         (:wat::core::match sresp
                           ((:wat::query::Store::ScanResponse::Success rows next-cur)
                             (:wat::telemetry::Journal::SiftMetricsResponse::Success
                               (:wat::core::foldl
                                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::telemetry::Metric]) row <- :wat::query::Row]
                                   -> (:wat::core::Vector :- [:wat::telemetry::Metric])
                                   (:wat::core::let [m (:wat::edn::read (:wat::query::Row/data row))]
                                     (:wat::core::if (:wat::core::apply  pfn m [])
                                       (:wat::core::conj acc m)
                                       acc)))
                                 (:wat::core::Vector :- [:wat::telemetry::Metric])
                                 rows)
                               next-cur))
                           ((:wat::query::Store::ScanResponse::Transient err)
                             (:wat::telemetry::Journal::SiftMetricsResponse::Transient err))
                           ((:wat::query::Store::ScanResponse::Fatal err)
                             (:wat::telemetry::Journal::SiftMetricsResponse::Fatal err))
                           ;; wire-breach at the store peer propagates outward as our own op's breach.
                           ((:wat::query::Store::ScanResponse::RequestTooLarge bytes cap)
                             (:wat::telemetry::Journal::SiftMetricsResponse::RequestTooLarge bytes cap))
                           ((:wat::query::Store::ScanResponse::RequestMalformed mpath mexpected mgot)
                             (:wat::telemetry::Journal::SiftMetricsResponse::RequestMalformed mpath mexpected mgot))))
                       ;; a lost/closed store peer must NOT kill the shared journal service — map to our own
                       ;; Fatal response value and KEEP SERVING (the client-triggerable-DoS arc forbids raise).
                       ((:wat::kernel::RecvOutcome::Lost cause)
                         (:wat::telemetry::Journal::SiftMetricsResponse::Fatal
                           (:wat::query::Fatal :reason (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause)))))
                       ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape
                       ;; (the operation cannot complete either way) with the TRUE reason: the
                       ;; store peer was alive and the substrate was asked to stop.
                       (:wat::kernel::RecvOutcome::Stopped
                         (:wat::telemetry::Journal::SiftMetricsResponse::Fatal
                           (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: stop requested mid-call — the store peer was ALIVE"))))
                       (:wat::kernel::RecvOutcome::Closed
                         (:wat::telemetry::Journal::SiftMetricsResponse::Fatal
                           (:wat::query::Fatal :reason (:wat::query::Fault :message "journal.wat: store peer closed"))))))
                   (:wat::telemetry::Journal::SiftMetricsResponse::Fatal
                     (:wat::query::Fatal :reason
                       (:wat::query::Fault :message "sift-metrics: predicate must be pure, deterministic, and total"))))]
       (:wat::service::Outcome::Reply s qresp)))])
