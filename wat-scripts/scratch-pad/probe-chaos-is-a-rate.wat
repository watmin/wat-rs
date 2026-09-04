;; probe-chaos-is-a-rate.wat — THE RATE, not the one-shot.
;;
;; probe-disrupt-reaps-and-reacquires.wat returns empty alarms: a reap is survivable
;; and stops. That is the probe, not the stone. This file is the stone's spine:
;; -disrupt re-arms itself from a seeded draw.
;;
;; Three cells:
;;   rate 0     start arms nothing; after a wait that would have fired, draws=0
;;   rate > 0   many hits across max-draws (one firing fails)
;;   same seed  two runs → same hits AND same points
;;
;; Every RecvOutcome named. No -1/-2 collapse.

(:wat::config::set-redef! true)

(:wat::core::defsurface :cr::Sink :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :cr::Sink::PingRequest [pad <- :wat::core::String])
   (:wat::core::defenum :cr::Sink::PingResponse :wat::enum::Pure
     :Ok [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :cr::Sink  req <- :cr::Sink::PingRequest] -> :cr::Sink::PingResponse :max-request-bytes 65536)])

(:wat::service::defservice :cr::sink
  :satisfies :cr::Sink
  ;; 700: oversized ping severs the SENDER. Contract cap stays 65536.
  :max-frame-bytes 700
  :durable   [seen <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :cr::sink::Record] -> :cr::sink::State
          (:cr::sink::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::core::let
       [n (:wat::i64::+ (:cr::sink::Record/seen (:cr::sink::State/durable s)) 1)]
       (:wat::service::Outcome::Continue
         (:cr::sink::State :durable (:cr::sink::Record :seen n))
         (:wat::core::Some (:cr::Sink::Reply::Ping (:cr::Sink::PingResponse::Ok n)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:cr::Sink::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:cr::sink::Op])]))))])

(:wat::core::defsurface :cr::Front :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :cr::Front::StartRequest [])
   (:wat::core::defenum :cr::Front::StartResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :cr::Front::ReportRequest [])
   (:wat::core::defenum :cr::Front::ReportResponse :wat::enum::Pure
     :Ok [hits <- :wat::core::i64  draws <- :wat::core::i64  points <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start  [self <- :cr::Front  req <- :cr::Front::StartRequest]  -> :cr::Front::StartResponse :max-request-bytes 65536)
   (report [self <- :cr::Front  req <- :cr::Front::ReportRequest] -> :cr::Front::ReportResponse :max-request-bytes 65536)])

(:wat::service::defservice :cr::front
  :satisfies :cr::Front
  :durable   [sink-addr         <- (:wat::kernel::Address :- [:cr::Sink::Op :cr::Sink::Reply])
              disrupt-rate-bp   <- :wat::core::i64
              disrupt-seed      <- :wat::core::i64
              disrupt-lo-ms     <- :wat::core::i64
              disrupt-hi-ms     <- :wat::core::i64
              disrupt-max-draws <- :wat::core::i64
              disrupt-hits      <- :wat::core::i64
              disrupt-draws     <- :wat::core::i64
              disrupt-points    <- :wat::core::String]
  :ephemeral [sink <- (:wat::kernel::Peer :- [:cr::Sink::Op :cr::Sink::Reply])]
  :peers     [:cr::Sink]
  :init (:wat::core::fn [record <- :cr::front::Record] -> :cr::front::State
          (:cr::front::State :durable record
            :sink (:wat::core::match (:wat::kernel::connect (:cr::front::Record/sink-addr record))
                    ((:wat::kernel::ConnectOutcome::Connected p) p)
                    (_ (:wat::kernel::assertion-failed! "cr: init dial failed" :wat::core::None :wat::core::None)))))
  :impls
  [(start [s ctx req]
     (:wat::core::let
       [rec  (:cr::front::State/durable s)
        rate (:cr::front::Record/disrupt-rate-bp rec)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:cr::Front::Reply])])
        tickless (:wat::core::Vector :- [(:wat::service::Alarm :- [:cr::front::Op])])]
       (:wat::core::if (:wat::i64::> rate 0)
         (:wat::core::let
           [pair (:wat::rand::int-from (:cr::front::Record/disrupt-seed rec)
                    (:cr::front::Record/disrupt-lo-ms rec)
                    (:cr::front::Record/disrupt-hi-ms rec))
            seed1 (:wat::core::first pair)
            delay (:wat::core::second pair)
            rec'  (:cr::front::Record
                    :sink-addr (:cr::front::Record/sink-addr rec)
                    :disrupt-rate-bp rate
                    :disrupt-seed seed1
                    :disrupt-lo-ms (:cr::front::Record/disrupt-lo-ms rec)
                    :disrupt-hi-ms (:cr::front::Record/disrupt-hi-ms rec)
                    :disrupt-max-draws (:cr::front::Record/disrupt-max-draws rec)
                    :disrupt-hits (:cr::front::Record/disrupt-hits rec)
                    :disrupt-draws (:cr::front::Record/disrupt-draws rec)
                    :disrupt-points (:cr::front::Record/disrupt-points rec))
            s' (:cr::front::State :durable rec' :sink (:cr::front::State/sink s))]
           (:wat::service::Outcome::Continue s'
             (:wat::core::Some (:cr::Front::Reply::Start (:cr::Front::StartResponse::Ok)))
             none-sends
             [(:wat::service::Alarm :after (:wat::time::Millisecond delay) :op :-disrupt)]))
         (:wat::service::Outcome::Continue s
           (:wat::core::Some (:cr::Front::Reply::Start (:cr::Front::StartResponse::Ok)))
           none-sends
           tickless))))
   (report [s ctx req]
     (:wat::core::let
       [rec (:cr::front::State/durable s)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:cr::Front::Reply])])
        none-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:cr::front::Op])])]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:cr::Front::Reply::Report
           (:cr::Front::ReportResponse::Ok
             (:cr::front::Record/disrupt-hits rec)
             (:cr::front::Record/disrupt-draws rec)
             (:cr::front::Record/disrupt-points rec))))
         none-sends none-arms)))
   (-disrupt [s ctx]
     (:wat::core::let
       [rec   (:cr::front::State/durable s)
        old   (:cr::front::State/sink s)
        rate  (:cr::front::Record/disrupt-rate-bp rec)
        lo    (:cr::front::Record/disrupt-lo-ms rec)
        hi    (:cr::front::Record/disrupt-hi-ms rec)
        maxd  (:cr::front::Record/disrupt-max-draws rec)
        addr  (:cr::front::Record/sink-addr rec)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:cr::Front::Reply])])
        draw1 (:wat::rand::int-from (:cr::front::Record/disrupt-seed rec) 0 10000)
        seed1 (:wat::core::first draw1)
        bp    (:wat::core::second draw1)
        draws (:wat::i64::+ (:cr::front::Record/disrupt-draws rec) 1)
        hit?  (:wat::i64::< bp rate)
        pad   (:wat::core::foldl
                (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
                  (:wat::string::concat acc "xxxxxxxxxx"))
                "" (:wat::core::range 0 100))
        ;; Poison names every outcome. Message = no tear (thread locus) — keep the peer.
        poisoned (:wat::core::if hit?
                    (:wat::core::match (:cr::Sink/ping old (:cr::Sink::PingRequest :pad pad))
                      ((:wat::kernel::RecvOutcome::Message _r) "message")
                      ((:wat::kernel::RecvOutcome::Lost _c) "lost")
                      (:wat::kernel::RecvOutcome::Closed "closed")
                      (:wat::kernel::RecvOutcome::Stopped
                        (:wat::kernel::assertion-failed! "cr: poison stopped" :wat::core::None :wat::core::None)))
                    "miss")
        tore? (:wat::core::or (:wat::core::= poisoned "lost") (:wat::core::= poisoned "closed"))
        sink' (:wat::core::if tore?
                (:wat::core::match (:wat::kernel::connect addr)
                  ((:wat::kernel::ConnectOutcome::Connected p) p)
                  (_ (:wat::kernel::assertion-failed! "cr: re-acquire failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                old)
        hits' (:wat::core::if tore?
                (:wat::i64::+ (:cr::front::Record/disrupt-hits rec) 1)
                (:cr::front::Record/disrupt-hits rec))
        points' (:wat::core::if tore?
                  (:wat::core::format "{p}{d},"
                    :p (:cr::front::Record/disrupt-points rec) :d draws)
                  (:cr::front::Record/disrupt-points rec))
        draw2 (:wat::rand::int-from seed1 lo hi)
        seed2 (:wat::core::first draw2)
        delay (:wat::core::second draw2)
        rec'  (:cr::front::Record
                :sink-addr addr
                :disrupt-rate-bp rate
                :disrupt-seed seed2
                :disrupt-lo-ms lo
                :disrupt-hi-ms hi
                :disrupt-max-draws maxd
                :disrupt-hits hits'
                :disrupt-draws draws
                :disrupt-points points')
        s' (:cr::front::State :durable rec' :sink sink')
        rearm? (:wat::core::or (:wat::core::= maxd 0) (:wat::i64::< draws maxd))
        arms (:wat::core::if rearm?
               [(:wat::service::Alarm :after (:wat::time::Millisecond delay) :op :-disrupt)]
               (:wat::core::Vector :- [(:wat::service::Alarm :- [:cr::front::Op])]))]
       (:wat::service::SelfOutcome::Continue s' none-sends arms)))])

(:wat::core::defn :cr::pids [pl <- :wat::spawn::ProcessLaunch] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :cr::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c)
      (:wat::kernel::assertion-failed! "cr: nap lost" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "cr: nap stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "cr: nap closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :cr::dial-front
  [a <- (:wat::kernel::Address :- [:cr::Front::Op :cr::Front::Reply])]
  -> :cr::Front
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "cr: dial front failed" :wat::core::None :wat::core::None))))

;; Handles must outlive the peer — dropping them stops the service.
(:wat::core::defn :cr::boot
  [rate <- :wat::core::i64  seed <- :wat::core::i64  maxd <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:cr::sink::Handle :cr::front::Handle :cr::Front])
  (:wat::core::let
    [sh (:cr::sink/start :locus (:wat::spawn::process) :record (:cr::sink::Record :seen 0))
     fh (:cr::front/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:cr::sink/grant sh (:cr::pids pl))))
          :record (:cr::front::Record
                    :sink-addr (:cr::sink::Handle/addr sh)
                    :disrupt-rate-bp rate
                    :disrupt-seed seed
                    :disrupt-lo-ms 20
                    :disrupt-hi-ms 40
                    :disrupt-max-draws maxd
                    :disrupt-hits 0
                    :disrupt-draws 0
                    :disrupt-points ""))
     f (:cr::dial-front (:cr::front::Handle/addr fh))
     _ (:wat::core::match (:cr::Front/start f (:cr::Front::StartRequest))
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:cr::Front::StartResponse::Ok) nil)
             ((:cr::Front::StartResponse::RequestTooLarge _b _c)
               (:wat::kernel::assertion-failed! "cr: start too large" :wat::core::None :wat::core::None))
             ((:cr::Front::StartResponse::RequestMalformed _p _e _g)
               (:wat::kernel::assertion-failed! "cr: start malformed" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost _c)
           (:wat::kernel::assertion-failed! "cr: start lost" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "cr: start stopped" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "cr: start closed" :wat::core::None :wat::core::None)))]
    (:wat::core::Tuple sh fh f)))

(:wat::core::defn :cr::report [f <- :cr::Front] -> :wat::core::String
  (:wat::core::match (:cr::Front/report f (:cr::Front::ReportRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:cr::Front::ReportResponse::Ok hits draws points)
          (:wat::core::format "hits={h};draws={d};points={p}" :h hits :d draws :p points))
        ((:cr::Front::ReportResponse::RequestTooLarge _b _c)
          (:wat::kernel::assertion-failed! "cr: report too large" :wat::core::None :wat::core::None))
        ((:cr::Front::ReportResponse::RequestMalformed _p _e _g)
          (:wat::kernel::assertion-failed! "cr: report malformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost _c)
      (:wat::kernel::assertion-failed! "cr: report lost" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "cr: report stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "cr: report closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [b0 (:cr::boot 0 1 0)
     f0 (:wat::core::third b0)
     _  (:cr::nap 400)
     r0 (:cr::report f0)
     b1 (:cr::boot 2500 42 40)
     f1 (:wat::core::third b1)
     _  (:cr::nap 2500)
     r1 (:cr::report f1)
     b2 (:cr::boot 2500 42 40)
     f2 (:wat::core::third b2)
     _  (:cr::nap 2500)
     r2 (:cr::report f2)]
    (:wat::kernel::println
      (:wat::core::format "rate0={z};run1={a};run2={b};replay={rp};verdict={v}"
        :z r0 :a r1 :b r2
        :rp (:wat::core::if (:wat::core::= r1 r2) "SAME" "DIFFERS")
        :v (:wat::core::if
             (:wat::core::and (:wat::core::= r0 "hits=0;draws=0;points=")
               (:wat::core::and (:wat::core::= r1 r2)
                 (:wat::core::not (:wat::core::= r1 "hits=0;draws=0;points="))))
             "CHAOS-IS-A-RATE"
             "DO-NOT-DRAW")))))
