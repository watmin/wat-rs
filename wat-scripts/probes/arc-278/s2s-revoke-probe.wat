;; s2s-revoke-probe.wat — arc 293 revoke: mirror of s2s-midlife-vec-probe.wat (the grant proof),
;; symmetric twin exercising Admin::DenyPeer / Status::PeersDenied / <fqdn>/revoke.
;; The circuit builder:
;;   1. starts echo' on a PROCESS;
;;   2. starts caller1' on a PROCESS whose post-spawn grants caller1's pid → drives it (echo:hi)
;;      — proves the GRANT path is untouched by the mirror (no regression);
;;   3. MID-LIFE explicit revoke: calls (echo'/revoke eh <2-elem dummy vec>) directly from main
;;      AFTER echo has already booted + served caller1 — proves the revoke verb is callable
;;      post-boot AND folds a multi-element pid vec (the ack returns before the call completes);
;;   4. revokes caller1's ACTUAL granted pid (captured via a second grant+revoke round trip in
;;      caller1's OWN post-spawn hook, racing caller1's own :init connect' — the same race margin
;;      the shipped grant probe already trusts, doubled) — caller2 is spawned the same way but its
;;      pid is granted then immediately revoked before its own connect' fires, so its :init connect'
;;      should be refused by echo's accept-gate.
;; Prints (main channel):
;;   echo:hi                 (caller1 — granted only, proves grant path intact)
;;   revoke-midlife-ok        (dummy 2-elem vec fold+ack completed)
;;   caller2-init-refused     (caller2's connect' was bounced post grant+revoke race)
;; or, if the race did not land the revoke before caller2's connect (rare), whatever caller2 prints.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo
  :durable   []
  :ephemeral []
  :impls
  [(echo [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defsurface :probe::Caller :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Caller::RunRequest  [])
   (:wat::core::defenum :probe::Caller::RunResponse :wat::enum::Pure :Ok [out <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(run [self <- :probe::Caller  req <- :probe::Caller::RunRequest] -> :probe::Caller::RunResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::caller
  :satisfies :probe::Caller
  :durable   []
  :ephemeral [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]
  :peers     [:probe::Echo]
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
        out  (:wat::core::match er ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Caller::Reply::Run (:probe::Caller::RunResponse::Ok out))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Caller::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::caller::Op])]))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea  (:probe::echo::Handle/addr eh)
     ;; caller1 — granted at boot via its post-spawn hook (UNCHANGED grant path).
     ch1 (:probe::caller/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:probe::echo/grant eh
                        (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
           :record (:probe::caller::Record) :echo-addr ea)
     cc1 (:wat::core::match (:wat::kernel::connect (:probe::caller::Handle/addr ch1)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     rr1 (:probe::Caller/run cc1 (:probe::Caller::RunRequest))
     _   (:wat::kernel::println (:wat::core::match rr1 ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Caller::RunResponse::Ok out) out)
  ((:probe::Caller::RunResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Caller::RunResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
     ;; MID-LIFE explicit revoke, direct from main, echo already serving — a 2-element dummy vec.
     ;; (dummy pids; the fold + ack must complete and return nil — mirrors the grant probe's
     ;; identical mid-life dummy-vec proof.)
     _   (:probe::echo/revoke eh (:wat::core::Vector :- [:wat::core::i64] 900001 900002))
     _   (:wat::kernel::println "revoke-midlife-ok")
     ;; caller2 — its OWN post-spawn hook grants then IMMEDIATELY revokes its own pid
     ;; (both a synchronous request/reply round trip on echo's already-warm lineage peer),
     ;; racing caller2's own :init connect' (which must fork, close-sweep, reparse this
     ;; whole source file, macro-expand + typecheck it, THEN call connect') — the same race
     ;; margin the shipped grant probe already trusts (post-spawn-grant vs. child init
     ;; connect'), doubled. caller2's :init connect' should therefore be refused by echo's
     ;; accept-gate; the refusal surfaces as an unhandled RuntimeError inside caller2's
     ;; process, which crashes caller2 before its own listener ever starts serving — so
     ;; main's :wat::kernel::connect' to caller2's OWN Handle addr should itself fail
     ;; (connection refused / peer never came up).
     ch2 (:probe::caller/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::core::let
                        [pidv (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl))
                         _    (:probe::echo/grant eh pidv)
                         _    (:probe::echo/revoke eh pidv)]
                        nil)))
           :record (:probe::caller::Record) :echo-addr ea)
     cc2 (:wat::core::match (:wat::kernel::connect (:probe::caller::Handle/addr ch2)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     rr2 (:probe::Caller/run cc2 (:probe::Caller::RunRequest))]
    (:wat::kernel::println (:wat::core::match rr2 ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Caller::RunResponse::Ok out) out)
  ((:probe::Caller::RunResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Caller::RunResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))))
