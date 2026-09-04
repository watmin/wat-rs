;; wat-scripts/topic/sns-fanout.wat — wat-topic: one topic, N subscriber queues.
;;
;; Lives in userland on the wat-grep / wat-gen precedent — built here, promoted to
;; wat/topic.wat when it demonstrates excellence, and that promotion is the builder's
;; ruling. See wat-scripts/topic/README.md. Sibling: wat-scripts/queue/ (wat-queue).
;;
;; ★ WHAT THIS PROVES, and why it is one file: the SAME topic code fans out over
;;   `(:wat::spawn::thread)` and `(:wat::spawn::process)` and must deliver the SAME count.
;;   The locus is a PARAMETER, so the differential is the artifact — not something a
;;   reader has to remember to run twice. Prints "3 3". Any other pair is a defect.
;;
;; Shape: topic-service = publish surface + ONE queue-service instance + J internal
;; workers. publish writes N rows (one per subscription) into the internal queue, then
;; replies Ok — the write is the durability. Workers drain to subscriber queues and
;; ack only on Queue/send Ok; Full is "do not ack" and visibility expiry is the retry.
;; The unit is (message, subscriber), not message.
;;
;; ── the shape, established by bisecting UP from wat-scripts/probes/arc-278/s2s-process-probe.wat ──
;;   · a service MAY hold N peers of ONE surface as a `(Vector :- [(Peer :- [Op Reply])])`
;;     ephemeral field, and send to every element.
;;   · a `(Vector :- [Address])` survives a `/start` kwarg across a process fork.
;;   · on the PROCESS locus each subscriber queue must GRANT the topic-worker's pid: a
;;     queue's birth-seed allow-set holds only `getppid()` (its owner, `main`), so the
;;     worker is a STRANGER to it and is bounced until granted. See
;;     tests/services/probe_arc170_m1_teeth_admitted.wat ("served ONLY because we granted
;;     it") and probe_arc209_c0b3bb_bounced.wat. The grant rides `process/post-spawn`,
;;     which fires owner-side with the child's ProcessLaunch{pid} AFTER the fork and
;;     BEFORE `:init` ships — the grant-before-dial ordering.
;;
;; The topic itself has one scalar Queue peer (the inbox) so `:peers [:queue::Queue]` is
;; a true bijection — no anchor field. The worker holds the same scalar (its bijection)
;; plus a Vector of subscriber Queue peers; the Vector is invisible to the derivation
;; that only reads the TOP-LEVEL type head (wat/service.wat:824).

(:wat::load-file! "../queue/sqs.wat")

;; ── TOPIC ───────────────────────────────────────────────────────────────────────
;; publish means ACCEPTED, and accepted means the N rows are in the inbox store.
(:wat::core::defsurface :demo::Topic :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :demo::Topic::PublishRequest [msg <- :wat::core::String])
   (:wat::core::defenum :demo::Topic::PublishResponse :wat::enum::Pure
     :Ok []
     :Full [depth <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :demo::Topic::StatsRequest [])
   (:wat::core::defenum :demo::Topic::StatsResponse :wat::enum::Pure
     :Ok [depth <- :wat::core::i64  ticks <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(publish [self <- :demo::Topic  req <- :demo::Topic::PublishRequest]
     -> :demo::Topic::PublishResponse :max-request-bytes 524288)
   (stats   [self <- :demo::Topic  req <- :demo::Topic::StatsRequest]
     -> :demo::Topic::StatsResponse :max-request-bytes 524288)])

(:wat::service::defservice :demo::topic
  :satisfies :demo::Topic
  :durable   [nsubs <- :wat::core::i64
              inbox-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  :ephemeral [inbox <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record <- :demo::topic::Record]
          -> :demo::topic::State
          (:demo::topic::State :durable record
            :inbox
              (:wat::core::match (:wat::kernel::connect (:demo::topic::Record/inbox-addr record))
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [(publish [s ctx req]
     (:wat::core::let
       [msg   (:demo::Topic::PublishRequest/msg req)
        nsubs (:demo::topic::Record/nsubs (:demo::topic::State/durable s))
        now   (:wat::time::epoch-nanos (:wat::time::now))
        bodies (:wat::core::foldl
                 (:wat::core::fn
                   [acc <- (:wat::core::Vector :- [:wat::core::String])
                    i   <- :wat::core::i64]
                   -> (:wat::core::Vector :- [:wat::core::String])
                   (:wat::core::conj acc
                     (:wat::core::format "{i}|{m}" :i i :m msg)))
                 (:wat::core::Vector :- [:wat::core::String])
                 (:wat::core::range 0 nsubs))
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::topic::Op])])
        sr (:queue::Queue/send (:demo::topic::State/inbox s)
             (:queue::Queue::SendRequest :queue "inbox" :bodies bodies :now-ns now))]
       (:wat::core::match sr
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:queue::Queue::SendResponse::Ok)
               (:wat::service::Outcome::Continue s
                 (:wat::core::Some (:demo::Topic::Reply::Publish (:demo::Topic::PublishResponse::Ok)))
                 sends none-alarms))
             ((:queue::Queue::SendResponse::Full d c)
               (:wat::service::Outcome::Continue s
                 (:wat::core::Some (:demo::Topic::Reply::Publish (:demo::Topic::PublishResponse::Full d c)))
                 sends none-alarms))
             (_ (:wat::kernel::assertion-failed! "topic publish: send not Ok/Full" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost _cause)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:demo::topic::Record/inbox-addr (:demo::topic::State/durable s)))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "topic: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:demo::topic::State :durable (:demo::topic::State/durable s) :inbox fresh)]
             ;; Do not claim Ok — the inbox write is unknowable. Full is the caller's retry.
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:demo::Topic::Reply::Publish (:demo::Topic::PublishResponse::Full 0 0)))
               sends none-alarms)))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "topic publish: send stopped" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "topic publish: send closed" :wat::core::None :wat::core::None)))))

   (stats [s ctx req]
     (:wat::core::let
       [sends (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::Topic::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:demo::topic::Op])])
        st (:queue::Queue/stats (:demo::topic::State/inbox s)
             (:queue::Queue::StatsRequest))]
       (:wat::core::match st
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:queue::Queue::StatsResponse::Ok _calls ticks visible unacked)
               (:wat::service::Outcome::Continue s
                 (:wat::core::Some (:demo::Topic::Reply::Stats (:demo::Topic::StatsResponse::Ok
                   (:wat::i64::+ visible unacked) ticks)))
                 sends none-alarms))
             (_ (:wat::kernel::assertion-failed! "topic stats: inbox stats not Ok" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost _cause)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:demo::topic::Record/inbox-addr (:demo::topic::State/durable s)))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "topic: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:demo::topic::State :durable (:demo::topic::State/durable s) :inbox fresh)]
             ;; Conservative: not drained. -1 means unread (ticks-of already uses it).
             ;; Do not invent a depth we did not read.
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:demo::Topic::Reply::Stats (:demo::Topic::StatsResponse::Ok -1 -1)))
               sends none-alarms)))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "topic stats: stopped" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "topic stats: closed" :wat::core::None :wat::core::None)))))])

;; ── internal worker ────────────────────────────────────────────────────────────
;; Shape of :fanout::worker: park on the inbox, take a batch, act, ack. Failure
;; handling is "do not ack" — visibility expiry is the retry. No counter.
(:wat::core::defsurface :demo::TopicWorker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :demo::TopicWorker::StartRequest [])
   (:wat::core::defenum :demo::TopicWorker::StartResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start [self <- :demo::TopicWorker  req <- :demo::TopicWorker::StartRequest]
     -> :demo::TopicWorker::StartResponse :max-request-bytes 524288)])

(:wat::service::defservice :demo::topic-worker
  :satisfies :demo::TopicWorker
  :durable   [vis-ns <- :wat::core::i64
              inbox-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])
              sub-addrs  <- (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])]
  :ephemeral [inbox <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
              subs  <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record <- :demo::topic-worker::Record]
          -> :demo::topic-worker::State
          (:demo::topic-worker::State :durable record
            :inbox
              (:wat::core::match (:wat::kernel::connect (:demo::topic-worker::Record/inbox-addr record))
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
            :subs
              (:wat::core::foldl
                (:wat::core::fn
                  [acc <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])
                   a   <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
                  -> (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])
                  (:wat::core::conj acc
                    (:wat::core::match (:wat::kernel::connect a)
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                      ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                      ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
                (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])
                (:demo::topic-worker::Record/sub-addrs record))))
  :impls
  [(start [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:demo::TopicWorker::Reply::Start (:demo::TopicWorker::StartResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::TopicWorker::Reply])])
       [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]))
   (-tick [s ctx]
     (:wat::core::let
       [rec   (:demo::topic-worker::State/durable s)
        inbox (:demo::topic-worker::State/inbox s)
        subs  (:demo::topic-worker::State/subs s)
        vis   (:demo::topic-worker::Record/vis-ns rec)
        now   (:wat::time::epoch-nanos (:wat::time::now))
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:demo::TopicWorker::Reply])])
        rr (:queue::Queue/receive inbox
             (:queue::Queue::ReceiveRequest
               :queue "inbox" :now-ns now :visibility-ns vis :limit 10 :wait (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 250))))]
       (:wat::core::match rr
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:queue::Queue::ReceiveResponse::Ok envs)
               (:wat::core::let
                 [nsubs (:wat::core::count subs)
                  empty-bucket (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
                  empty-buckets (:wat::core::foldl
                                  (:wat::core::fn
                                    [acc <- (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                                     _i  <- :wat::core::i64]
                                    -> (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                                    (:wat::core::conj acc empty-bucket))
                                  (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                                  (:wat::core::range 0 nsubs))
                  t1 (:wat::time::epoch-nanos (:wat::time::now))
                  buckets (:wat::core::foldl
                            (:wat::core::fn
                              [acc <- (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                               e   <- :queue::Envelope]
                              -> (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                              (:wat::core::let
                                [eid   (:queue::Envelope/id e)
                                 body  (:queue::Envelope/body e)
                                 parts (:wat::string::split body "|")
                                 nparts (:wat::core::count parts)]
                                (:wat::core::if (:wat::i64::< nparts 2)
                                  (:wat::kernel::assertion-failed! "topic-worker: body missing idx prefix" :wat::core::None :wat::core::None)
                                  (:wat::core::let
                                    [idx (:wat::edn::read (:wat::core::nth parts 0))
                                     rest (:wat::core::foldl
                                            (:wat::core::fn [a <- :wat::core::String  i <- :wat::core::i64]
                                              -> :wat::core::String
                                              (:wat::core::let [p (:wat::core::nth parts i)]
                                                (:wat::core::if (:wat::core::= a "")
                                                  p
                                                  (:wat::string::concat a (:wat::string::concat "|" p)))))
                                            ""
                                            (:wat::core::range 1 nparts))
                                     t3 (:wat::time::epoch-nanos (:wat::time::now))
                                     stamped (:wat::core::format "{b}|{t1}|{t2}|{t3}" :b rest :t1 t1 :t2 t1 :t3 t3)
                                     pair (:wat::core::Tuple eid stamped)]
                                    (:wat::core::foldl
                                      (:wat::core::fn
                                        [bacc <- (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                                         i    <- :wat::core::i64]
                                        -> (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                                        (:wat::core::conj bacc
                                          (:wat::core::if (:wat::core::= i idx)
                                            (:wat::core::conj (:wat::core::nth acc i) pair)
                                            (:wat::core::nth acc i))))
                                      (:wat::core::Vector :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])])
                                      (:wat::core::range 0 nsubs))))))
                            empty-buckets
                            envs)
                  peers (:wat::core::foldl
                      (:wat::core::fn [acc <- (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                                     (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])])
                                       i   <- :wat::core::i64]
                        -> (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                  (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])])
                        (:wat::core::let
                          [inb (:wat::core::first acc)
                           ss  (:wat::core::second acc)
                           bucket (:wat::core::nth buckets i)]
                          (:wat::core::if (:wat::core::empty? bucket)
                            acc
                            (:wat::core::let
                              [bodies (:wat::core::foldl
                                        (:wat::core::fn
                                          [bacc <- (:wat::core::Vector :- [:wat::core::String])
                                           p    <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])]
                                          -> (:wat::core::Vector :- [:wat::core::String])
                                          (:wat::core::conj bacc (:wat::core::second p)))
                                        (:wat::core::Vector :- [:wat::core::String])
                                        bucket)
                               qpeer (:wat::core::nth ss i)
                               qname (:wat::core::format "q{i}" :i i)
                               sr (:queue::Queue/send qpeer
                                    (:queue::Queue::SendRequest :queue qname :bodies bodies :now-ns t1))]
                              (:wat::core::match sr
                                ((:wat::kernel::RecvOutcome::Message sresp)
                                  (:wat::core::match sresp
                                    ((:queue::Queue::SendResponse::Ok)
                                      (:wat::core::let
                                        [inb2 (:wat::core::foldl
                                                (:wat::core::fn
                                                  [inb0 <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                   p    <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])]
                                                  -> (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                  (:wat::core::match
                                                    (:queue::Queue/ack inb0
                                                      (:queue::Queue::AckRequest :queue "inbox" :id (:wat::core::first p)))
                                                    ((:wat::kernel::RecvOutcome::Message _ar) inb0)
                                                    ((:wat::kernel::RecvOutcome::Lost _cause)
                                                      (:wat::core::match
                                                        (:wat::kernel::connect (:demo::topic-worker::Record/inbox-addr rec))
                                                        ((:wat::kernel::ConnectOutcome::Connected p) p)
                                                        (_ (:wat::kernel::assertion-failed! "topic-worker: redial inbox failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))))
                                                    (:wat::kernel::RecvOutcome::Stopped
                                                      (:wat::kernel::assertion-failed! "topic-worker: ack stopped" :wat::core::None :wat::core::None))
                                                    (:wat::kernel::RecvOutcome::Closed
                                                      (:wat::kernel::assertion-failed! "topic-worker: ack closed" :wat::core::None :wat::core::None))))
                                                inb
                                                bucket)]
                                        (:wat::core::Tuple inb2 ss)))
                                    ((:queue::Queue::SendResponse::Full _d _c) acc)
                                    (_ (:wat::kernel::assertion-failed! "topic-worker: send not Ok/Full" :wat::core::None :wat::core::None))))
                                ((:wat::kernel::RecvOutcome::Lost _cause)
                                  ;; Hard site: this sub may have taken the batch. Do not ack
                                  ;; the bucket — visibility redelivers; Seen absorbs if it landed.
                                  (:wat::core::let
                                    [fresh (:wat::core::match
                                             (:wat::kernel::connect (:wat::core::nth (:demo::topic-worker::Record/sub-addrs rec) i))
                                             ((:wat::kernel::ConnectOutcome::Connected p) p)
                                             (_ (:wat::kernel::assertion-failed! "topic-worker: redial sub failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                                     ss' (:wat::core::foldl
                                           (:wat::core::fn
                                             [bacc <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])
                                              j    <- :wat::core::i64]
                                             -> (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])
                                             (:wat::core::conj bacc
                                               (:wat::core::if (:wat::core::= j i) fresh (:wat::core::nth ss j))))
                                           (:wat::core::Vector :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])])
                                           (:wat::core::range 0 nsubs))]
                                    (:wat::core::Tuple inb ss')))
                                (:wat::kernel::RecvOutcome::Stopped
                                  (:wat::kernel::assertion-failed! "topic-worker: send stopped" :wat::core::None :wat::core::None))
                                (:wat::kernel::RecvOutcome::Closed
                                  (:wat::kernel::assertion-failed! "topic-worker: send closed" :wat::core::None :wat::core::None)))))))
                      (:wat::core::Tuple inbox subs)
                      (:wat::core::range 0 nsubs))
                  s' (:demo::topic-worker::State
                       :durable rec
                       :inbox (:wat::core::first peers)
                       :subs (:wat::core::second peers))]
                 (:wat::service::SelfOutcome::Continue s'
                   none-sends
                   [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)])))
             (_ (:wat::kernel::assertion-failed! "topic-worker: receive not Ok" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost _cause)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:demo::topic-worker::Record/inbox-addr rec))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "topic-worker: redial inbox failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:demo::topic-worker::State :durable rec :inbox fresh :subs subs)]
             (:wat::service::SelfOutcome::Continue s'
               none-sends
               [(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)])))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "topic-worker: receive stopped" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "topic-worker: receive closed" :wat::core::None :wat::core::None)))))])

;; ── parent-side helpers ────────────────────────────────────────────────────────
(:wat::core::defn :demo::dial-topic
  [a <- (:wat::kernel::Address :- [:demo::Topic::Op :demo::Topic::Reply])]
  -> (:wat::kernel::Peer :- [:demo::Topic::Op :demo::Topic::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::dial-queue
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::dial-topic-worker
  [a <- (:wat::kernel::Address :- [:demo::TopicWorker::Op :demo::TopicWorker::Reply])]
  -> (:wat::kernel::Peer :- [:demo::TopicWorker::Op :demo::TopicWorker::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::pids [pl <- :wat::spawn::ProcessLaunch]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :demo::face-start-tw
  [w <- (:wat::kernel::Peer :- [:demo::TopicWorker::Op :demo::TopicWorker::Reply])]
  -> :wat::core::nil
  (:wat::core::match (:demo::TopicWorker/start w (:demo::TopicWorker::StartRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::TopicWorker::StartResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "topic-worker: start not Ok" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost _cause) nil)
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "topic-worker: start stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "topic-worker: start closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::nap-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

;; Sentinel: -1 means unread. A dead peer must not look like work. ticks-of
;; already returned -1; q-depth / depth-of-topic join it.
(:wat::core::defn :demo::depth-of-topic [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok n _ticks) n)
        (_ -1)))
    (_ -1)))

(:wat::core::defn :demo::ticks-of [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok _n ticks) ticks)
        (_ -1)))
    (_ -1)))

(:wat::core::defn :demo::require!
  [r <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:wat::core::= r "")
    nil
    (:wat::kernel::assertion-failed! r :wat::core::None :wat::core::None)))

(:wat::core::defn :demo::elapsed-ms [start-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns) 1000000))

(:wat::core::defn :demo::poll-until-inbox-zero*
  [t <- :demo::Topic  left <- :wat::core::i64  start-ns <- :wat::core::i64  total <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let [n (:demo::depth-of-topic t)]
    (:wat::core::if (:wat::core::= n -1)
      (:wat::core::format "inbox-unread: last={n} attempts={a} elapsed={ms}"
        :n n :a (:wat::i64::- total left) :ms (:demo::elapsed-ms start-ns))
      (:wat::core::if (:wat::core::= n 0)
        ""
        (:wat::core::if (:wat::i64::<= left 1)
          (:wat::core::format "inbox-never-zero: last={n} attempts={a} elapsed={ms}"
            :n n :a total :ms (:demo::elapsed-ms start-ns))
          (:wat::core::let [_ (:demo::nap-ms 1)]
            (:demo::poll-until-inbox-zero* t (:wat::i64::- left 1) start-ns total)))))))

(:wat::core::defn :demo::poll-until-inbox-zero
  [t <- :demo::Topic  attempts <- :wat::core::i64] -> :wat::core::String
  (:demo::poll-until-inbox-zero* t attempts (:wat::time::epoch-nanos (:wat::time::now)) attempts))

(:wat::core::defn :demo::accept!
  [t <- :demo::Topic  msg <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:demo::Topic/publish t (:demo::Topic::PublishRequest :msg msg))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::PublishResponse::Ok) nil)
        ((:demo::Topic::PublishResponse::Full _d _c)
          (:wat::core::let [_ (:demo::nap-ms 1)]
            (:demo::accept! t msg)))
        (_ (:wat::kernel::assertion-failed! "topic publish not Ok/Full" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "topic publish recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::q-depth
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible unacked)
          (:wat::core::Tuple visible unacked))
        (_ (:wat::core::Tuple -1 -1))))
    (_ (:wat::core::Tuple -1 -1))))

(:wat::core::defn :demo::poll-until-unacked*
  [q <- :queue::Queue  left <- :wat::core::i64  start-ns <- :wat::core::i64  total <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [d (:demo::q-depth q)
     v (:wat::core::first d)
     u (:wat::core::second d)]
    (:wat::core::if (:wat::core::= u -1)
      (:wat::core::format "unacked-unread: last={v}/{u} attempts={a} elapsed={ms}"
        :v v :u u :a (:wat::i64::- total left) :ms (:demo::elapsed-ms start-ns))
      (:wat::core::if (:wat::i64::>= u 1)
        ""
        (:wat::core::if (:wat::i64::<= left 1)
          (:wat::core::format "unacked-never-rose: last={v}/{u} attempts={a} elapsed={ms}"
            :v v :u u :a total :ms (:demo::elapsed-ms start-ns))
          (:wat::core::let [_ (:demo::nap-ms 1)]
            (:demo::poll-until-unacked* q (:wat::i64::- left 1) start-ns total)))))))

(:wat::core::defn :demo::poll-until-unacked
  [q <- :queue::Queue  attempts <- :wat::core::i64] -> :wat::core::String
  (:demo::poll-until-unacked* q attempts (:wat::time::epoch-nanos (:wat::time::now)) attempts))

;; Presence: one receive, arrives on the wire, nothing to eat. Visibility is a
;; required argument — the hold must be visible at the call site.
(:wat::core::defn :demo::receive-blocking
  [q <- :queue::Queue  name <- :wat::core::String  vis-ns <- :wat::core::i64  wait <- :queue::Queue::Wait]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue name :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns vis-ns :limit 1 :wait wait))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs)
          (:wat::core::if (:wat::core::empty? envs)
            ""
            (:queue::Envelope/id (:wat::core::first envs))))
        (_ (:wat::kernel::assertion-failed! "receive-blocking: not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive-blocking: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::claim-one!
  [q <- :queue::Queue  name <- :wat::core::String  vis-ns <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue name :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns vis-ns :limit 1 :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs)
          (:wat::core::if (:wat::core::empty? envs)
            ""
            (:queue::Envelope/id (:wat::core::first envs))))
        (_ (:wat::kernel::assertion-failed! "claim-one!: receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "claim-one!: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::ack-one
  [q <- :queue::Queue  name <- :wat::core::String  id <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match
    (:queue::Queue/ack q (:queue::Queue::AckRequest :queue name :id id))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "ack-one failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :demo::send-one
  [q <- :queue::Queue  name <- :wat::core::String  body <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest :queue name
        :bodies (:wat::core::Vector :- [:wat::core::String] body)
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "send-one not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "send-one recv failed" :wat::core::None :wat::core::None))))

;; THREAD: no grant needed — a thread-tier queue shares the parent's admission.
(:wat::core::defn :demo::run-thread [] -> :wat::core::i64
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     stores (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::mem-store::Handle])
                               _i  <- :wat::core::i64]
                -> (:wat::core::Vector :- [:wat::query::mem-store::Handle])
                (:wat::core::conj acc
                  (:wat::query::mem-store/start :locus (:wat::spawn::thread)
                    :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))))
              (:wat::core::Vector :- [:wat::query::mem-store::Handle])
              (:wat::core::range 0 3))
     queues (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::queue::Handle])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [:queue::queue::Handle])
                (:wat::core::conj acc
                  (:queue::queue/start :locus (:wat::spawn::thread)
                    :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr (:wat::core::nth stores i))))))
              (:wat::core::Vector :- [:queue::queue::Handle])
              (:wat::core::range 0 3))
     qaddrs (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
                (:wat::core::conj acc (:queue::queue::Handle/addr (:wat::core::nth queues i))))
              (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
              (:wat::core::range 0 3))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 3 :inbox-addr (:queue::queue::Handle/addr iqh)))
     wh (:demo::topic-worker/start :locus (:wat::spawn::thread)
          :record (:demo::topic-worker::Record :vis-ns 200000000
                    :inbox-addr (:queue::queue::Handle/addr iqh)
                    :sub-addrs qaddrs))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     tw (:demo::dial-topic-worker (:demo::topic-worker::Handle/addr wh))
     qclients (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::Queue])
                                 i   <- :wat::core::i64]
                  -> (:wat::core::Vector :- [:queue::Queue])
                  (:wat::core::conj acc
                    (:demo::dial-queue (:queue::queue::Handle/addr (:wat::core::nth queues i)))))
                (:wat::core::Vector :- [:queue::Queue])
                (:wat::core::range 0 3))
     _ (:demo::face-start-tw tw)
     _ (:demo::accept! tc "hello")
     _ (:demo::require! (:demo::poll-until-inbox-zero tc 5000))
     got (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::let [d (:demo::q-depth (:wat::core::nth qclients i))]
               (:wat::core::if (:wat::i64::> (:wat::i64::+ (:wat::core::first d) (:wat::core::second d)) 0)
                 (:wat::i64::+ acc 1)
                 acc)))
           0
           (:wat::core::range 0 3))]
    got))

;; PROCESS: inbox grants the topic (send) and the worker (receive); each subscriber
;; queue grants the worker (send).
(:wat::core::defn :demo::run-process [] -> :wat::core::i64
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::process)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::mem-store/grant ish (:demo::pids pl))))
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     stores (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::mem-store::Handle])
                               _i  <- :wat::core::i64]
                -> (:wat::core::Vector :- [:wat::query::mem-store::Handle])
                (:wat::core::conj acc
                  (:wat::query::mem-store/start :locus (:wat::spawn::process)
                    :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))))
              (:wat::core::Vector :- [:wat::query::mem-store::Handle])
              (:wat::core::range 0 3))
     queues (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::queue::Handle])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [:queue::queue::Handle])
                (:wat::core::let
                  [sh (:wat::core::nth stores i)
                   h (:queue::queue/start
                        :locus (:wat::spawn::process/post-spawn
                                 (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                   (:wat::query::mem-store/grant sh (:demo::pids pl))))
                        :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr sh)))]
                  (:wat::core::conj acc h)))
              (:wat::core::Vector :- [:queue::queue::Handle])
              (:wat::core::range 0 3))
     qaddrs (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
                (:wat::core::conj acc (:queue::queue::Handle/addr (:wat::core::nth queues i))))
              (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
              (:wat::core::range 0 3))
     th (:demo::topic/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:queue::queue/grant iqh (:demo::pids pl))))
          :record (:demo::topic::Record :nsubs 3 :inbox-addr (:queue::queue::Handle/addr iqh)))
     wh (:demo::topic-worker/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::core::let
                       [pids (:demo::pids pl)
                        _ (:queue::queue/grant iqh pids)]
                       (:wat::core::foldl
                         (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
                           (:queue::queue/grant (:wat::core::nth queues i) pids))
                         nil
                         (:wat::core::range 0 3)))))
          :record (:demo::topic-worker::Record :vis-ns 200000000
                    :inbox-addr (:queue::queue::Handle/addr iqh)
                    :sub-addrs qaddrs))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     tw (:demo::dial-topic-worker (:demo::topic-worker::Handle/addr wh))
     qclients (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::Queue])
                                 i   <- :wat::core::i64]
                  -> (:wat::core::Vector :- [:queue::Queue])
                  (:wat::core::conj acc
                    (:demo::dial-queue (:queue::queue::Handle/addr (:wat::core::nth queues i)))))
                (:wat::core::Vector :- [:queue::Queue])
                (:wat::core::range 0 3))
     _ (:demo::face-start-tw tw)
     _ (:demo::accept! tc "hello")
     _ (:demo::require! (:demo::poll-until-inbox-zero tc 5000))
     got (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::let [d (:demo::q-depth (:wat::core::nth qclients i))]
               (:wat::core::if (:wat::i64::> (:wat::i64::+ (:wat::core::first d) (:wat::core::second d)) 0)
                 (:wat::i64::+ acc 1)
                 acc)))
           0
           (:wat::core::range 0 3))]
    got))

;; 3 3 — thread and process must agree. Printed by wat-scripts/topic/run.wat
;; (`set-redef!` lives there so this file can be load-file!'d).
(:wat::core::defn :user::loci [] -> :wat::core::String
  (:wat::core::let
    [t (:demo::run-thread)
     p (:demo::run-process)]
    (:wat::string::concat (:wat::core::str t)
      (:wat::string::concat " " (:wat::core::str p)))))

;; ── gates ─────────────────────────────────────────────────────────────────────
;; Row 1: publish, then read inbox depth before any worker runs. The message is in
;; the store — a crash would not lose it.
(:wat::core::defn :user::durable-ok [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     _  (:demo::accept! tc "hello")
     n  (:demo::depth-of-topic tc)]
    (:wat::core::format "pending={n};durable={d}"
      :n n
      :d (:wat::core::if (:wat::i64::>= n 1) "yes" "no"))))

;; Row 2: one publish to N=3 writes 3 rows, not 1.
(:wat::core::defn :user::unit-is-per-sub [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 3 :inbox-addr (:queue::queue::Handle/addr iqh)))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     _  (:demo::accept! tc "hello")
     n  (:demo::depth-of-topic tc)]
    (:wat::core::format "rows={n};unit={u}"
      :n n
      :u (:wat::core::if (:wat::core::= n 3) "per-sub" "per-msg"))))

;; Publish returns after the inbox write, with no workers running — so even a
;; subscriber that would take 200ms cannot hold the publisher.
(:wat::core::defn :user::publish-is-async [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::match (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "hello"))
          ((:wat::kernel::RecvOutcome::Message _r) nil)
          (_ nil))
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     dt (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    (:wat::core::format "dt-ms={dt};prompt={p}"
      :dt dt
      :p (:wat::core::if (:wat::i64::< dt 100) "yes" "no"))))

;; A full inbox refuses. No workers, so nothing drains. cap 2, nsubs=1: third is Full.
(:wat::core::defn :user::inbox-refuses [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 2 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     r1 (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "a"))
     r2 (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "b"))
     r3 (:demo::Topic/publish tc (:demo::Topic::PublishRequest :msg "c"))
     tag (:wat::core::fn [rr <- (:wat::kernel::RecvOutcome :- [:demo::Topic::PublishResponse])] -> :wat::core::String
           (:wat::core::match rr
             ((:wat::kernel::RecvOutcome::Message r)
               (:wat::core::match r
                 ((:demo::Topic::PublishResponse::Ok) "ok")
                 ((:demo::Topic::PublishResponse::Full _d _c) "full")
                 (_ "other")))
             (_ "fail")))]
    (:wat::core::format "a={a};b={b};c={c}" :a (tag r1) :b (tag r2) :c (tag r3))))

;; Idle topic never ticks: no waiters on the inbox, so the queue does not arm.
(:wat::core::defn :user::idle-ticks [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     _  (:demo::nap-ms 20)
     n  (:demo::ticks-of tc)]
    (:wat::core::format "ticks={n}" :n n)))

;; Row 3: a full subscriber is retried via visibility, not dropped. Cap 1 filled with
;; a dummy (cap 0 never accepts). Worker hits Full, does not ack; after the dummy is
;; drained and vis expires, the message arrives. No retry counter.
(:wat::core::defn :user::refused-is-retried [] -> :wat::core::String
  (:user::refused-is-retried-gap 0))

(:wat::core::defn :user::refused-is-retried-gap [gap-ms <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     ssh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     sqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 1 :store-addr (:wat::query::mem-store::Handle/addr ssh)))
     qaddrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
              (:queue::queue::Handle/addr sqh))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     wh (:demo::topic-worker/start :locus (:wat::spawn::thread)
          :record (:demo::topic-worker::Record :vis-ns 200000000
                    :inbox-addr (:queue::queue::Handle/addr iqh)
                    :sub-addrs qaddrs))
     inbox (:demo::dial-queue (:queue::queue::Handle/addr iqh))
     subq  (:demo::dial-queue (:queue::queue::Handle/addr sqh))
     tc    (:demo::dial-topic (:demo::topic::Handle/addr th))
     tw    (:demo::dial-topic-worker (:demo::topic-worker::Handle/addr wh))
     _ (:demo::send-one subq "q0" "dummy")
     _ (:demo::face-start-tw tw)
     _ (:demo::accept! tc "hello")
     _ (:demo::require! (:demo::poll-until-unacked inbox 2000))
     dummy-id (:demo::claim-one! subq "q0" 1000000000000)
     _ (:demo::ack-one subq "q0" dummy-id)
     _ (:wat::core::if (:wat::i64::> gap-ms 0) (:demo::nap-ms gap-ms) nil)
     after-visible (:wat::core::first (:demo::q-depth subq))
     _ (:demo::nap-ms 350)
     after-expiry (:demo::receive-blocking subq "q0" 1000000000000
                    (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 2000)))]
    (:wat::core::format "inflight=yes;after-drain={a};after-expiry={b}"
      :a (:wat::core::if (:wat::core::= after-visible 0) "none"
           (:wat::core::if (:wat::i64::< after-visible 0) "unread" "got"))
      :b (:wat::core::if (:wat::core::= after-expiry "") "none" "got"))))

;; Row 4: N=2, one subscriber full. The healthy one receives immediately; publish
;; does not block.
(:wat::core::defn :user::stalled-does-not-stall [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr ish)))
     s0 (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     q0h (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 32 :store-addr (:wat::query::mem-store::Handle/addr s0)))
     s1 (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     q1h (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 1 :store-addr (:wat::query::mem-store::Handle/addr s1)))
     qaddrs (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
              (:queue::queue::Handle/addr q0h) (:queue::queue::Handle/addr q1h))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 2 :inbox-addr (:queue::queue::Handle/addr iqh)))
     wh (:demo::topic-worker/start :locus (:wat::spawn::thread)
          :record (:demo::topic-worker::Record :vis-ns 200000000
                    :inbox-addr (:queue::queue::Handle/addr iqh)
                    :sub-addrs qaddrs))
     q0 (:demo::dial-queue (:queue::queue::Handle/addr q0h))
     q1 (:demo::dial-queue (:queue::queue::Handle/addr q1h))
     tc (:demo::dial-topic (:demo::topic::Handle/addr th))
     tw (:demo::dial-topic-worker (:demo::topic-worker::Handle/addr wh))
     _ (:demo::send-one q1 "q1" "dummy")
     _ (:demo::face-start-tw tw)
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _ (:demo::accept! tc "hello")
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     dt (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)
     healthy (:demo::receive-blocking q0 "q0" 1000000000000
                (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 2000)))
     stalled-v (:wat::core::first (:demo::q-depth q1))]
    (:wat::core::format "healthy={h};stalled={s};dt-ms={dt};blocked={b}"
      :h (:wat::core::if (:wat::core::= healthy "") "none" "got")
      :s (:wat::core::if (:wat::i64::>= stalled-v 1) "held" "none")
      :dt dt
      :b (:wat::core::if (:wat::i64::< dt 100) "no" "yes"))))
