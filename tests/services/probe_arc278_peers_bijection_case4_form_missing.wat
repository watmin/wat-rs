;; Co-located fixture for probe_arc278_s2s_peer_on_thread.rs — arc 278 S4d run+assert gate.
;;
;; PROMOTED from wat-scripts/probes/arc-278/s2s-thread-probe.wat (which is only TYPE-CHECKED by
;; tests/lint/wat_scripts_fixes_load.rs, never RUN). This lifts the s2s peer-holding into a real
;; run-and-assert SYSTEM TEST: a `defservice` (`caller'`) that HOLDS a `:peers` client peer to
;; ANOTHER service (`echo'`) in a ROOT `:ephemeral` field and calls that surface's method through
;; it. This is `journal'`'s exact skeleton (journal' will hold a `:wat::query::Store` peer the same
;; way). Proven to type-check on both loci; this proves it RUNS on a THREAD.
;;
;; caller' dials echo', calls Echo/echo "hi", returns the reply string. Expect "echo:hi".

;; ── ECHO: the dialed surface + its service ──────────────────────────────────────
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo
  :durable   []
  :ephemeral []
  :impls
  [(echo [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::Echo::EchoResponse::Ok
         (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

;; ── CALLER: a surface + a service that DIALS echo' (the s2s peer) ───────────────
(:wat::core::defsurface :probe::Caller :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Caller::RunRequest  [])
   (:wat::core::defenum :probe::Caller::RunResponse :wat::enum::Pure
     :Ok              [out <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(run [self <- :probe::Caller  req <- :probe::Caller::RunRequest] -> :probe::Caller::RunResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::caller
  :satisfies :probe::Caller
  :durable   []
  ;; the dialed peer — a client (Peer' :- [Echo::Op Echo::Reply]), held as a ROOT ephemeral field
  :ephemeral [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]
  ;; the explicit s2s dependency DAG — set-equal to the ephemeral peer field's surface
  :peers     [:probe::Bogus]
  ;; :init connects to echo' (its Address' crosses the fork as an operating-input cap record)
  :init (:wat::core::fn
          [record    <- :probe::caller::Record
           echo-addr <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])]
          -> :probe::caller::State
          (:probe::caller::State :durable record :echo (:wat::core::match (:wat::kernel::connect echo-addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [(run [s ctx req]
     (:wat::core::let
       [echo (:probe::caller::State/echo s)
        er   (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg "hi"))
        rresp (:wat::core::match er ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
                ((:probe::Echo::EchoResponse::Ok reply)
                  (:probe::Caller::RunResponse::Ok reply))
                ;; wire-breach at the echo peer propagates outward as our own op's breach.
                ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                  (:probe::Caller::RunResponse::RequestTooLarge bytes cap))
                ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
                  (:probe::Caller::RunResponse::RequestMalformed mpath mexpected mgot)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))]
       (:wat::service::Outcome::Reply s rresp)))])

;; ── the crossing: start both on THREADS, dial caller', which dials echo'. Return the reply. ──
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [eh  (:probe::echo/start   :locus (:wat::spawn::thread) :record (:probe::echo::Record))
     ea  (:probe::echo::Handle/addr eh)
     ch  (:probe::caller/start  :locus (:wat::spawn::thread) :record (:probe::caller::Record) :echo-addr ea)
     cc  (:wat::core::match (:wat::kernel::connect (:probe::caller::Handle/addr ch)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     rr  (:probe::Caller/run cc (:probe::Caller::RunRequest))]
    (:wat::core::match rr ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:probe::Caller::RunResponse::Ok out) out)
      ((:probe::Caller::RunResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None))
      ((:probe::Caller::RunResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
