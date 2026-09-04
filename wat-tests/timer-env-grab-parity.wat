;; wat-tests/timer-env-grab-parity.wat — arc 292 R3: the env-grab idiom, locus-parity proof.
;;
;; THE idiom "programs don't care about their tier": code reads its OWN `peer-kind`
;; off its ambient `(:wat::program::env)` and hands it to `(after …)`. The tier-open
;; `(Timer' :- [O])` fuses into whatever reactor it landed on. The SAME service runs unchanged
;; on a thread (crossbeam) and a process (io_uring).
;;
;; Model: wat-tests/service-locus-parity.wat — ONE defservice, two deftests differing in
;; EXACTLY ONE token, the locus `(:wat::spawn::thread)` vs `(:wat::spawn::process)`. The
;; generated client face (start / connect' / wait-tick / Handle / Response) is byte-identical.
;; The op handler runs inside the spawned peer, where the program-env is installed (a real
;; spawn — spawn.rs:623), so `(:wat::program::env)` resolves and the env-grab fires.
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; start takes ::Record(0). Op body doesn't read count (only the env-grab matters here).

;; ── the surface (the deadline protocol, lifted) ──────────────────────────────
;; arc 278 S4c: the surface OWNS its protocol messages (:messages) so a :satisfies
;; service ships them across a process fork.
(:wat::core::defsurface :wat-tests::Deadline :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::Deadline::WaitTickRequest  [])
   (:wat::core::defenum :wat-tests::Deadline::WaitTickResponse :wat::enum::Pure
     :Ok              [fired <- :wat::core::keyword]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(wait-tick [self <- :wat-tests::Deadline  req <- :wat-tests::Deadline::WaitTickRequest] -> :wat-tests::Deadline::WaitTickResponse :max-request-bytes 524288)])

;; ── the service, defined once at top-level (shared by both deftests) ──────────
(:wat::service::defservice :wat-tests::deadline
  :satisfies :wat-tests::Deadline
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(wait-tick [s ctx req]
     (:wat::core::let
       [m (:wat::core::match
            (:wat::kernel::select
              (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
                (:wat::kernel::after
                  (:wat::program::Env/peer-kind (:wat::program::env))   ;; grab MY OWN kind off the env
                  (:wat::time::Milliseconds 50)
                  :tick)))
             
            ((:wat::spawn::ServiceEvent::Message _idx mm) mm)
            (_ :no-tick))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:wat-tests::Deadline::Reply::WaitTick (:wat-tests::Deadline::WaitTickResponse::Ok m))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Deadline::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat-tests::deadline::Op])]))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::timer::env-grab-on-thread
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::deadline/start :locus (:wat::spawn::thread) :record (:wat-tests::deadline::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::deadline::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::Deadline/wait-tick c (:wat-tests::Deadline::WaitTickRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::Deadline::WaitTickResponse::Ok fired) fired)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::Deadline::WaitTickResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "deadline-wait-tick: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::Deadline::WaitTickResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    :tick))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
(:wat::test::deftest :wat-tests::timer::env-grab-on-process
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::deadline/start :locus (:wat::spawn::process) :record (:wat-tests::deadline::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::deadline::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::Deadline/wait-tick c (:wat-tests::Deadline::WaitTickRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::Deadline::WaitTickResponse::Ok fired) fired)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::Deadline::WaitTickResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "deadline-wait-tick: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::Deadline::WaitTickResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    :tick))
