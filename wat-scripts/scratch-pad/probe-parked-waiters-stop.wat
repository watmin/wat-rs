;; probe-parked-waiters-stop.wat — VERIFY substrate finding (b).
;;
;; THE CLAIM (SCORE-the-sane-circuit.md:56, and circuit.wat:114 which acts on it):
;;   "a parked receive (:wait :UpTo) at process locus with >=4 waiters never completes,
;;    so Admin::Stop hangs waiting on the tick."
;; Reported repro `1 1 2` stops / `1 1 4` hangs. Recorded as NOT independently verified,
;; and the workaround it justifies -- workers polling at :wait :Immediate, re-arming every 1ms --
;; generates 94% of the circuit's hops (144,485 receive calls for 8,000 messages).
;;
;; So this is the gate on the whole perf lane, and it is verified here, not assumed.
;;
;; SHAPE. One queue at process locus over one store. J parker services at process locus,
;; each with a `-tick` that does a PARKED receive on the permanently-empty queue. Arm them,
;; let them park, then stop them and print. The sweep prints after each J, so if it hangs
;; the last printed line names the threshold. Run under `timeout`: a timeout IS the finding.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defsurface :vb::Parker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :vb::Parker::ArmRequest [])
   (:wat::core::defenum :vb::Parker::ArmResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(arm [self <- :vb::Parker  req <- :vb::Parker::ArmRequest]
     -> :vb::Parker::ArmResponse :max-request-bytes 524288)])

(:wat::service::defservice :vb::parker
  :satisfies :vb::Parker
  :durable   [queue-name <- :wat::core::String]
  :ephemeral [q <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record     <- :vb::parker::Record
           queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
          -> :vb::parker::State
          (:vb::parker::State :durable record
            :q (:wat::core::match (:wat::kernel::connect queue-addr)
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 (_ (:wat::kernel::assertion-failed! "parker: dial failed" :wat::core::None :wat::core::None)))))
  :impls
  [(arm [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:vb::Parker::Reply::Arm (:vb::Parker::ArmResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:vb::Parker::Reply])])
       [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))

   ;; THE PARKED RECEIVE. :wait :UpTo 50ms on a queue that never gets a message.
   (-tick [s ctx]
     (:wat::core::let
       [name (:vb::parker::Record/queue-name (:vb::parker::State/durable s))
        q    (:vb::parker::State/q s)
        now  (:wat::time::epoch-nanos (:wat::time::now))
        rr   (:queue::Queue/receive q
               (:queue::Queue::ReceiveRequest
                 :queue name :now-ns now :visibility-ns 1000000000000
                 :limit 10 :wait (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 50))))]
       (:wat::core::match rr
         ((:wat::kernel::RecvOutcome::Message _r)
           (:wat::service::SelfOutcome::Continue s
             (:wat::core::Vector :- [(:wat::service::Directed :- [:vb::Parker::Reply])])
             [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))
         (_ (:wat::service::SelfOutcome::Continue s
              (:wat::core::Vector :- [(:wat::service::Directed :- [:vb::Parker::Reply])])
              (:wat::core::Vector :- [(:wat::service::Alarm :- [:vb::parker::Op])]))))))])

(:wat::core::defn :vb::pids [pl <- :wat::spawn::ProcessLaunch]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :vb::nap-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

(:wat::core::defn :vb::dial-parker
  [a <- (:wat::kernel::Address :- [:vb::Parker::Op :vb::Parker::Reply])] -> :vb::Parker
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "dial-parker failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :vb::arm! [p <- :vb::Parker] -> :wat::core::nil
  (:wat::core::match (:vb::Parker/arm p (:vb::Parker::ArmRequest))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "arm failed" :wat::core::None :wat::core::None))))

;; J parkers on one queue: arm, let them park, then Stop each.
(:wat::core::defn :vb::run-j [j <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::process)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::query::mem-store/grant sh (:vb::pids pl))))
          :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr sh)))
     parkers (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::Vector :- [:vb::parker::Handle])
                                _i  <- :wat::core::i64]
                 -> (:wat::core::Vector :- [:vb::parker::Handle])
                 (:wat::core::conj acc
                   (:vb::parker/start
                     :locus (:wat::spawn::process/post-spawn
                              (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                (:queue::queue/grant qh (:vb::pids pl))))
                     :record (:vb::parker::Record :queue-name "q0")
                     :queue-addr (:queue::queue::Handle/addr qh))))
               (:wat::core::Vector :- [:vb::parker::Handle])
               (:wat::core::range 0 j))
     _arm (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
              (:vb::arm! (:vb::dial-parker (:vb::parker::Handle/addr (:wat::core::nth parkers i)))))
            nil
            (:wat::core::range 0 j))
     _settle (:vb::nap-ms 250)
     _stop (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
               (:wat::core::let [_o (:vb::parker/stop (:wat::core::nth parkers i))] (:wat::i64::+ acc 1)))
             0
             (:wat::core::range 0 j))
     _qs (:queue::queue/stop qh)
     _ss (:wat::query::mem-store/stop sh)]
    j))

(:wat::core::defn :vb::step [j <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     r  (:vb::run-j j)
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::kernel::println
      (:wat::core::format "j={j} STOPPED-OK in {ms}ms"
        :j r :ms (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)))))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::format "j1={a}" :a (:vb::run-j 1)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_1 (:vb::step 1)
     _2 (:vb::step 2)
     _3 (:vb::step 3)
     _4 (:vb::step 4)
     _5 (:vb::step 5)
     _6 (:vb::step 8)]
    (:wat::kernel::println "ALL J COMPLETED — finding (b) NOT REPRODUCED")))
