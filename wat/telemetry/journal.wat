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
(:wat::core::defn :wat::telemetry'::time-sk [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::string::concat
    (:wat::core::string::concat "#inst \"" (:wat::time::to-iso8601 (:wat::time::at-nanos ns) 9))
    "\""))

;; the uuid correlation GSI's index-keys for a scope uuid + the row's sk.
(:wat::core::defn :wat::telemetry'::uuid-index-keys
  [uuid <- :wat::core::Uuid  sk <- :wat::core::String]
  -> (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)
  (:wat::core::HashMap :wat::core::String :wat::query::IndexKey
    "by-uuid" (:wat::query::IndexKey :ipk (:wat::edn::write uuid) :isk sk)))

;; Metric -> StoredRow (pk = namespace+:Metric; sk = #inst; data = the tagged Metric EDN).
(:wat::core::defn :wat::telemetry'::metric->row
  [m <- :wat::telemetry'::Metric] -> :wat::query::StoredRow
  (:wat::core::let
    [sk (:wat::telemetry'::time-sk (:wat::telemetry'::Metric/time-ns m))]
    (:wat::query::StoredRow
      :pk (:wat::edn::write (:wat::telemetry'::PartitionKey
                              :namespace (:wat::telemetry'::Metric/namespace m)
                              :kind :wat::telemetry'::Kind::Metric))
      :sk sk
      :data (:wat::edn::write m)
      :index-keys (:wat::telemetry'::uuid-index-keys (:wat::telemetry'::Metric/uuid m) sk))))

;; Log -> StoredRow (pk = namespace+:Log; sk = #inst; data = the tagged Log EDN).
(:wat::core::defn :wat::telemetry'::log->row
  [l <- :wat::telemetry'::Log] -> :wat::query::StoredRow
  (:wat::core::let
    [sk (:wat::telemetry'::time-sk (:wat::telemetry'::Log/time-ns l))]
    (:wat::query::StoredRow
      :pk (:wat::edn::write (:wat::telemetry'::PartitionKey
                              :namespace (:wat::telemetry'::Log/namespace l)
                              :kind :wat::telemetry'::Kind::Log))
      :sk sk
      :data (:wat::edn::write l)
      :index-keys (:wat::telemetry'::uuid-index-keys (:wat::telemetry'::Log/uuid l) sk))))

;; ── the service ─────────────────────────────────────────────────────────────────
(:wat::service::defservice :wat::telemetry'::journal'
  :satisfies :wat::telemetry'::Journal
  :durable   []
  ;; the dialed backend peer — a client Peer'<Store::Op,Store::Reply>, held as a ROOT ephemeral field
  :ephemeral [store <- :wat::kernel::Peer'<wat::query::Store::Op,wat::query::Store::Reply>]
  ;; the explicit s2s dependency DAG — set-equal to the ephemeral peer field's surface
  :peers     [:wat::query::Store]
  ;; :init connects to the given store (its Address' is a start operating-input, EDN — crosses a fork),
  ;; then ENSURES the store's schema ONCE: the base table (pk, sk) + the by-uuid correlation GSI.
  ;; journal' owns the schema because the store is domain-blind. A no-op on mem-store'; on
  ;; sqlite-store' this CREATEs the table + index, so the later `put`s succeed (mem hid this need).
  :init (:wat::core::fn
          [record     <- :wat::telemetry'::journal'::Record
           store-addr <- :wat::kernel::Address'<wat::query::Store::Op,wat::query::Store::Reply>]
          -> :wat::telemetry'::journal'::State
          (:wat::core::let
            [store (:wat::kernel::connect' store-addr)
             _es   (:wat::query::Store/ensure-schema store
                     (:wat::query::Store::EnsureSchemaRequest
                       :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
                       :indexes (:wat::core::Vector :wat::query::IndexSchema
                                  (:wat::query::IndexSchema
                                    :name "by-uuid" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))]
            (:wat::telemetry'::journal'::State :durable record :store store)))
  :impls
  [(write-metrics [s req]
     (:wat::core::let
       [store (:wat::telemetry'::journal'::State/store s)
        batch (:wat::telemetry'::Journal::WriteMetricsRequest/batch req)
        rows  (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :wat::query::StoredRow)
                                 m   <- :wat::telemetry'::Metric]
                  -> (:wat::core::Vector :wat::query::StoredRow)
                  (:wat::core::conj acc (:wat::telemetry'::metric->row m)))
                (:wat::core::Vector :wat::query::StoredRow)
                batch)
        put-resp (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
        wresp (:wat::core::match put-resp -> :wat::telemetry'::Journal::WriteMetricsResponse
                ((:wat::query::Store::PutResponse::Success)
                  (:wat::telemetry'::Journal::WriteMetricsResponse::Success))
                ((:wat::query::Store::PutResponse::Constraint err)
                  (:wat::telemetry'::Journal::WriteMetricsResponse::Constraint err))
                ((:wat::query::Store::PutResponse::Transient err)
                  (:wat::telemetry'::Journal::WriteMetricsResponse::Transient err))
                ((:wat::query::Store::PutResponse::Fatal err)
                  (:wat::telemetry'::Journal::WriteMetricsResponse::Fatal err)))]
       (:wat::service::Outcome::Reply s wresp)))

   (write-logs [s req]
     (:wat::core::let
       [store (:wat::telemetry'::journal'::State/store s)
        batch (:wat::telemetry'::Journal::WriteLogsRequest/batch req)
        rows  (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :wat::query::StoredRow)
                                 l   <- :wat::telemetry'::Log]
                  -> (:wat::core::Vector :wat::query::StoredRow)
                  (:wat::core::conj acc (:wat::telemetry'::log->row l)))
                (:wat::core::Vector :wat::query::StoredRow)
                batch)
        put-resp (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
        wresp (:wat::core::match put-resp -> :wat::telemetry'::Journal::WriteLogsResponse
                ((:wat::query::Store::PutResponse::Success)
                  (:wat::telemetry'::Journal::WriteLogsResponse::Success))
                ((:wat::query::Store::PutResponse::Constraint err)
                  (:wat::telemetry'::Journal::WriteLogsResponse::Constraint err))
                ((:wat::query::Store::PutResponse::Transient err)
                  (:wat::telemetry'::Journal::WriteLogsResponse::Transient err))
                ((:wat::query::Store::PutResponse::Fatal err)
                  (:wat::telemetry'::Journal::WriteLogsResponse::Fatal err)))]
       (:wat::service::Outcome::Reply s wresp)))])
