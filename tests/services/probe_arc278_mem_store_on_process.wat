;; Co-located fixture for probe_arc278_mem_store_on_process.rs.
;;
;; THE MISSING GUARD: a RESERVED-namespace (:wat::query::) first-party service —
;; :wat::query::mem-store' — round-tripping on a FORKED (process) locus. Every prior
;; process-locus service test uses a USER namespace (:my::/:probe::), which cannot trip
;; the reserved-prefix gate; so a reserved-ns service crossing a fork was never guarded,
;; and the arc-294 kwargs companion-defmacro turned that latent gap into a hard
;; StartupError. This test locks the class shut.
;;
;; Identical in shape to tests/rete/probe_arc278_smem_roundtrip.wat EXCEPT the locus is
;; (process) instead of (thread). mem-store' + Store + its messages are all baked into
;; every child (src/stdlib.rs), so the child needs NOTHING re-shipped; the reserved-prefix
;; gate must let the child's benign re-declaration of those baked forms through (the
;; idempotent-before-gate reorder).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h          (:wat::query::mem-store/start :locus (:wat::spawn::process)
                  :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     store      (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     empty-ik   (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey])
     ik-a       (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                  "by-v" (:wat::query::IndexKey :ipk "u#1" :isk "v1"))
     input-rows (:wat::core::Vector :- [:wat::query::StoredRow]
                  (:wat::query::StoredRow :pk "u#1" :sk "a" :data "{:v 1}" :index-keys ik-a)
                  (:wat::query::StoredRow :pk "u#1" :sk "b" :data "{:v 2}" :index-keys empty-ik))
     _put       (:wat::query::Store/put store (:wat::query::Store::PutRequest input-rows))
     scan-resp  (:wat::query::Store/scan store
                  (:wat::query::Store::ScanRequest
                    :pk "u#1" :sk-lo "a" :sk-hi "z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match scan-resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows _cursor) (:wat::core::count rows))
      (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
