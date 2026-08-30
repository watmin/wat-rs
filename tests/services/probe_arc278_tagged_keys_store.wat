;; Co-located fixture for probe_arc278_tagged_keys_store.rs — arc 278 T1b.2 key-design gate.
;;
;; Proves the `:wat::query::Store` (via the baked mem-store') round-trips + indexes correctly with
;; the TAGGED-EDN key shapes journal' will produce (isolated from journal' itself):
;;   pk  = #wat.telemetry'/PartitionKey {:namespace … :kind …}  (:wat::edn::write of a record)
;;   sk  = #inst "<constant-width iso8601-nanos>"               (:wat::time::to-iso8601 … 9)
;;   gsi = #uuid "8-4-4-4-12"                                   (:wat::edn::write of a Uuid)
;;
;; The load-bearing property: a CONSTANT-WIDTH #inst sk sorts lexicographically = chronologically,
;; so mem-store''s `sort-by Row/sk` returns rows in time order — including a second-boundary instant
;; (the case the generic EDN writer's variable-width AutoSi render would sort WRONG).

;; ── key builders ─────────────────────────────────────────────────────────────────
(:wat::core::defn :user::pk [] -> :wat::core::String
  (:wat::edn::write
    (:wat::telemetry::PartitionKey :namespace "some-ns" :kind :wat::telemetry::Kind::Metric)))

;; sk = #inst "<iso8601 with 9 fixed fractional digits, Z>" — constant width, sort-safe.
(:wat::core::defn :user::mk-sk [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::string::concat
    (:wat::string::concat "#inst \"" (:wat::time::to-iso8601 (:wat::time::at-nanos ns) 9))
    "\""))

(:wat::core::defn :user::uuid-edn [s <- :wat::core::String] -> :wat::core::String
  (:wat::edn::write (:wat::uuid::from-string s)))

;; ── TEST A — a constant-width #inst sk sorts chronologically ─────────────────────
;; Put three rows OUT OF ORDER (late, early, mid) on one tagged pk; scan; return the returned
;; sks joined by "|". Two are on second-boundaries (…01.0 and …02.0) and one is 1ns after early
;; (…01.000000001) — so a wrong (variable-width) render would misorder the boundary vs the sub-second.
(:wat::core::defn :user::scan-order [] -> :wat::core::String
  (:wat::core::let
    [h     (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     store (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:user::pk)
     u1    (:user::uuid-edn "11111111-1111-4111-8111-111111111111")
     sk-late  (:user::mk-sk 2000000000)   ;; 1970-01-01T00:00:02.000000000Z (boundary)
     sk-early (:user::mk-sk 1000000000)   ;; 1970-01-01T00:00:01.000000000Z (boundary)
     sk-mid   (:user::mk-sk 1000000001)   ;; 1970-01-01T00:00:01.000000001Z (1ns after early)
     ik    (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
             "by-uuid" (:wat::query::IndexKey :ipk u1 :isk sk-early))
     rows  (:wat::core::Vector :- [:wat::query::StoredRow]
             (:wat::query::StoredRow :pk pk :sk sk-late  :data "{:v 3}" :index-keys ik)
             (:wat::query::StoredRow :pk pk :sk sk-early :data "{:v 1}" :index-keys ik)
             (:wat::query::StoredRow :pk pk :sk sk-mid   :data "{:v 2}" :index-keys ik))
     _put  (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
     resp  (:wat::query::Store/scan store
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success out _cursor)
        ;; return the scanned sks as an EDN vector (ORDERED) — the .rs golden-compares it.
        (:wat::edn::write
          (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) r <- :wat::query::Row]
              -> (:wat::core::Vector :- [:wat::core::String])
              (:wat::core::conj acc (:wat::query::Row/sk r)))
            (:wat::core::Vector :- [:wat::core::String])
            out)))
      (_ "SCAN-FAILED"))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; ── TEST B — a #uuid GSI scan-index round-trips ──────────────────────────────────
;; Three rows: two share uuid u1, one has u2. scan-index by u1 must return exactly 2.
(:wat::core::defn :user::index-count [] -> :wat::core::i64
  (:wat::core::let
    [h     (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     store (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:user::pk)
     u1    (:user::uuid-edn "11111111-1111-4111-8111-111111111111")
     u2    (:user::uuid-edn "22222222-2222-4222-8222-222222222222")
     ska   (:user::mk-sk 1000000000)
     skb   (:user::mk-sk 1000000001)
     skc   (:user::mk-sk 2000000000)
     ik1a  (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
             "by-uuid" (:wat::query::IndexKey :ipk u1 :isk ska))
     ik1b  (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
             "by-uuid" (:wat::query::IndexKey :ipk u1 :isk skb))
     ik2   (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
             "by-uuid" (:wat::query::IndexKey :ipk u2 :isk skc))
     rows  (:wat::core::Vector :- [:wat::query::StoredRow]
             (:wat::query::StoredRow :pk pk :sk ska :data "{:v 1}" :index-keys ik1a)
             (:wat::query::StoredRow :pk pk :sk skb :data "{:v 2}" :index-keys ik1b)
             (:wat::query::StoredRow :pk pk :sk skc :data "{:v 3}" :index-keys ik2))
     _put  (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
     resp  (:wat::query::Store/scan-index store
             (:wat::query::Store::ScanIndexRequest
               :index "by-uuid" :ipk u1 :isk-lo "#" :isk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanIndexResponse::Success out _cursor) (:wat::core::count out))
      (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
