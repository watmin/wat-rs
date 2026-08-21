;; tests/services/probe_arc278_sift_arena.wat — arc 278 sift-arena Part B: the two-universe
;; flood-and-sift, PROCESS-tier, guaranteed foreign. Co-located fixture for the sibling .rs,
;; slurped via startup_beside(file!()).
;;
;; Proves the sift Predicate earns its keep on the real use case: a PRODUCER service floods a
;; shared journal with records of ITS OWN universe (types defined in its `:messages` — arbitrary
;; domain records, not the Op/Reply shape); a CONSUMER service — which never `:peers` the
;; Producer, never `:satisfies` anything that would compile `:prod::*` — pages through the
;; journal via the FOREIGN READER (`read-foreign` + `ForeignRecord/class`/`get`) and returns the
;; EXACT survivor count. The consumer's inability to typed-decode `:prod::*` is a PROCESS
;; property (separate registries per fork) — this is why the whole arena runs on PROCESS.
;;
;; Architecture (mirrors tests/services/probe_arc278_s2s_peer_on_process.wat's peer-holding shape
;; + tests/services/probe_arc278_journal_service_sqlite_on_process.wat's grant-before-dial chain):
;;   mem-store' (process) <-- grant-before-dial -- journal' (process) <-- grant-before-dial --
;;     { producer' (process), consumer' (process) }
;; producer' floods N=240 Logs cycling 4 shapes (mod i 4 -> Alert-high/Alert-low/Flow/Query) into
;; the journal; consumer' pages `Journal/sift-logs` (limit 50/page) with a class-guarded
;; read-foreign predicate, accumulating the survivor count until the cursor is exhausted.

;; ── PRODUCER: :prod::Producer — carries its OWN log-payload universe (Alert/Flow/Query, no
;; forced Op/Reply shape) plus the flood op pair. ──
(:wat::core::defsurface :prod::Producer :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :prod::Alert [severity <- :wat::core::String  code  <- :wat::core::i64])
   (:wat::core::defrecord :prod::Flow  [proto    <- :wat::core::String  bytes <- :wat::core::i64])
   (:wat::core::defrecord :prod::Query [rows     <- :wat::core::i64])
   (:wat::core::defrecord :prod::Producer::FloodRequest
     [count     <- :wat::core::i64
      namespace <- :wat::core::String])
   (:wat::core::defenum :prod::Producer::FloodResponse :wat::enum::Pure
     :Done            [written <- :wat::core::i64]
     :RequestTooLarge [bytes   <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(flood [self <- :prod::Producer  req <- :prod::Producer::FloodRequest] -> :prod::Producer::FloodResponse :max-request-bytes 524288)])

(:wat::service::defservice :prod::producer
  :satisfies :prod::Producer
  :durable   []
  ;; the dialed backend peer — a client Peer'<Journal::Op,Journal::Reply>, held as a ROOT
  ;; ephemeral field. NEVER an ephemeral/peer to anything Consumer-shaped — the producer only
  ;; ever talks to the shared journal.
  :ephemeral [journal <- :wat::kernel::Peer<wat::telemetry::Journal::Op,wat::telemetry::Journal::Reply>]
  :peers     [:wat::telemetry::Journal]
  :init (:wat::core::fn
          [record       <- :prod::producer::Record
           journal-addr <- :wat::kernel::Address<wat::telemetry::Journal::Op,wat::telemetry::Journal::Reply>]
          -> :prod::producer::State
          (:prod::producer::State :durable record :journal (:wat::core::match (:wat::kernel::connect journal-addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [(flood [s ctx req]
     (:wat::core::let
       [count    (:prod::Producer::FloodRequest/count req)
        ns       (:prod::Producer::FloodRequest/namespace req)
        tags     (:wat::core::HashMap :wat::core::keyword :wat::core::String)
        idxs     (:wat::core::range 0 count)
        logs     (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
                   (:wat::core::map
                     (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log
                       (:wat::core::let
                         [shp (:wat::core::mod i 4)
                          msg (:wat::core::if (:wat::core::= shp 0)
                                (:wat::edn::write (:prod::Alert :severity "high" :code i))
                                (:wat::core::if (:wat::core::= shp 1)
                                  (:wat::edn::write (:prod::Alert :severity "low" :code i))
                                  (:wat::core::if (:wat::core::= shp 2)
                                    (:wat::edn::write (:prod::Flow :proto "tcp" :bytes i))
                                    (:wat::edn::write (:prod::Query :rows i)))))]
                         (:wat::telemetry::Log :namespace ns :uuid (:wat::core::Uuid/nil) :tags tags
                           :time-ns (:wat::core::i64::+ i 1) :emitted-from (:wat::kernel::call-site)
                           :level :wat::telemetry::Level::Info :message msg)))
                     idxs))
        journal  (:prod::producer::State/journal s)
        _wr      (:wat::telemetry::Journal/write-logs journal
                   (:wat::telemetry::Journal::WriteLogsRequest logs))]
       (:wat::service::Outcome::Reply s (:prod::Producer::FloodResponse::Done count))))])

;; ── CONSUMER: :cons::Consumer — NEVER peers/satisfies anything Producer-shaped and NEVER
;; defines `:prod::*`. Its inability to typed-decode `:prod::*` IS the guarantee: it pages the
;; journal via the FOREIGN READER only. ──
;; NOTE — a real finding (STOP-1-adjacent): a forked service child does NOT inherit the whole
;; co-located fixture file; it re-boots on the STATIC stdlib PLUS exactly what `service-forms`
;; ships (the satisfied surface's `:messages` + every `:peers` surface's `:messages` + this
;; defservice's own internals — wat/service.wat:1126-1170). A plain top-level `defn` living
;; elsewhere in this fixture (e.g. a `:cons::sift-loop` helper) is NOT in that set, so a forked
;; consumer' child raises `UnresolvedReference` calling it. Fix: fold `PageState` into Consumer's
;; OWN `:messages` (so it ships with the surface) and implement the cursor-loop as a BOUNDED
;; `:wat::core::foldl` inline inside the `sift` closure (no separate top-level defn needed) — an
;; already-`done` state is a no-op passthrough for remaining iterations, so this is exactly the
;; TCO cursor-loop's observable behavior (page until exhausted) without requiring a helper defn to
;; cross the fork. 8 iterations of `:limit 50` covers up to 400 rows — comfortably >= the 240 the
;; arena floods.
(:wat::core::defsurface :cons::Consumer :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :cons::Consumer::SiftRequest [namespace <- :wat::core::String])
   (:wat::core::defenum :cons::Consumer::SiftResponse :wat::enum::Pure
     :Count           [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :cons::Consumer::PageState
     [done <- :wat::core::bool
      cur  <- (:wat::core::Option :wat::core::String)
      acc  <- :wat::core::i64])]
  :features
  [(sift [self <- :cons::Consumer  req <- :cons::Consumer::SiftRequest] -> :cons::Consumer::SiftResponse :max-request-bytes 524288)])

(:wat::service::defservice :cons::consumer
  :satisfies :cons::Consumer
  :durable   []
  :ephemeral [journal <- :wat::kernel::Peer<wat::telemetry::Journal::Op,wat::telemetry::Journal::Reply>]
  ;; ONLY the Journal — never Producer. This IS the guarantee: consumer''s child registry never
  ;; compiles `:prod::*`.
  :peers     [:wat::telemetry::Journal]
  :init (:wat::core::fn
          [record       <- :cons::consumer::Record
           journal-addr <- :wat::kernel::Address<wat::telemetry::Journal::Op,wat::telemetry::Journal::Reply>]
          -> :cons::consumer::State
          (:cons::consumer::State :durable record :journal (:wat::core::match (:wat::kernel::connect journal-addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [(sift [s ctx req]
     (:wat::core::let
       [journal    (:cons::consumer::State/journal s)
        ns         (:cons::Consumer::SiftRequest/namespace req)
        page-idxs  (:wat::core::range 0 8)
        initial    (:cons::Consumer::PageState :done false :cur :wat::core::None :acc 0)
        ;; the cursor-loop: pages `Journal/sift-logs` (small :limit) accumulating survivor count
        ;; until `next-cur` is None (:done true) — remaining iterations then no-op. The sieve is a
        ;; class-guarded FOREIGN predicate — `ForeignRecord/class` checked BEFORE
        ;; `ForeignRecord/get :severity`. `get` returns Option (HashMap/get's contract);
        ;; miss is None, never a raise. The class-guard is the semantic filter (only
        ;; Alert), not a raise-avoidance. `match` on the Option, not `Option/expect`
        ;; (expect raises — not total; journal sift-logs requires total?).
        final      (:wat::core::foldl
                     (:wat::core::fn [state <- :cons::Consumer::PageState  _i <- :wat::core::i64]
                       -> :cons::Consumer::PageState
                       (:wat::core::if (:cons::Consumer::PageState/done state)
                         state
                         (:wat::core::let
                           [sieve (:wat::query::sieve-pred
                                    (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
                                      (:wat::core::match
                                        (:wat::edn::read-foreign (:wat::telemetry::Log/message log))
                                        ((:wat::edn::ReadForeignOutcome::Value fr)
                                          (:wat::core::if
                                            (:wat::core::= (:wat::edn::ForeignRecord/class fr) "prod::Alert")
                                            (:wat::core::match (:wat::edn::ForeignRecord/get fr :severity)
                                              ((:wat::core::Some s) (:wat::core::= s "high"))
                                              (:wat::core::None false))
                                            false))
                                        ((:wat::edn::ReadForeignOutcome::Malformed _) false))))
                            sr    (:wat::telemetry::Journal/sift-logs journal
                                    (:wat::telemetry::Journal::SiftLogsRequest :namespace ns
                                      :time-lo 0 :time-hi 100000 :limit 50
                                      :cursor (:cons::Consumer::PageState/cur state) :sieve sieve))]
                           (:wat::core::match sr ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
                             ((:wat::telemetry::Journal::SiftLogsResponse::Success logs next-cur)
                               (:wat::core::let
                                 [new-acc (:wat::core::+ (:cons::Consumer::PageState/acc state)
                                            (:wat::core::count logs))]
                                 (:wat::core::match next-cur 
                                   (:wat::core::None
                                     (:cons::Consumer::PageState :done true :cur :wat::core::None :acc new-acc))
                                   ((:wat::core::Some c)
                                     (:cons::Consumer::PageState :done false :cur (:wat::core::Some c) :acc new-acc)))))
                             (_ (:cons::Consumer::PageState :done true :cur :wat::core::None :acc -1)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))))
                     initial
                     page-idxs)]
       (:wat::service::Outcome::Reply s
         (:cons::Consumer::SiftResponse::Count (:cons::Consumer::PageState/acc final)))))])

;; ── the orchestrator (the circuit builder): mem-store' + journal' + producer' + consumer', all
;; PROCESS-tier, grant-before-dial at every hop. flood (block), then sift (block); return count. ──
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store/grant msh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     ph    (:prod::producer/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::telemetry::journal/grant jh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:prod::producer::Record) :journal-addr jaddr)
     ch    (:cons::consumer/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::telemetry::journal/grant jh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:cons::consumer::Record) :journal-addr jaddr)
     producer (:wat::core::match (:wat::kernel::connect (:prod::producer::Handle/addr ph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     consumer (:wat::core::match (:wat::kernel::connect (:cons::consumer::Handle/addr ch)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _flood   (:prod::Producer/flood producer
                (:prod::Producer::FloodRequest :count 240 :namespace "arena-ns"))
     sr       (:cons::Consumer/sift consumer (:cons::Consumer::SiftRequest :namespace "arena-ns"))]
    (:wat::core::match sr ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:cons::Consumer::SiftResponse::Count n) n)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:cons::Consumer::SiftResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None))
      ((:cons::Consumer::SiftResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
