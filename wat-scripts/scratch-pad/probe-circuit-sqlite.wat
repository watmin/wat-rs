;; probe-circuit-sqlite.wat — the fan-out circuit with sqlite-store as the backend.
;; mem-store remains the oracle (five differentials). This is the measurement of
;; whether sqlite delivers the circuit. Load paths are the same as fanout/
;; (`../topic`, `../queue`) because scratch-pad is a sibling of those dirs.
;;
;; N messages → 1 topic → M queues → J workers/queue → N×M outcomes.
;; Placement: this directory (composes topic + queue; does not live inside either).
;;
;; ★ TOPOLOGY IS THE SAFETY ARGUMENT. receive is scan-index then put. What serializes
;;   those two calls is that a defservice is a serializing actor (the store's serve loop).
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

;; ── worker: process that pulls from ONE queue ───────────────────────────────────
(:wat::core::defsurface :fanout::Worker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :fanout::Outcome
     [worker <- :wat::core::String
      queue  <- :wat::core::String
      id     <- :wat::core::String
      body   <- :wat::core::String])
   (:wat::core::defrecord :fanout::Worker::DrainRequest [])
   (:wat::core::defenum :fanout::Worker::DrainResponse :wat::enum::Pure
     :Ok [outcomes <- (:wat::core::Vector :- [:fanout::Outcome])]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(drain [self <- :fanout::Worker  req <- :fanout::Worker::DrainRequest]
     -> :fanout::Worker::DrainResponse :max-request-bytes 524288)])

(:wat::service::defservice :fanout::worker
  :satisfies :fanout::Worker
  :durable   [id         <- :wat::core::String
              queue-name <- :wat::core::String
              cap        <- :wat::core::i64]
  :ephemeral [q <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])]
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
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [(drain [s ctx req]
     (:wat::core::let
       [rec  (:fanout::worker::State/durable s)
        q    (:fanout::worker::State/q s)
        name (:fanout::worker::Record/queue-name rec)
        wid  (:fanout::worker::Record/id rec)
        cap  (:fanout::worker::Record/cap rec)
        vis  1000000000000
        acc0 (:wat::core::Vector :- [:fanout::Outcome])
        outs (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::Outcome])
                                _i  <- :wat::core::i64]
                 -> (:wat::core::Vector :- [:fanout::Outcome])
                 (:wat::core::let
                   [now (:wat::time::epoch-nanos (:wat::time::now))
                    rr  (:queue::Queue/receive q
                          (:queue::Queue::ReceiveRequest
                            :queue name :now-ns now :visibility-ns vis :limit 1 :wait-ns 0))]
                   (:wat::core::match rr
                     ((:wat::kernel::RecvOutcome::Message r)
                       (:wat::core::match r
                         ((:queue::Queue::ReceiveResponse::Ok envs)
                           (:wat::core::if (:wat::core::empty? envs)
                             acc
                             (:wat::core::let
                               [e (:wat::core::first envs)
                                eid (:queue::Envelope/id e)
                                ebody (:queue::Envelope/body e)
                                ar (:queue::Queue/ack q
                                     (:queue::Queue::AckRequest :queue name :id eid))]
                               (:wat::core::match ar
                                 ((:wat::kernel::RecvOutcome::Message _ar)
                                   (:wat::core::conj acc
                                     (:fanout::Outcome :worker wid :queue name :id eid :body ebody)))
                                 (_ acc)))))
                         (_ acc)))
                     (_ acc))))
               acc0
               (:wat::core::range 0 cap))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:fanout::Worker::Reply::Drain (:fanout::Worker::DrainResponse::Ok outs))) (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::worker::Op])]))))])

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

;; Fire drain without waiting — workers run concurrently; take-drain recvs after.
(:wat::core::defn :fanout::kick-drain
  [w <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
  -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::send w
      (:fanout::Worker::Op::Drain (:fanout::Worker::DrainRequest)))
    (:wat::kernel::SendOutcome::Sent nil)
    (:wat::kernel::SendOutcome::Closed nil)
    (:wat::kernel::SendOutcome::Stopped nil)
    ((:wat::kernel::SendOutcome::Lost _c) nil)))

(:wat::core::defn :fanout::take-drain
  [w <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
  -> (:wat::core::Vector :- [:fanout::Outcome])
  (:wat::core::match (:wat::kernel::recv w)
    ((:wat::kernel::RecvOutcome::Message recvd)
      (:wat::core::match recvd
        ((:fanout::Worker::Reply::Drain resp)
          (:wat::core::match resp
            ((:fanout::Worker::DrainResponse::Ok outs) outs)
            (_ (:wat::core::Vector :- [:fanout::Outcome]))))
        (_ (:wat::kernel::assertion-failed! "fanout: misrouted drain reply" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "fanout: drain stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "fanout: drain closed" :wat::core::None :wat::core::None))))

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

;; The circuit. Wiring + input stream. Parameterized so main and the floor share it.
(:wat::core::defn :user::run
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [stores (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::sqlite-store::Handle])
                               _i  <- :wat::core::i64]
                -> (:wat::core::Vector :- [:wat::query::sqlite-store::Handle])
                (:wat::core::conj acc
                  (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
                    :record (:wat::query::sqlite-store::Record
                              :path ":memory:"
                              :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))))
              (:wat::core::Vector :- [:wat::query::sqlite-store::Handle])
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
                                   (:wat::query::sqlite-store/grant sh (:fanout::pids pl))))
                        :record (:queue::queue::Record)
                        :store-addr (:wat::query::sqlite-store::Handle/addr sh))]
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
     _pub (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
              (:wat::core::match
                (:demo::Topic/publish topic
                  (:demo::Topic::PublishRequest :msg (:wat::core::str i)))
                ((:wat::kernel::RecvOutcome::Message _r) nil)
                (_ nil)))
            nil
            (:wat::core::range 0 n))
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
                                               :queue-name (:fanout::qname qi)
                                               :cap n)
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
     _kick (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::nil  w <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])] -> :wat::core::nil
               (:fanout::kick-drain w))
             nil
             wpeers)
     outs (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::Outcome])
                             w   <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
              -> (:wat::core::Vector :- [:fanout::Outcome])
              (:wat::core::foldl
                (:wat::core::fn [a <- (:wat::core::Vector :- [:fanout::Outcome])
                                 o <- :fanout::Outcome]
                  -> (:wat::core::Vector :- [:fanout::Outcome])
                  (:wat::core::conj a o))
                acc
                (:fanout::take-drain w)))
            (:wat::core::Vector :- [:fanout::Outcome])
            wpeers)
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
                   (:wat::core::range 0 m))]
    (:fanout::summarize n m j outs empty-flags)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:user::run 12 2 2))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::run 2000 4 3)))
