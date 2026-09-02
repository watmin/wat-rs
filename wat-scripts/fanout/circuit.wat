;; wat-scripts/fanout/circuit.wat — the app that proves wat-topic and wat-queue compose.
;;
;; N messages → 1 topic → M queues → J workers/queue → N×M outcomes.
;; Placement: this directory (composes topic + queue; does not live inside either).
;;
;; ★ TOPOLOGY IS THE SAFETY ARGUMENT. receive is scan-index then put. What serializes
;;   those two calls is that a defservice is a serializing actor (wat/query/mem.wat:22-24).
;;   ONE queue service instance per queue; J workers DIAL it. J queue services over one
;;   store would each serialize internally and race each other.
;;
;; ★ PARALLELISM BY IDS, NOT A CLOCK. Each worker is given an id at spawn and stamps it
;;   on every outcome. All M×J ids must appear. Workers are :locus process.
;;
;; ★ A DUPLICATE IS A FINDING. The summary reports total vs distinct vs dup. Do not
;;   dedupe. Visibility is 10^12 ns (~1000s) so a worker slower than the window is not
;;   the explanation — extras are actor-serialization failures, not redelivery.
;;
;; Composition: load-file! the shipped topic and queue programs (they each have
;; :user::main). set-redef! lets this file's main win. Adapter :satisfies :demo::Sub
;; and Queue/send on deliver — the missing wire between topic and queue.
;;
;; Shape: start workers (consume immediately, on empty queues) → publish alongside
;; them → drain on depth (pending = 0 AND in-flight = 0 AND topic outbox = 0) →
;; Admin::Stop; tallies return via Status::Stopped. Publish means accepted; the
;; topic fans out on its own tick. A completion check must cover every place a
;; message can rest — the outbox is the new one.
;;
;; :user::main  → N=2000 M=4 J=3 (standalone weight)
;; :user::compute → N=12 M=2 J=2 (floor; same wiring)

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")
(:wat::load-file! "../queue/sqs.wat")

;; ── adapter: :demo::Sub whose deliver is Queue/send ──────────────────────────────
(:wat::service::defservice :fanout::adapter
  :satisfies :demo::Sub
  :durable   [queue-name <- :wat::core::String]
  :ephemeral [q <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record     <- :fanout::adapter::Record
           queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
          -> :fanout::adapter::State
          (:fanout::adapter::State :durable record
            :q (:wat::core::match (:wat::kernel::connect queue-addr)
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 ((:wat::kernel::ConnectOutcome::Refused c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Rejected c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Failed c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [(deliver [s ctx req]
     (:wat::core::let
       [name (:fanout::adapter::Record/queue-name (:fanout::adapter::State/durable s))
        body (:demo::Sub::DeliverRequest/msg req)
        now  (:wat::time::epoch-nanos (:wat::time::now))
        sr   (:queue::Queue/send (:fanout::adapter::State/q s)
               (:queue::Queue::SendRequest :queue name :body body :now-ns now))]
       (:wat::core::match sr
         ((:wat::kernel::RecvOutcome::Message _r)
           (:wat::service::Outcome::Continue s (:wat::core::Some (:demo::Sub::Reply::Deliver (:demo::Sub::DeliverResponse::Ok body))) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Sub::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::adapter::Op])])))
         (_ (:wat::service::Outcome::Continue s (:wat::core::Some (:demo::Sub::Reply::Deliver (:demo::Sub::DeliverResponse::Ok body))) (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Sub::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::adapter::Op])]))))))])

;; ── worker: self-scheduling process that pulls from ONE queue ────────────────
(:wat::core::defsurface :fanout::Worker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :fanout::Outcome
     [worker <- :wat::core::String
      queue  <- :wat::core::String
      id     <- :wat::core::String
      body   <- :wat::core::String])
   (:wat::core::defrecord :fanout::Worker::StartRequest [])
   (:wat::core::defenum :fanout::Worker::StartResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start [self <- :fanout::Worker  req <- :fanout::Worker::StartRequest]
     -> :fanout::Worker::StartResponse :max-request-bytes 524288)])

(:wat::service::defservice :fanout::worker
  :satisfies :fanout::Worker
  :durable   [id         <- :wat::core::String
              queue-name <- :wat::core::String]
  :ephemeral [q        <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
              outcomes <- (:wat::core::PersistentVector :- [:fanout::Outcome])]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record     <- :fanout::worker::Record
           queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
          -> :fanout::worker::State
          (:fanout::worker::State :durable record
            :q (:wat::core::match (:wat::kernel::connect queue-addr)
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 ((:wat::kernel::ConnectOutcome::Refused c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Rejected c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Failed c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
            :outcomes (:wat::core::PersistentVector :- [:fanout::Outcome])))
  :stop (:wat::core::fn [s <- :fanout::worker::State] -> (:wat::core::PersistentVector :- [:fanout::Outcome])
          (:fanout::worker::State/outcomes s))
  :impls
  [(start [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:fanout::Worker::Reply::Start (:fanout::Worker::StartResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))
   ;; Park, don't poll. wait-ns 250 ms is the idle wait. An empty return is
   ;; "nothing yet" — re-arm so the serve loop can take Admin::Stop. The
   ;; queue/topic now arm from state (level-triggered); the 1 ms after a
   ;; park is the Stop yield, not the idle poll.
   (-tick [s ctx]
     (:wat::core::let
       [rec  (:fanout::worker::State/durable s)
        q    (:fanout::worker::State/q s)
        name (:fanout::worker::Record/queue-name rec)
        wid  (:fanout::worker::Record/id rec)
        outs (:fanout::worker::State/outcomes s)
        now  (:wat::time::epoch-nanos (:wat::time::now))
        vis  1000000000000
        rr   (:queue::Queue/receive q
               (:queue::Queue::ReceiveRequest
                 :queue name :now-ns now :visibility-ns vis :limit 10 :wait-ns 250000000))]
       (:wat::core::match rr
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:queue::Queue::ReceiveResponse::Ok envs)
               (:wat::core::let
                 [outs' (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:fanout::Outcome])
                                           e   <- :queue::Envelope]
                            -> (:wat::core::PersistentVector :- [:fanout::Outcome])
                            (:wat::core::let
                              [eid   (:queue::Envelope/id e)
                               ebody (:queue::Envelope/body e)
                               ar    (:queue::Queue/ack q
                                       (:queue::Queue::AckRequest :queue name :id eid))]
                              (:wat::core::match ar
                                ((:wat::kernel::RecvOutcome::Message _ar)
                                  (:wat::vector::conj acc
                                    (:fanout::Outcome :worker wid :queue name :id eid :body ebody)))
                                ((:wat::kernel::RecvOutcome::Lost cause)
                                  (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                                (:wat::kernel::RecvOutcome::Stopped
                                  (:wat::kernel::assertion-failed! "fanout worker: ack stopped" :wat::core::None :wat::core::None))
                                (:wat::kernel::RecvOutcome::Closed
                                  (:wat::kernel::assertion-failed! "fanout worker: ack closed" :wat::core::None :wat::core::None)))))
                          outs
                          envs)
                  s' (:fanout::worker::State :durable rec :q q :outcomes outs')]
                 (:wat::service::SelfOutcome::Continue s'
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)])))
             (_ (:wat::kernel::assertion-failed! "fanout worker: receive not Ok" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost cause)
           (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "fanout worker: receive stopped" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "fanout worker: receive closed" :wat::core::None :wat::core::None)))))])

;; Delayed-ack worker: receive this tick, ack the next. Row 2 removes the in-flight
;; term from the drain condition and requires a loss — same-tick ack would hide it.
(:wat::service::defservice :fanout::held-worker
  :satisfies :fanout::Worker
  :durable   [id         <- :wat::core::String
              queue-name <- :wat::core::String]
  :ephemeral [q        <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
              outcomes <- (:wat::core::Vector :- [:fanout::Outcome])
              held     <- (:wat::core::Vector :- [:queue::Envelope])]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record     <- :fanout::held-worker::Record
           queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
          -> :fanout::held-worker::State
          (:fanout::held-worker::State :durable record
            :q (:wat::core::match (:wat::kernel::connect queue-addr)
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 ((:wat::kernel::ConnectOutcome::Refused c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Rejected c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Failed c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
            :outcomes (:wat::core::Vector :- [:fanout::Outcome])
            :held (:wat::core::Vector :- [:queue::Envelope])))
  :stop (:wat::core::fn [s <- :fanout::held-worker::State] -> (:wat::core::Vector :- [:fanout::Outcome])
          (:fanout::held-worker::State/outcomes s))
  :impls
  [(start [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:fanout::Worker::Reply::Start (:fanout::Worker::StartResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))
   (-tick [s ctx]
     (:wat::core::let
       [rec  (:fanout::held-worker::State/durable s)
        q    (:fanout::held-worker::State/q s)
        name (:fanout::held-worker::Record/queue-name rec)
        wid  (:fanout::held-worker::Record/id rec)
        outs (:fanout::held-worker::State/outcomes s)
        held (:fanout::held-worker::State/held s)]
       (:wat::core::if (:wat::core::not (:wat::core::empty? held))
         (:wat::core::let
           [outs' (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::Outcome])
                                     e   <- :queue::Envelope]
                      -> (:wat::core::Vector :- [:fanout::Outcome])
                      (:wat::core::let
                        [eid   (:queue::Envelope/id e)
                         ebody (:queue::Envelope/body e)
                         ar    (:queue::Queue/ack q
                                 (:queue::Queue::AckRequest :queue name :id eid))]
                        (:wat::core::match ar
                          ((:wat::kernel::RecvOutcome::Message _ar)
                            (:wat::core::conj acc
                              (:fanout::Outcome :worker wid :queue name :id eid :body ebody)))
                          ((:wat::kernel::RecvOutcome::Lost cause)
                            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Stopped
                            (:wat::kernel::assertion-failed! "held-worker: ack stopped" :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Closed
                            (:wat::kernel::assertion-failed! "held-worker: ack closed" :wat::core::None :wat::core::None)))))
                    outs
                    held)
            s' (:fanout::held-worker::State :durable rec :q q :outcomes outs'
                 :held (:wat::core::Vector :- [:queue::Envelope]))]
           (:wat::service::SelfOutcome::Continue s'
             (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 500) :op :-tick)]))
         (:wat::core::let
           [now (:wat::time::epoch-nanos (:wat::time::now))
            vis 1000000000000
            rr  (:queue::Queue/receive q
                  (:queue::Queue::ReceiveRequest
                    :queue name :now-ns now :visibility-ns vis :limit 10 :wait-ns 50000000))]
           (:wat::core::match rr
             ((:wat::kernel::RecvOutcome::Message r)
               (:wat::core::match r
                 ((:queue::Queue::ReceiveResponse::Ok envs)
                   (:wat::core::if (:wat::core::empty? envs)
                     (:wat::service::SelfOutcome::Continue s
                       (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)])
                     (:wat::core::let
                       [s' (:fanout::held-worker::State :durable rec :q q :outcomes outs :held envs)]
                       (:wat::service::SelfOutcome::Continue s'
                         (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :after (:wat::time::Millisecond 500) :op :-tick)]))))
                 (_ (:wat::kernel::assertion-failed! "held-worker: receive not Ok" :wat::core::None :wat::core::None))))
             ((:wat::kernel::RecvOutcome::Lost cause)
               (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Stopped
               (:wat::kernel::assertion-failed! "held-worker: receive stopped" :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Closed
               (:wat::kernel::assertion-failed! "held-worker: receive closed" :wat::core::None :wat::core::None)))))))])

;; ── parent-side helpers (owner thread; Handles stay in :user::run's let) ────────
(:wat::core::defn :fanout::qname [i <- :wat::core::i64] -> :wat::core::String
  (:wat::core::format "q{i}" :i i))

(:wat::core::defn :fanout::wid [qi <- :wat::core::i64  wi <- :wat::core::i64] -> :wat::core::String
  (:wat::core::format "q{qi}-w{wi}" :qi qi :wi wi))

(:wat::core::defn :fanout::dial-topic
  [a <- (:wat::kernel::Address :- [:demo::Topic::Op :demo::Topic::Reply])]
  -> :demo::Topic
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::dial-queue
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::dial-worker
  [a <- (:wat::kernel::Address :- [:fanout::Worker::Op :fanout::Worker::Reply])]
  -> (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::pids [pl <- :wat::spawn::ProcessLaunch]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :fanout::face-start
  [w <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
  -> :wat::core::nil
  (:wat::core::match (:fanout::Worker/start w (:fanout::Worker::StartRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:fanout::Worker::StartResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "fanout: start not Ok" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "fanout: start stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "fanout: start closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::nap-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

(:wat::core::defn :fanout::depth-of
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks pending inflight)
          (:wat::core::Tuple pending inflight))
        (_ (:wat::core::Tuple 1 1))))
    (_ (:wat::core::Tuple 1 1))))

(:wat::core::defn :fanout::queue-drained? [q <- :queue::Queue] -> :wat::core::bool
  (:wat::core::let [d (:fanout::depth-of q)]
    (:wat::core::and (:wat::core::= (:wat::core::first d) 0)
      (:wat::core::= (:wat::core::second d) 0))))

(:wat::core::defn :fanout::all-drained?
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [ok <- :wat::core::bool  q <- :queue::Queue] -> :wat::core::bool
      (:wat::core::if (:wat::core::not ok) false (:fanout::queue-drained? q)))
    true
    qclients))

(:wat::core::defn :fanout::topic-outbox [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok n _ticks) n)
        (_ 1)))
    (_ 1)))

(:wat::core::defn :fanout::fully-drained?
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic] -> :wat::core::bool
  (:wat::core::and (:fanout::all-drained? qclients)
    (:wat::core::= (:fanout::topic-outbox t) 0)))

;; TCO. No attempts bound — if this hangs, the drain condition is wrong.
;; Third term: topic outbox. An accepted-but-undelivered message rests there,
;; invisible to pending and in-flight.
(:wat::core::defn :fanout::wait-drained
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic] -> :wat::core::nil
  (:wat::core::if (:fanout::fully-drained? qclients t)
    nil
    (:wat::core::let [_ (:fanout::nap-ms 5)]
      (:fanout::wait-drained qclients t))))

(:wat::core::defn :fanout::accept!
  [t <- :demo::Topic  msg <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:demo::Topic/publish t (:demo::Topic::PublishRequest :msg msg))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::PublishResponse::Ok) nil)
        ((:demo::Topic::PublishResponse::Full _d _c)
          (:wat::core::let [_ (:fanout::nap-ms 1)]
            (:fanout::accept! t msg)))
        (_ (:wat::kernel::assertion-failed! "fanout: publish not Ok/Full" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "fanout: publish recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::wait-pending-zero
  [q <- :queue::Queue] -> :wat::core::nil
  (:wat::core::if (:wat::core::= (:wat::core::first (:fanout::depth-of q)) 0)
    nil
    (:wat::core::let [_ (:fanout::nap-ms 5)]
      (:fanout::wait-pending-zero q))))

(:wat::core::defn :fanout::sum-calls
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  q <- :queue::Queue] -> :wat::core::i64
      (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:queue::Queue::StatsResponse::Ok calls _ticks _p _f)
              (:wat::i64::+ acc calls))
            (_ acc)))
        (_ acc)))
    0
    qclients))

(:wat::core::defn :fanout::sum-ticks
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  q <- :queue::Queue] -> :wat::core::i64
      (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:queue::Queue::StatsResponse::Ok _calls ticks _p _f)
              (:wat::i64::+ acc ticks))
            (_ acc)))
        (_ acc)))
    0
    qclients))

(:wat::core::defn :fanout::collect-stop
  [handles <- (:wat::core::Vector :- [:fanout::worker::Handle])]
  -> (:wat::core::Vector :- [:fanout::Outcome])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::Outcome])
                     h   <- :fanout::worker::Handle]
      -> (:wat::core::Vector :- [:fanout::Outcome])
      (:wat::core::foldl
        (:wat::core::fn [a <- (:wat::core::Vector :- [:fanout::Outcome])
                         o <- :fanout::Outcome]
          -> (:wat::core::Vector :- [:fanout::Outcome])
          (:wat::core::conj a o))
        acc
        (:fanout::worker/stop h)))
    (:wat::core::Vector :- [:fanout::Outcome])
    handles))

(:wat::core::defn :fanout::key-of [o <- :fanout::Outcome] -> :wat::core::String
  (:wat::string::concat (:fanout::Outcome/queue o)
    (:wat::string::concat "/" (:fanout::Outcome/id o))))

(:wat::core::defn :fanout::body-key [o <- :fanout::Outcome] -> :wat::core::String
  (:wat::string::concat (:fanout::Outcome/queue o)
    (:wat::string::concat "/" (:fanout::Outcome/body o))))

(:wat::core::defn :fanout::summarize
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64
   outs <- (:wat::core::Vector :- [:fanout::Outcome])
   empty <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [total (:wat::core::count outs)
     id-map (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                               o   <- :fanout::Outcome]
                -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                (:wat::hashmap::assoc acc (:fanout::key-of o) true))
              (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
              outs)
     w-map (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                              o   <- :fanout::Outcome]
               -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
               (:wat::hashmap::assoc acc (:fanout::Outcome/worker o) true))
             (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
             outs)
     distinct (:wat::core::count (:wat::hashmap::keys id-map))
     wcount   (:wat::core::count (:wat::hashmap::keys w-map))
     dup      (:wat::core::- total distinct)]
    (:wat::core::format
      "n={n};m={m};j={j};total={total};distinct={distinct};dup={dup};workers={workers};empty={empty}"
      :n n :m m :j j :total total :distinct distinct :dup dup :workers wcount :empty empty)))

;; Wiring + input stream. start workers → publish → drain on depth → Stop.
(:wat::core::defn :user::run*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:wat::core::let
    [t-setup0 (:wat::time::epoch-nanos (:wat::time::now))
     stores (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::mem-store::Handle])
                               _i  <- :wat::core::i64]
                -> (:wat::core::Vector :- [:wat::query::mem-store::Handle])
                (:wat::core::conj acc
                  (:wat::query::mem-store/start :locus (:wat::spawn::process)
                    :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))))
              (:wat::core::Vector :- [:wat::query::mem-store::Handle])
              (:wat::core::range 0 m))
     queues (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::queue::Handle])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [:queue::queue::Handle])
                (:wat::core::let
                  [sh (:wat::core::nth stores i)
                   h  (:queue::queue/start
                        :locus (:wat::spawn::process/post-spawn
                                 (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                   (:wat::query::mem-store/grant sh (:fanout::pids pl))))
                        :record (:queue::queue::Record)
                        :store-addr (:wat::query::mem-store::Handle/addr sh))]
                  (:wat::core::conj acc h)))
              (:wat::core::Vector :- [:queue::queue::Handle])
              (:wat::core::range 0 m))
     adapters (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::adapter::Handle])
                                 i   <- :wat::core::i64]
                  -> (:wat::core::Vector :- [:fanout::adapter::Handle])
                  (:wat::core::let
                    [qh (:wat::core::nth queues i)
                     h  (:fanout::adapter/start
                          :locus (:wat::spawn::process/post-spawn
                                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                     (:queue::queue/grant qh (:fanout::pids pl))))
                          :record (:fanout::adapter::Record :queue-name (:fanout::qname i))
                          :queue-addr (:queue::queue::Handle/addr qh))]
                    (:wat::core::conj acc h)))
                (:wat::core::Vector :- [:fanout::adapter::Handle])
                (:wat::core::range 0 m))
     sub-addrs (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])])
                                  i   <- :wat::core::i64]
                   -> (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])])
                   (:wat::core::conj acc
                     (:fanout::adapter::Handle/addr (:wat::core::nth adapters i))))
                 (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])])
                 (:wat::core::range 0 m))
     th (:demo::topic/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::core::let
                       [pids (:fanout::pids pl)]
                       (:wat::core::foldl
                         (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
                           (:fanout::adapter/grant (:wat::core::nth adapters i) pids))
                         nil
                         (:wat::core::range 0 m)))))
          :record (:demo::topic::Record :cap 4096 :delay-ns 1000) :addrs sub-addrs)
     qclients (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::Queue])
                                 i   <- :wat::core::i64]
                  -> (:wat::core::Vector :- [:queue::Queue])
                  (:wat::core::conj acc
                    (:fanout::dial-queue (:queue::queue::Handle/addr (:wat::core::nth queues i)))))
                (:wat::core::Vector :- [:queue::Queue])
                (:wat::core::range 0 m))
     topic (:fanout::dial-topic (:demo::topic::Handle/addr th))
     workers (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::worker::Handle])
                                qi  <- :wat::core::i64]
                 -> (:wat::core::Vector :- [:fanout::worker::Handle])
                 (:wat::core::let
                   [qh (:wat::core::nth queues qi)
                    inner (:wat::core::foldl
                            (:wat::core::fn [wacc <- (:wat::core::Vector :- [:fanout::worker::Handle])
                                             wi   <- :wat::core::i64]
                              -> (:wat::core::Vector :- [:fanout::worker::Handle])
                              (:wat::core::let
                                [h (:fanout::worker/start
                                     :locus (:wat::spawn::process/post-spawn
                                              (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                                (:queue::queue/grant qh (:fanout::pids pl))))
                                     :record (:fanout::worker::Record
                                               :id (:fanout::wid qi wi)
                                               :queue-name (:fanout::qname qi))
                                     :queue-addr (:queue::queue::Handle/addr qh))]
                                (:wat::core::conj wacc h)))
                            acc
                            (:wat::core::range 0 j))]
                   inner))
               (:wat::core::Vector :- [:fanout::worker::Handle])
               (:wat::core::range 0 m))
     wcount (:wat::core::* m j)
     wpeers (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])])
                (:wat::core::conj acc
                  (:fanout::dial-worker (:fanout::worker::Handle/addr (:wat::core::nth workers i)))))
              (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])])
              (:wat::core::range 0 wcount))
     _go (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::nil  w <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])] -> :wat::core::nil
             (:fanout::face-start w))
           nil
           wpeers)
     t-pub0 (:wat::time::epoch-nanos (:wat::time::now))
     _pub (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
              (:fanout::accept! topic (:wat::core::str i)))
            nil
            (:wat::core::range 0 n))
     t-drain0 (:wat::time::epoch-nanos (:wat::time::now))
     _drain (:fanout::wait-drained qclients topic)
     t-stop0 (:wat::time::epoch-nanos (:wat::time::now))
     calls (:fanout::sum-calls qclients)
     ticks (:fanout::sum-ticks qclients)
     outs (:fanout::collect-stop workers)
     empty-flags (:wat::core::foldl
                   (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
                     (:wat::core::let
                       [qp (:wat::core::nth qclients i)
                        now (:wat::time::epoch-nanos (:wat::time::now))
                        rr (:queue::Queue/receive qp
                             (:queue::Queue::ReceiveRequest
                               :queue (:fanout::qname i) :now-ns now :visibility-ns 1000000000000 :limit 1 :wait-ns 0))]
                       (:wat::core::match rr
                         ((:wat::kernel::RecvOutcome::Message r)
                           (:wat::core::match r
                             ((:queue::Queue::ReceiveResponse::Ok envs)
                               (:wat::core::if (:wat::core::empty? envs) acc 0))
                             (_ 0)))
                         (_ 0))))
                   1
                   (:wat::core::range 0 m))
     summary (:fanout::summarize n m j outs empty-flags)
     t-end (:wat::time::epoch-nanos (:wat::time::now))
     ms (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
          (:wat::i64::/ (:wat::i64::- b a) 1000000))
     phases (:wat::core::format
              "setup={setup};publish={pub};drain={drain};stop={stop};ticks={ticks}"
              :setup (ms t-setup0 t-pub0)
              :pub (ms t-pub0 t-drain0)
              :drain (ms t-drain0 t-stop0)
              :stop (ms t-stop0 t-end)
              :ticks ticks)]
    (:wat::core::Tuple summary calls phases)))

(:wat::core::defn :user::run
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::first (:user::run* n m j)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:user::run 12 2 2))

(:wat::core::defn :user::compute-calls [] -> :wat::core::String
  (:wat::core::let [pair (:user::run* 12 2 2)]
    (:wat::core::format "calls={c}" :c (:wat::core::second pair))))

(:wat::core::defn :user::phased [] -> :wat::core::String
  (:wat::core::let [triple (:user::run* 2000 4 3)]
    (:wat::core::format "{s}|{p}"
      :s (:wat::core::first triple)
      :p (:wat::core::third triple))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [triple (:user::run* 2000 4 3)]
    (:wat::core::let
      [_ (:wat::kernel::println
           (:wat::core::format "queue-receive-calls={c}" :c (:wat::core::second triple)))
       _ (:wat::kernel::println (:wat::core::first triple))]
      (:wat::kernel::println (:wat::core::third triple)))))

;; ★ Row 2: pending-only drain + delayed-ack worker MUST lose the held message.
(:wat::core::defn :user::pending-only-loses [] -> :wat::core::String
  (:wat::core::let
    [n 4
     msh (:wat::query::mem-store/start :locus (:wat::spawn::process)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::mem-store/grant msh (:fanout::pids pl))))
           :record (:queue::queue::Record)
           :store-addr (:wat::query::mem-store::Handle/addr msh))
     hh  (:fanout::held-worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:queue::queue/grant qh (:fanout::pids pl))))
           :record (:fanout::held-worker::Record :id "held-0" :queue-name "q0")
           :queue-addr (:queue::queue::Handle/addr qh))
     q   (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     w   (:fanout::dial-worker (:fanout::held-worker::Handle/addr hh))
     _   (:fanout::face-start w)
     _pub (:wat::core::foldl
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
     _ (:fanout::wait-pending-zero q)
     outs (:fanout::held-worker/stop hh)
     distinct (:wat::core::count
                (:wat::hashmap::keys
                  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                                     o   <- :fanout::Outcome]
                      -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                      (:wat::hashmap::assoc acc (:fanout::key-of o) true))
                    (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                    outs)))]
    (:wat::core::format
      "n={n};distinct={d};lost={lost}"
      :n n :d distinct
      :lost (:wat::core::if (:wat::i64::< distinct n) "yes" "no"))))

;; Row 5: Admin::Stop while a worker is long-polling an empty queue returns promptly.
(:wat::core::defn :user::stop-idle [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::process)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::mem-store/grant msh (:fanout::pids pl))))
           :record (:queue::queue::Record)
           :store-addr (:wat::query::mem-store::Handle/addr msh))
     wh  (:fanout::worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:queue::queue/grant qh (:fanout::pids pl))))
           :record (:fanout::worker::Record :id "idle-0" :queue-name "q0")
           :queue-addr (:queue::queue::Handle/addr qh))
     w   (:fanout::dial-worker (:fanout::worker::Handle/addr wh))
     _   (:fanout::face-start w)
     _   (:fanout::nap-ms 20)
     t0  (:wat::time::epoch-nanos (:wat::time::now))
     _   (:fanout::worker/stop wh)
     t1  (:wat::time::epoch-nanos (:wat::time::now))
     dt  (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    (:wat::core::format "dt-ms={dt}" :dt dt)))

;; ★ Row 3: drain without the outbox term MUST lose accepted-but-undelivered messages.
;; Topic delay 500ms so the outbox still holds them when queues look empty.
(:wat::core::defn :user::outbox-term-loses [] -> :wat::core::String
  (:wat::core::let
    [n 4
     msh (:wat::query::mem-store/start :locus (:wat::spawn::process)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::mem-store/grant msh (:fanout::pids pl))))
           :record (:queue::queue::Record)
           :store-addr (:wat::query::mem-store::Handle/addr msh))
     ah  (:fanout::adapter/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:queue::queue/grant qh (:fanout::pids pl))))
           :record (:fanout::adapter::Record :queue-name "q0")
           :queue-addr (:queue::queue::Handle/addr qh))
     th  (:demo::topic/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:fanout::adapter/grant ah (:fanout::pids pl))))
           :record (:demo::topic::Record :cap 16 :delay-ns 500000000)
           :addrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:demo::Sub::Op :demo::Sub::Reply])]
                    (:fanout::adapter::Handle/addr ah)))
     wh  (:fanout::worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:queue::queue/grant qh (:fanout::pids pl))))
           :record (:fanout::worker::Record :id "ob-0" :queue-name "q0")
           :queue-addr (:queue::queue::Handle/addr qh))
     topic (:fanout::dial-topic (:demo::topic::Handle/addr th))
     q     (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     w     (:fanout::dial-worker (:fanout::worker::Handle/addr wh))
     _     (:fanout::face-start w)
     _pub  (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
               (:fanout::accept! topic (:wat::core::str i)))
             nil
             (:wat::core::range 0 n))
     _     (:fanout::wait-pending-zero q)
     outs  (:fanout::worker/stop wh)
     distinct (:wat::core::count
                (:wat::hashmap::keys
                  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                                     o   <- :fanout::Outcome]
                      -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                      (:wat::hashmap::assoc acc (:fanout::key-of o) true))
                    (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                    outs)))]
    (:wat::core::format
      "n={n};distinct={d};lost={lost}"
      :n n :d distinct
      :lost (:wat::core::if (:wat::i64::< distinct n) "yes" "no"))))
