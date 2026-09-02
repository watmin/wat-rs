;; probe-three-waiters-wake.wat — the untested half of long-poll adoption.
;;
;; probe-parked-waiters-stop.wat proved parked waiters can be Stopped. It never
;; woke them. The circuit's trap door is 12 waiters across 4 queues (J=3 per
;; queue) at process locus: send must Directed-wake the parked receives, and
;; the workers must ack, or drain waits forever on in-flight.
;;
;; SHAPE. One queue at process locus. J parker services at process locus, each
;; with a -tick that parks (wait-ns 250ms), acks what it gets, re-arms on empty.
;; Arm them, let them park, SEND n messages, then wait for pending=0 AND
;; in-flight=0 with a bound. A timeout prints leftover depth — that is the
;; finding. Parent thread only; parkers do not call sibling defns.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defsurface :vw::Parker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :vw::Parker::ArmRequest [])
   (:wat::core::defenum :vw::Parker::ArmResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(arm [self <- :vw::Parker  req <- :vw::Parker::ArmRequest]
     -> :vw::Parker::ArmResponse :max-request-bytes 524288)])

(:wat::service::defservice :vw::parker
  :satisfies :vw::Parker
  :durable   [queue-name <- :wat::core::String]
  :ephemeral [q     <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
              n-got <- :wat::core::i64]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record     <- :vw::parker::Record
           queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
          -> :vw::parker::State
          (:vw::parker::State :durable record
            :q (:wat::core::match (:wat::kernel::connect queue-addr)
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 (_ (:wat::kernel::assertion-failed! "parker: dial failed" :wat::core::None :wat::core::None)))
            :n-got 0))
  :stop (:wat::core::fn [s <- :vw::parker::State] -> :wat::core::i64
          (:vw::parker::State/n-got s))
  :impls
  [(arm [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:vw::Parker::Reply::Arm (:vw::Parker::ArmResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:vw::Parker::Reply])])
       [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))

   (-tick [s ctx]
     (:wat::core::let
       [name (:vw::parker::Record/queue-name (:vw::parker::State/durable s))
        q    (:vw::parker::State/q s)
        got  (:vw::parker::State/n-got s)
        now  (:wat::time::epoch-nanos (:wat::time::now))
        vis  1000000000000
        rr   (:queue::Queue/receive q
               (:queue::Queue::ReceiveRequest
                 :queue name :now-ns now :visibility-ns vis
                 :limit 10 :wait-ns 250000000))]
       (:wat::core::match rr
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:queue::Queue::ReceiveResponse::Ok envs)
               (:wat::core::if (:wat::core::empty? envs)
                 (:wat::service::SelfOutcome::Continue s
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:vw::Parker::Reply])])
                   [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)])
                 (:wat::core::let
                   [got' (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::i64  e <- :queue::Envelope]
                             -> :wat::core::i64
                             (:wat::core::let
                               [ar (:queue::Queue/ack q
                                     (:queue::Queue::AckRequest
                                       :queue name :id (:queue::Envelope/id e)))]
                               (:wat::core::match ar
                                 ((:wat::kernel::RecvOutcome::Message _ar)
                                   (:wat::i64::+ acc 1))
                                 (_ acc))))
                           got
                           envs)
                    s' (:vw::parker::State :durable (:vw::parker::State/durable s)
                         :q q :n-got got')]
                   (:wat::service::SelfOutcome::Continue s'
                     (:wat::core::Vector :- [(:wat::service::Directed :- [:vw::Parker::Reply])])
                     [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))))
             (_ (:wat::service::SelfOutcome::Continue s
                  (:wat::core::Vector :- [(:wat::service::Directed :- [:vw::Parker::Reply])])
                  [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))))
         (_ (:wat::service::SelfOutcome::Continue s
              (:wat::core::Vector :- [(:wat::service::Directed :- [:vw::Parker::Reply])])
              [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)])))))])

(:wat::core::defn :vw::pids [pl <- :wat::spawn::ProcessLaunch]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :vw::nap-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

(:wat::core::defn :vw::dial-parker
  [a <- (:wat::kernel::Address :- [:vw::Parker::Op :vw::Parker::Reply])] -> :vw::Parker
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "dial-parker failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :vw::dial-queue
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "dial-queue failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :vw::arm! [p <- :vw::Parker] -> :wat::core::nil
  (:wat::core::match (:vw::Parker/arm p (:vw::Parker::ArmRequest))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "arm failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :vw::depth-of
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks pending inflight)
          (:wat::core::Tuple pending inflight))
        (_ (:wat::core::Tuple 1 1))))
    (_ (:wat::core::Tuple 1 1))))

(:wat::core::defn :vw::calls-of [q <- :queue::Queue] -> :wat::core::i64
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok calls _ticks _p _f) calls)
        (_ -1)))
    (_ -1)))

;; TCO. Bound so a hang becomes a printed leftover depth, not a silent timeout.
(:wat::core::defn :vw::wait-depth
  [q <- :queue::Queue  left <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [d (:vw::depth-of q)
     p (:wat::core::first d)
     f (:wat::core::second d)]
    (:wat::core::if (:wat::core::and (:wat::core::= p 0) (:wat::core::= f 0))
      (:wat::core::format "status=drained;left={left};p={p};f={f}" :left left :p p :f f)
      (:wat::core::if (:wat::i64::<= left 0)
        (:wat::core::format "status=timeout;left=0;p={p};f={f}" :p p :f f)
        (:wat::core::let [_ (:vw::nap-ms 50)]
          (:vw::wait-depth q (:wat::i64::- left 1)))))))

(:wat::core::defn :vw::run-j
  [j <- :wat::core::i64  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     sh (:wat::query::mem-store/start :locus (:wat::spawn::process)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::query::mem-store/grant sh (:vw::pids pl))))
          :record (:queue::queue::Record)
          :store-addr (:wat::query::mem-store::Handle/addr sh))
     parkers (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::Vector :- [:vw::parker::Handle])
                                _i  <- :wat::core::i64]
                 -> (:wat::core::Vector :- [:vw::parker::Handle])
                 (:wat::core::conj acc
                   (:vw::parker/start
                     :locus (:wat::spawn::process/post-spawn
                              (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                (:queue::queue/grant qh (:vw::pids pl))))
                     :record (:vw::parker::Record :queue-name "q0")
                     :queue-addr (:queue::queue::Handle/addr qh))))
               (:wat::core::Vector :- [:vw::parker::Handle])
               (:wat::core::range 0 j))
     _arm (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
              (:vw::arm! (:vw::dial-parker (:vw::parker::Handle/addr (:wat::core::nth parkers i)))))
            nil
            (:wat::core::range 0 j))
     _settle (:vw::nap-ms 100)
     q (:vw::dial-queue (:queue::queue::Handle/addr qh))
     _send (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
               (:wat::core::let
                 [now (:wat::time::epoch-nanos (:wat::time::now))]
                 (:wat::core::match
                   (:queue::Queue/send q
                     (:queue::Queue::SendRequest :queue "q0" :body (:wat::core::str i) :now-ns now))
                   ((:wat::kernel::RecvOutcome::Message _r) nil)
                   (_ nil))))
             nil
             (:wat::core::range 0 n))
     w (:vw::wait-depth q 80)
     calls (:vw::calls-of q)
     got (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
             (:wat::i64::+ acc (:vw::parker/stop (:wat::core::nth parkers i))))
           0
           (:wat::core::range 0 j))
     _qs (:queue::queue/stop qh)
     _ss (:wat::query::mem-store/stop sh)
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::core::format
      "j={j};n={n};{w};got={got};calls={c};ms={ms}"
      :j j :n n :w w :got got :c calls
      :ms (:wat::i64::/ (:wat::i64::- t1 t0) 1000000))))

(:wat::core::defn :vw::step [j <- :wat::core::i64  n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println (:vw::run-j j n)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:vw::run-j 3 9))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_1 (:vw::step 1 9)
     _2 (:vw::step 2 9)
     _3 (:vw::step 3 9)]
    (:wat::kernel::println "WAKE SWEEP COMPLETE")))
