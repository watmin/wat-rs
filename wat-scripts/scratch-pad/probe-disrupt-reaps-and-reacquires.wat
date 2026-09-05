;; probe-disrupt-closes-its-own-peer.wat — THE LAST UNKNOWN BEFORE 3c.
;;
;; The `-disrupt` shape: an internal arm, fired by a self-arming alarm, that REAPS one of
;; the service's own ephemeral peers and re-acquires it. The builder's own framing:
;; "the pipe is busted .. its gotta get reaped .. so .. we need to reacquire our handle".
;;
;; Three things are already proven and are NOT re-asked here:
;;   · a severed connection is recoverable          probe-closed-is-recoverable.wat
;;   · a seeded draw replays from wat               probe-rand-is-usable-from-wat.wat
;;   · a None reply surfaces as LOST, not a hang    probe-reply-drop-is-userland.wat
;;
;; ★ THE ONE THING NOTHING HAS DONE: `:wat::kernel::close` CONSUMES the peer, and a
;; service's peer lives in a TYPED :ephemeral field. So an arm that reaps its own peer must
;; put a fresh one back in the same call, or the field cannot be rebuilt. If that does not
;; work, `-disrupt` cannot hold its own state and the design changes.
;;
;;   front holds a peer to sink. `start` arms -disrupt at 60 ms.
;;   -disrupt closes the peer and re-dials, threading the fresh peer into state.
;;
;;   expect: before=ok:1 ; disrupts=1 ; after=ok:2   — the service kept serving across a reap

(:wat::config::set-redef! true)

(:wat::core::defsurface :dz::Sink :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :dz::Sink::PingRequest [pad <- :wat::core::String])
   (:wat::core::defenum :dz::Sink::PingResponse :wat::enum::Pure
     :Ok [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :dz::Sink  req <- :dz::Sink::PingRequest] -> :dz::Sink::PingResponse :max-request-bytes 65536)])

(:wat::service::defservice :dz::sink
  :satisfies :dz::Sink
  ;; 700: an oversized ping exceeds the FRAME cap and severs the SENDER's connection.
  ;; Contract cap stays 65536 — :max-frame-bytes (deployment) is a different axis from
  ;; :max-request-bytes (contract, which replies RequestTooLarge instead of tearing down).
  :max-frame-bytes 700
  :durable   [seen <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :dz::sink::Record] -> :dz::sink::State
          (:dz::sink::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::core::let
       [n (:wat::i64::+ (:dz::sink::Record/seen (:dz::sink::State/durable s)) 1)]
       (:wat::service::Outcome::Continue
         (:dz::sink::State :durable (:dz::sink::Record :seen n))
         (:wat::core::Some (:dz::Sink::Reply::Ping (:dz::Sink::PingResponse::Ok n)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:dz::Sink::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:dz::sink::Op])]))))])

(:wat::core::defsurface :dz::Front :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :dz::Front::HitRequest [])
   (:wat::core::defenum :dz::Front::HitResponse :wat::enum::Pure
     :Ok [n <- :wat::core::i64  disrupts <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :dz::Front::StartRequest [])
   (:wat::core::defenum :dz::Front::StartResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(hit   [self <- :dz::Front  req <- :dz::Front::HitRequest]   -> :dz::Front::HitResponse :max-request-bytes 65536)
   (start [self <- :dz::Front  req <- :dz::Front::StartRequest] -> :dz::Front::StartResponse :max-request-bytes 65536)])

(:wat::service::defservice :dz::front
  :satisfies :dz::Front
  :durable   [sink-addr <- (:wat::kernel::Address :- [:dz::Sink::Op :dz::Sink::Reply])
              disrupts  <- :wat::core::i64]
  :ephemeral [sink <- (:wat::kernel::Peer :- [:dz::Sink::Op :dz::Sink::Reply])]
  :peers     [:dz::Sink]
  :init (:wat::core::fn [record <- :dz::front::Record] -> :dz::front::State
          (:dz::front::State :durable record
            :sink (:wat::core::match (:wat::kernel::connect (:dz::front::Record/sink-addr record))
                    ((:wat::kernel::ConnectOutcome::Connected p) p)
                    (_ (:wat::kernel::assertion-failed! "dz: init dial failed" :wat::core::None :wat::core::None)))))
  :impls
  [(start [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:dz::Front::Reply::Start (:dz::Front::StartResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:dz::Front::Reply])])
       [(:wat::service::Alarm :delay (:wat::time::Milliseconds 60) :op :-disrupt)]))
   ;; ★ THE DISRUPTOR. Internal arm: SelfOutcome has NO reply field — a disruptor that
   ;; could answer a caller has no form. It reaps its own peer and re-acquires it.
   (-disrupt [s ctx]
     (:wat::core::let
       [rec  (:dz::front::State/durable s)
        old  (:dz::front::State/sink s)
        ;; REAP BY SPEAKING TOO LOUDLY. `:wat::kernel::close` is kernel-only AND takes a
        ;; spawned-child handle, not a dialed connection — an arm cannot close its peer.
        ;; An oversized FRAME severs the sender's own connection, which is the same fault
        ;; and is the mechanism circuit.wat already uses.
        pad  (:wat::core::foldl
               (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
                 (:wat::string::concat acc "xxxxxxxxxx"))
               "" (:wat::core::range 0 100))
        _c   (:wat::core::match (:dz::Sink/ping old (:dz::Sink::PingRequest :pad pad)) (_ nil))
        fresh (:wat::core::match (:wat::kernel::connect (:dz::front::Record/sink-addr rec))
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                (_ (:wat::kernel::assertion-failed! "dz: re-acquire failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
        n    (:wat::i64::+ (:dz::front::Record/disrupts rec) 1)]
       (:wat::service::SelfOutcome::Continue
         (:dz::front::State
           :durable (:dz::front::Record :sink-addr (:dz::front::Record/sink-addr rec) :disrupts n)
           :sink fresh)
         (:wat::core::Vector :- [(:wat::service::Directed :- [:dz::Front::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:dz::front::Op])]))))
   (hit [s ctx req]
     (:wat::core::let
       [rec (:dz::front::State/durable s)
        p   (:dz::front::State/sink s)
        n   (:wat::core::match (:dz::Sink/ping p (:dz::Sink::PingRequest :pad ""))
              ((:wat::kernel::RecvOutcome::Message r)
                (:wat::core::match r
                  ((:dz::Sink::PingResponse::Ok k) k)
                  (_ -1)))
              (_ -2))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:dz::Front::Reply::Hit
           (:dz::Front::HitResponse::Ok n (:dz::front::Record/disrupts rec))))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:dz::Front::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:dz::front::Op])]))))])

(:wat::core::defn :dz::pids [pl <- :wat::spawn::ProcessLaunch] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :dz::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    (_ nil)))

(:wat::core::defn :dz::hit [p <- :dz::Front] -> :wat::core::String
  (:wat::core::match (:dz::Front/hit p (:dz::Front::HitRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:dz::Front::HitResponse::Ok n d) (:wat::core::format "ok:{n}/d={d}" :n n :d d))
        (_ "BAD")))
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED") (:wat::kernel::RecvOutcome::TimedOut "LOST")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [sh (:dz::sink/start :locus (:wat::spawn::process) :record (:dz::sink::Record :seen 0))
     ;; ★ THE GRANT. At process locus sink's birth-seed allow-set holds only getppid(),
     ;; so front is a STRANGER and is bounced until granted (sns-fanout.wat:22-27). The
     ;; grant rides process/post-spawn — owner-side, after the fork, BEFORE :init dials.
     ;; Without this the very first ping fails and no amount of frame-cap tuning helps.
     fh (:dz::front/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:dz::sink/grant sh (:dz::pids pl))))
          :record (:dz::front::Record :sink-addr (:dz::sink::Handle/addr sh) :disrupts 0))
     f (:wat::core::match (:wat::kernel::connect (:dz::front::Handle/addr fh))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "dz: dial front failed" :wat::core::None :wat::core::None)))
     _ (:wat::core::match (:dz::Front/start f (:dz::Front::StartRequest)) (_ nil))
     before (:dz::hit f)
     _ (:dz::nap 200)
     after (:dz::hit f)]
    (:wat::kernel::println
      (:wat::core::format "before={b};after={a};verdict={v}"
        :b before :a after
        :v (:wat::core::if (:wat::core::= after "ok:2/d=1")
             "DISRUPT-REAPS-AND-REACQUIRES" "see-cells")))))
