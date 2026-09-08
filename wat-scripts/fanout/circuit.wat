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
;; ★ A DUPLICATE IS A FINDING, and it is a MESSAGE duplicate. Envelope ids are minted
;;   per send (`uuid::v4`), so keying on queue/envelope-id cannot see a redelivery.
;;   Identity is the published seq (first field of the body). Workers claim it on a
;;   shared seen-service; a second delivery is acked and dropped. distinct counts
;;   (queue, seq). Loss still shortens distinct. Do not dedupe in the queue.
;;
;; Composition: load-file! the shipped topic and queue programs (they each have
;; :user::main). set-redef! lets this file's main win. The topic owns ONE inbox
;; queue plus J internal workers that Queue/send to subscriber queues and ack
;; only on Ok. No adapter — workers call Queue/send directly so Full is "do not
;; ack" (visibility expiry), not a blocked in-flight Sub/deliver.
;;
;; Shape: start workers (consume immediately, on empty queues) → publish alongside
;; them → drain on depth (visible = 0 AND unacked = 0 AND topic inbox = 0) →
;; Admin::Stop; tallies return via Status::Stopped. Publish means accepted; the
;; write is the N inbox rows. A completion check must cover every place a
;; message can rest — the inbox is the new one.
;;
;; :user::main  → N=2000 M=4 J=3 (standalone weight)
;; :user::compute → N=12 M=2 J=2 (floor; same wiring)
;;
;; Store is sqlite-store. mem-store remains the differential oracle in
;; wat-scripts/queue/sqs.wat :user::compute. The sqlite probe copy was
;; the same transform and is gone.

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")
(:wat::load-file! "../queue/sqs.wat")

;; ── seen: the consumer's shared identity set. ONE instance; J workers DIAL it.
;; Claim is First or Dup. At-least-once stays the queue's contract.
(:wat::core::defsurface :fanout::Seen :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defenum :fanout::Seen::Verdict :wat::enum::Pure
     :Recorded []
     :Absent [])
   (:wat::core::defrecord :fanout::Seen::CheckRequest
     [queue <- :wat::core::String
      seqs  <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :fanout::Seen::CheckResponse :wat::enum::Pure
     :Ok [hits <- (:wat::core::Vector :- [:fanout::Seen::Verdict])]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :fanout::Seen::MarkRequest
     [queue <- :wat::core::String
      seqs  <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :fanout::Seen::MarkResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :fanout::Seen::StatsRequest [])
   (:wat::core::defenum :fanout::Seen::StatsResponse :wat::enum::Pure
     :Ok [recorded <- :wat::core::i64  skipped <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(check [self <- :fanout::Seen  req <- :fanout::Seen::CheckRequest]
     -> :fanout::Seen::CheckResponse :max-request-bytes 524288)
   (mark [self <- :fanout::Seen  req <- :fanout::Seen::MarkRequest]
     -> :fanout::Seen::MarkResponse :max-request-bytes 524288)
   (stats [self <- :fanout::Seen  req <- :fanout::Seen::StatsRequest]
     -> :fanout::Seen::StatsResponse :max-request-bytes 524288)])

(:wat::service::defservice :fanout::seen
  :satisfies :fanout::Seen
  ;; 256: small enough that a 2KB disrupt claim severs ONE worker's connection.
  ;; Normal claims (`q0/1999`) fit. Contract cap stays 524288. Thread locus
  ;; does not tear — that is a property, not a second mechanism.
  :max-frame-bytes 256
  ;; Counters are durable so a stats read is a fact about this run.
  ;; claimed stays ephemeral: the receipt does not cross the wire and does not
  ;; survive hibernation (S31). Restart seen and every message looks Absent again.
  :durable   [recorded       <- :wat::core::i64
              skipped        <- :wat::core::i64
              drop-check-bp  <- :wat::core::i64
              drop-mark-bp   <- :wat::core::i64
              drop-seed      <- :wat::core::i64
              drop-after?    <- :wat::core::bool]
  :ephemeral [claimed <- (:wat::core::PersistentMap :- [:wat::core::String :wat::core::bool])]
  :init (:wat::core::fn [record <- :fanout::seen::Record] -> :fanout::seen::State
          (:fanout::seen::State :durable record
            :claimed (:wat::core::PersistentMap :- [:wat::core::String :wat::core::bool])))
  :impls
  [(check [s ctx req]
     (:wat::core::let
       [qname (:fanout::Seen::CheckRequest/queue req)
        seqs  (:fanout::Seen::CheckRequest/seqs req)
        claimed (:fanout::seen::State/claimed s)
        rec     (:fanout::seen::State/durable s)
        rate   (:fanout::seen::Record/drop-check-bp rec)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Seen::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::seen::Op])])
        _cap (:wat::core::if (:wat::i64::> (:wat::core::count seqs) 10)
                (:wat::kernel::assertion-failed! "seen.check: batch larger than 10" :wat::core::None :wat::core::None)
                nil)
        pair (:wat::core::if (:wat::i64::> rate 0)
               (:wat::rand::int-from (:fanout::seen::Record/drop-seed rec) 0 10000)
               (:wat::core::Tuple (:fanout::seen::Record/drop-seed rec) 0))
        seed1 (:wat::core::first pair)
        bp    (:wat::core::second pair)
        hit?  (:wat::core::and (:wat::i64::> rate 0) (:wat::i64::< bp rate))
        hits (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::Seen::Verdict])  seq <- :wat::core::String]
                 -> (:wat::core::Vector :- [:fanout::Seen::Verdict])
                 (:wat::core::let
                   [key (:wat::string::concat qname (:wat::string::concat "/" seq))
                    already? (:wat::core::match (:wat::map::get claimed key)
                               ((:wat::core::Some _) true)
                               (:wat::core::None false))]
                   (:wat::core::conj acc
                     (:wat::core::if already?
                       (:fanout::Seen::Verdict::Recorded)
                       (:fanout::Seen::Verdict::Absent)))))
               (:wat::core::Vector :- [:fanout::Seen::Verdict])
               seqs)
        rec' (:fanout::seen::Record
               :recorded (:fanout::seen::Record/recorded rec)
               :skipped (:fanout::seen::Record/skipped rec)
               :drop-check-bp rate
               :drop-mark-bp (:fanout::seen::Record/drop-mark-bp rec)
               :drop-seed seed1
               :drop-after? (:fanout::seen::Record/drop-after? rec))
        s' (:fanout::seen::State :durable rec' :claimed claimed)
        reply (:wat::core::if hit?
                :wat::core::None
                (:wat::core::Some (:fanout::Seen::Reply::Check (:fanout::Seen::CheckResponse::Ok hits))))]
       (:wat::service::Outcome::Continue s' reply sends none-alarms)))
   (mark [s ctx req]
     (:wat::core::let
       [qname (:fanout::Seen::MarkRequest/queue req)
        seqs  (:fanout::Seen::MarkRequest/seqs req)
        rec0    (:fanout::seen::State/durable s)
        rate   (:fanout::seen::Record/drop-mark-bp rec0)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Seen::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::seen::Op])])
        _cap (:wat::core::if (:wat::i64::> (:wat::core::count seqs) 10)
                (:wat::kernel::assertion-failed! "seen.mark: batch larger than 10" :wat::core::None :wat::core::None)
                nil)
        pair (:wat::core::if (:wat::i64::> rate 0)
               (:wat::rand::int-from (:fanout::seen::Record/drop-seed rec0) 0 10000)
               (:wat::core::Tuple (:fanout::seen::Record/drop-seed rec0) 0))
        seed1 (:wat::core::first pair)
        bp    (:wat::core::second pair)
        hit?  (:wat::core::and (:wat::i64::> rate 0) (:wat::i64::< bp rate))
        folded (:wat::core::foldl
                 (:wat::core::fn
                   [acc <- (:wat::core::Tuple :- [(:wat::core::PersistentMap :- [:wat::core::String :wat::core::bool])
                                                  :wat::core::i64 :wat::core::i64])
                    seq <- :wat::core::String]
                   -> (:wat::core::Tuple :- [(:wat::core::PersistentMap :- [:wat::core::String :wat::core::bool])
                                             :wat::core::i64 :wat::core::i64])
                   (:wat::core::let
                     [claimed (:wat::core::first acc)
                      recd    (:wat::core::second acc)
                      skip    (:wat::core::third acc)
                      key (:wat::string::concat qname (:wat::string::concat "/" seq))
                      already? (:wat::core::match (:wat::map::get claimed key)
                                 ((:wat::core::Some _) true)
                                 (:wat::core::None false))]
                     (:wat::core::if already?
                       (:wat::core::Tuple claimed recd (:wat::i64::+ skip 1))
                       (:wat::core::Tuple (:wat::map::assoc claimed key true) (:wat::i64::+ recd 1) skip))))
                 (:wat::core::Tuple
                   (:fanout::seen::State/claimed s)
                   (:fanout::seen::Record/recorded rec0)
                   (:fanout::seen::Record/skipped rec0))
                 seqs)
        rec' (:fanout::seen::Record
               :recorded (:wat::core::second folded) :skipped (:wat::core::third folded)
               :drop-check-bp (:fanout::seen::Record/drop-check-bp rec0)
               :drop-mark-bp rate :drop-seed seed1
               :drop-after? (:fanout::seen::Record/drop-after? rec0))
        s' (:fanout::seen::State :durable rec' :claimed (:wat::core::first folded))
        reply (:wat::core::if hit?
                :wat::core::None
                (:wat::core::Some (:fanout::Seen::Reply::Mark (:fanout::Seen::MarkResponse::Ok))))]
       (:wat::service::Outcome::Continue s' reply sends none-alarms)))
   (stats [s ctx req]
     (:wat::core::let
       [rec (:fanout::seen::State/durable s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Seen::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::seen::Op])])]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:fanout::Seen::Reply::Stats
           (:fanout::Seen::StatsResponse::Ok
             (:fanout::seen::Record/recorded rec)
             (:fanout::seen::Record/skipped rec))))
         sends none-alarms)))])

;; Silent server for showing timeout → discard → redial → retry on a FRESH peer.
;; Never settles. Not used by the circuit; `:user::deadline-redial-is-fresh` only.
(:wat::core::defsurface :fanout::Hold :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :fanout::Hold::WaitRequest [])
   (:wat::core::defenum :fanout::Hold::WaitResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(wait [self <- :fanout::Hold  req <- :fanout::Hold::WaitRequest]
     -> :fanout::Hold::WaitResponse :max-request-bytes 65536)])

(:wat::service::defservice :fanout::hold
  :satisfies :fanout::Hold
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :fanout::hold::Record] -> :fanout::hold::State
          (:fanout::hold::State :durable record))
  :impls
  [(wait [s ctx req]
     (:wat::service::Outcome::Continue s
       :wat::core::None
       (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Hold::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::hold::Op])])))])

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
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :fanout::Worker::DisruptsRequest [])
   (:wat::core::defenum :fanout::Worker::DisruptsResponse :wat::enum::Pure
     :Ok [hits <- :wat::core::i64  draws <- :wat::core::i64  points <- :wat::core::String
          check-exhausted <- :wat::core::i64  mark-exhausted <- :wat::core::i64
          ack-retries <- :wat::core::i64  ack-exhausted <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   ;; Nullary closed enums. Outcome crosses; Peer/Reply stay beside it (arc 293.W.2b).
   (:wat::core::defenum :fanout::SeenRetry :wat::enum::Pure
     :Got []
     :Exhausted [attempts <- :wat::core::i64])
   (:wat::core::defenum :fanout::QueueRetry :wat::enum::Pure
     :Got []
     :Exhausted [attempts <- :wat::core::i64])]
  :features
  [(start [self <- :fanout::Worker  req <- :fanout::Worker::StartRequest]
     -> :fanout::Worker::StartResponse :max-request-bytes 524288)
   (disrupts [self <- :fanout::Worker  req <- :fanout::Worker::DisruptsRequest]
     -> :fanout::Worker::DisruptsResponse :max-request-bytes 524288)])

(:wat::service::defservice :fanout::worker
  :satisfies :fanout::Worker
  :durable   [id         <- :wat::core::String
              queue-name <- :wat::core::String
              vis-ns     <- :wat::core::i64
              ack-delay-ms  <- :wat::core::i64
              work-delay-ms <- :wat::core::i64
              queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])
              seen-addr  <- (:wat::kernel::Address :- [:fanout::Seen::Op :fanout::Seen::Reply])
              disrupt-rate-bp   <- :wat::core::i64
              disrupt-seed      <- :wat::core::i64
              disrupt-lo-ms     <- :wat::core::i64
              disrupt-hi-ms     <- :wat::core::i64
              disrupt-max-draws <- :wat::core::i64
              disrupt-hits      <- :wat::core::i64
              disrupt-draws     <- :wat::core::i64
              disrupt-points    <- :wat::core::String
              check-exhausted   <- :wat::core::i64
              mark-exhausted    <- :wat::core::i64
              ack-retries       <- :wat::core::i64
              ack-exhausted     <- :wat::core::i64]
  :ephemeral [q        <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
              seen     <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
              outcomes <- (:wat::core::PersistentVector :- [:fanout::Outcome])]
  :peers     [:queue::Queue :fanout::Seen]
  :init (:wat::core::fn
          [record <- :fanout::worker::Record]
          -> :fanout::worker::State
          (:fanout::worker::State :durable record
            :q (:wat::core::match (:wat::kernel::connect (:fanout::worker::Record/queue-addr record))
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 ((:wat::kernel::ConnectOutcome::Refused c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Rejected c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                 ((:wat::kernel::ConnectOutcome::Failed c)
                   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
            :seen (:wat::core::match (:wat::kernel::connect (:fanout::worker::Record/seen-addr record))
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
     ;; Rate 0 arms nothing. Rate > 0 draws a first delay and arms -disrupt.
     ;; 3c-pre's always-on poison in start is gone — that was a proof instrument.
     (:wat::core::let
       [rec  (:fanout::worker::State/durable s)
        rate (:fanout::worker::Record/disrupt-rate-bp rec)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])])
        tick (:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)]
       (:wat::core::if (:wat::i64::> rate 0)
         (:wat::core::let
           [pair (:wat::rand::int-from (:fanout::worker::Record/disrupt-seed rec)
                    (:fanout::worker::Record/disrupt-lo-ms rec)
                    (:fanout::worker::Record/disrupt-hi-ms rec))
            seed1 (:wat::core::first pair)
            delay (:wat::core::second pair)
            rec'  (:fanout::worker::Record
                    :id (:fanout::worker::Record/id rec)
                    :queue-name (:fanout::worker::Record/queue-name rec)
                    :vis-ns (:fanout::worker::Record/vis-ns rec)
                    :ack-delay-ms (:fanout::worker::Record/ack-delay-ms rec)
                    :work-delay-ms (:fanout::worker::Record/work-delay-ms rec)
                    :queue-addr (:fanout::worker::Record/queue-addr rec)
                    :seen-addr (:fanout::worker::Record/seen-addr rec)
                    :disrupt-rate-bp rate
                    :disrupt-seed seed1
                    :disrupt-lo-ms (:fanout::worker::Record/disrupt-lo-ms rec)
                    :disrupt-hi-ms (:fanout::worker::Record/disrupt-hi-ms rec)
                    :disrupt-max-draws (:fanout::worker::Record/disrupt-max-draws rec)
                    :disrupt-hits (:fanout::worker::Record/disrupt-hits rec)
                    :disrupt-draws (:fanout::worker::Record/disrupt-draws rec)
                    :disrupt-points (:fanout::worker::Record/disrupt-points rec)
                    :check-exhausted (:fanout::worker::Record/check-exhausted rec)
                    :mark-exhausted (:fanout::worker::Record/mark-exhausted rec)
                    :ack-retries (:fanout::worker::Record/ack-retries rec)
                    :ack-exhausted (:fanout::worker::Record/ack-exhausted rec))
            s' (:fanout::worker::State :durable rec'
                 :q (:fanout::worker::State/q s)
                 :seen (:fanout::worker::State/seen s)
                 :outcomes (:fanout::worker::State/outcomes s))]
           (:wat::service::Outcome::Continue s'
             (:wat::core::Some (:fanout::Worker::Reply::Start (:fanout::Worker::StartResponse::Ok)))
             none-sends
             [tick (:wat::service::Alarm :delay (:wat::time::Milliseconds delay) :op :-disrupt)]))
         (:wat::service::Outcome::Continue s
           (:wat::core::Some (:fanout::Worker::Reply::Start (:fanout::Worker::StartResponse::Ok)))
           none-sends
           [tick]))))
   (disrupts [s ctx req]
     (:wat::core::let
       [rec (:fanout::worker::State/durable s)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])])
        none-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::worker::Op])])]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:fanout::Worker::Reply::Disrupts
           (:fanout::Worker::DisruptsResponse::Ok
             (:fanout::worker::Record/disrupt-hits rec)
             (:fanout::worker::Record/disrupt-draws rec)
             (:fanout::worker::Record/disrupt-points rec)
             (:fanout::worker::Record/check-exhausted rec)
             (:fanout::worker::Record/mark-exhausted rec)
             (:fanout::worker::Record/ack-retries rec)
             (:fanout::worker::Record/ack-exhausted rec))))
         none-sends none-arms)))
   (-disrupt [s ctx]
     (:wat::core::let
       [rec   (:fanout::worker::State/durable s)
        old   (:fanout::worker::State/seen s)
        rate  (:fanout::worker::Record/disrupt-rate-bp rec)
        lo    (:fanout::worker::Record/disrupt-lo-ms rec)
        hi    (:fanout::worker::Record/disrupt-hi-ms rec)
        maxd  (:fanout::worker::Record/disrupt-max-draws rec)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])])
        draw1 (:wat::rand::int-from (:fanout::worker::Record/disrupt-seed rec) 0 10000)
        seed1 (:wat::core::first draw1)
        bp    (:wat::core::second draw1)
        draws (:wat::i64::+ (:fanout::worker::Record/disrupt-draws rec) 1)
        hit?  (:wat::i64::< bp rate)
        pad   (:wat::core::foldl
                (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
                  (:wat::string::concat acc "xxxxxxxxxx"))
                "" (:wat::core::range 0 200))
        poisoned (:wat::core::if hit?
                    (:wat::core::match
                      (:fanout::Seen/check old
                        (:fanout::Seen::CheckRequest :queue "disrupt"
                          :seqs (:wat::core::Vector :- [:wat::core::String] pad)))
                      ((:wat::kernel::RecvOutcome::Message _r) "message")
                      ((:wat::kernel::RecvOutcome::Lost _c) "lost")
                      (:wat::kernel::RecvOutcome::Closed "closed")
                      (:wat::kernel::RecvOutcome::Stopped
                        (:wat::kernel::assertion-failed! "fanout worker: disrupt poison stopped" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut "lost"))
                    "miss")
        tore? (:wat::core::or (:wat::core::= poisoned "lost") (:wat::core::= poisoned "closed"))
        seen' (:wat::core::if tore?
                (:wat::core::match (:wat::kernel::connect (:fanout::worker::Record/seen-addr rec))
                  ((:wat::kernel::ConnectOutcome::Connected p) p)
                  (_ (:wat::kernel::assertion-failed! "fanout worker: redial seen failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                old)
        hits' (:wat::core::if tore?
                (:wat::i64::+ (:fanout::worker::Record/disrupt-hits rec) 1)
                (:fanout::worker::Record/disrupt-hits rec))
        points' (:wat::core::if tore?
                  (:wat::core::format "{p}{d},"
                    :p (:fanout::worker::Record/disrupt-points rec) :d draws)
                  (:fanout::worker::Record/disrupt-points rec))
        draw2 (:wat::rand::int-from seed1 lo hi)
        seed2 (:wat::core::first draw2)
        delay (:wat::core::second draw2)
        rec'  (:fanout::worker::Record
                :id (:fanout::worker::Record/id rec)
                :queue-name (:fanout::worker::Record/queue-name rec)
                :vis-ns (:fanout::worker::Record/vis-ns rec)
                :ack-delay-ms (:fanout::worker::Record/ack-delay-ms rec)
                :work-delay-ms (:fanout::worker::Record/work-delay-ms rec)
                :queue-addr (:fanout::worker::Record/queue-addr rec)
                :seen-addr (:fanout::worker::Record/seen-addr rec)
                :disrupt-rate-bp rate
                :disrupt-seed seed2
                :disrupt-lo-ms lo
                :disrupt-hi-ms hi
                :disrupt-max-draws maxd
                :disrupt-hits hits'
                :disrupt-draws draws
                :disrupt-points points'
                :check-exhausted (:fanout::worker::Record/check-exhausted rec)
                :mark-exhausted (:fanout::worker::Record/mark-exhausted rec)
                :ack-retries (:fanout::worker::Record/ack-retries rec)
                :ack-exhausted (:fanout::worker::Record/ack-exhausted rec))
        s' (:fanout::worker::State :durable rec'
             :q (:fanout::worker::State/q s) :seen seen'
             :outcomes (:fanout::worker::State/outcomes s))
        rearm? (:wat::core::or (:wat::core::= maxd 0) (:wat::i64::< draws maxd))
        arms (:wat::core::if rearm?
               [(:wat::service::Alarm :delay (:wat::time::Milliseconds delay) :op :-disrupt)]
               (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::worker::Op])]))]
       (:wat::service::SelfOutcome::Continue s' none-sends arms)))
   ;; Park, don't poll. :wait :UpTo 250 ms is the idle wait. An empty return is
   ;; "nothing yet" — re-arm so the serve loop can take Admin::Stop. The
   ;; queue/topic now arm from state (level-triggered); the 1 ms after a
   ;; park is the Stop yield, not the idle poll.
   (-tick [s ctx]
     (:wat::core::let
       [rec  (:fanout::worker::State/durable s)
        q    (:fanout::worker::State/q s)
        seen (:fanout::worker::State/seen s)
        name (:fanout::worker::Record/queue-name rec)
        wid  (:fanout::worker::Record/id rec)
        vis  (:fanout::worker::Record/vis-ns rec)
        ack-delay (:fanout::worker::Record/ack-delay-ms rec)
        work-delay (:fanout::worker::Record/work-delay-ms rec)
        outs (:fanout::worker::State/outcomes s)
        now  (:wat::time::epoch-nanos (:wat::time::now))
        empty-envs (:wat::core::Vector :- [:queue::Envelope])
        recv-req (:queue::Queue::ReceiveRequest
                   :queue name :now-ns now :visibility-ns vis :limit 10 :wait (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 250)))
        recv-got (:wat::service::call-by-deadline q
                   (:queue::Queue::Op::Receive recv-req) 1000
                   (:queue::Queue::Reply::Receive
                     (:queue::Queue::ReceiveResponse::Ok empty-envs)))]
       (:wat::core::match recv-got
         ((:wat::service::CallOutcome::Answered r)
           (:wat::core::match r
             ((:queue::Queue::Reply::Receive (:queue::Queue::ReceiveResponse::Ok envs))
               (:wat::core::let
                 [t4 (:wat::time::epoch-nanos (:wat::time::now))
                  ;; check-all → emit the absent → mark those → ack all. One round
                  ;; trip each. Receipt still written after emit (STOP-2).
                  triple (:wat::core::if (:wat::core::empty? envs)
                           (:wat::core::Tuple (:wat::core::Tuple q seen outs) (:wat::core::Tuple (:wat::core::Tuple 0 0) (:wat::core::Tuple 0 0)))
                           (:wat::core::let
                             [addr (:fanout::worker::Record/seen-addr rec)
                              seqs (:wat::core::foldl
                                     (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  e <- :queue::Envelope]
                                       -> (:wat::core::Vector :- [:wat::core::String])
                                       (:wat::core::let [parts (:wat::string::split (:queue::Envelope/body e) "|")]
                                         (:wat::core::conj acc (:wat::core::if (:wat::core::empty? parts) "" (:wat::core::first parts)))))
                                     (:wat::core::Vector :- [:wat::core::String])
                                     envs)
                              ids (:wat::core::foldl
                                    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  e <- :queue::Envelope]
                                      -> (:wat::core::Vector :- [:wat::core::String])
                                      (:wat::core::conj acc (:queue::Envelope/id e)))
                                    (:wat::core::Vector :- [:wat::core::String])
                                    envs)
                              redial (:wat::core::fn []
                                        -> (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                                        (:wat::core::match (:wat::kernel::connect addr)
                                          ((:wat::kernel::ConnectOutcome::Connected p) p)
                                          (_ (:wat::kernel::assertion-failed! "fanout worker: redial seen failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))))
                              inert-check (:fanout::Seen::Reply::Check
                                            (:fanout::Seen::CheckResponse::Ok
                                              (:wat::core::Vector :- [:fanout::Seen::Verdict])))
                              check-req (:fanout::Seen::CheckRequest :queue name :seqs seqs)
                              await-ms
                                (:wat::core::fn [ms <- :wat::core::i64] -> :wat::core::nil
                                  (:wat::core::match
                                    (:wat::kernel::recv
                                      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
                                    ((:wat::kernel::RecvOutcome::Message _m) nil)
                                    ((:wat::kernel::RecvOutcome::Lost _c) nil)
                                    (:wat::kernel::RecvOutcome::Stopped nil)
                                    (:wat::kernel::RecvOutcome::Closed nil)
                                    (:wat::kernel::RecvOutcome::TimedOut nil)))
                              ;; limit-ms = vis-ns / 1000000. After vis expiry the
                              ;; message is visible again and retrying is pointless.
                              limit-ms (:wat::i64::/ vis 1000000)
                              ;; Returns (outcome, peer, reply). Outcome crosses; peer stays.
                              seen-until
                                (:wat::core::fn
                                  [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                                   op <- :fanout::Seen::Op
                                   inert <- :fanout::Seen::Reply]
                                  -> (:wat::core::Tuple :- [:fanout::SeenRetry
                                                           (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                                                           :fanout::Seen::Reply])
                                  (:wat::core::match (:wat::service::call-by-deadline peer op 200 inert)
                                    ((:wat::service::CallOutcome::Answered r)
                                      (:wat::core::Tuple (:fanout::SeenRetry::Got) peer r))
                                    (_
                                      (:wat::core::let
                                        [elapsed0 (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) now) 1000000)]
                                        (:wat::core::if (:wat::i64::>= elapsed0 limit-ms)
                                          (:wat::core::Tuple (:fanout::SeenRetry::Exhausted 0) (redial) inert)
                                          (:wat::core::let
                                            [drawn0 (:wat::rand::int-from now 1 2)
                                             seed0 (:wat::core::first drawn0)
                                             d0 (:wat::core::second drawn0)
                                             _nap0 (await-ms d0)
                                             st0 (:wat::core::Tuple
                                                    (:wat::core::Tuple (redial) 1)
                                                    (:wat::core::Tuple seed0 1 false)
                                                    inert)
                                             st
                                               (:wat::core::foldl
                                                 (:wat::core::fn
                                                   [st <- (:wat::core::Tuple :- [(:wat::core::Tuple :- [(:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply]) :wat::core::i64])
                                                                                (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::bool])
                                                                                :fanout::Seen::Reply])
                                                    _i <- :wat::core::i64]
                                                   -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [(:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply]) :wat::core::i64])
                                                                             (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::bool])
                                                                             :fanout::Seen::Reply])
                                                   (:wat::core::let
                                                     [left (:wat::core::first st)
                                                      right (:wat::core::second st)
                                                      reply0 (:wat::core::third st)
                                                      p0 (:wat::core::first left)
                                                      retries (:wat::core::second left)
                                                      sd (:wat::core::first right)
                                                      attempt (:wat::core::second right)
                                                      done (:wat::core::third right)]
                                                     (:wat::core::if done
                                                       st
                                                       (:wat::core::match (:wat::service::call-by-deadline p0 op 200 inert)
                                                         ((:wat::service::CallOutcome::Answered r)
                                                           (:wat::core::Tuple left (:wat::core::Tuple sd attempt true) r))
                                                         (_
                                                           (:wat::core::let
                                                             [elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) now) 1000000)]
                                                             (:wat::core::if (:wat::i64::>= elapsed limit-ms)
                                                               (:wat::core::Tuple
                                                                 (:wat::core::Tuple (redial) retries)
                                                                 (:wat::core::Tuple sd attempt true)
                                                                 reply0)
                                                               (:wat::core::let
                                                                 [shifted (:wat::core::if (:wat::i64::>= attempt 7)
                                                                            100
                                                                            (:wat::core::foldl
                                                                              (:wat::core::fn [a <- :wat::core::i64  _j <- :wat::core::i64] -> :wat::core::i64
                                                                                (:wat::i64::* a 2))
                                                                              1
                                                                              (:wat::core::range 0 attempt)))
                                                                  ceiling (:wat::core::if (:wat::i64::> shifted 100) 100 shifted)
                                                                  drawn1 (:wat::rand::int-from sd 1 (:wat::i64::+ ceiling 1))
                                                                  seed1 (:wat::core::first drawn1)
                                                                  d (:wat::core::second drawn1)
                                                                  _nap (await-ms d)]
                                                                 ;; one increment = one receive-limit-10 batch retry, not per message
                                                                 (:wat::core::Tuple
                                                                   (:wat::core::Tuple (redial) (:wat::i64::+ retries 1))
                                                                   (:wat::core::Tuple seed1 (:wat::i64::+ attempt 1) false)
                                                                   reply0)))))))))
                                                 st0
                                                 (:wat::core::range 0 256))
                                             done1 (:wat::core::third (:wat::core::second st))
                                             elapsed-f (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) now) 1000000)]
                                            (:wat::core::if (:wat::core::and done1 (:wat::i64::< elapsed-f limit-ms))
                                              (:wat::core::Tuple (:fanout::SeenRetry::Got)
                                                (:wat::core::first (:wat::core::first st))
                                                (:wat::core::third st))
                                              (:wat::core::Tuple (:fanout::SeenRetry::Exhausted (:wat::core::second (:wat::core::first st)))
                                                (:wat::core::first (:wat::core::first st))
                                                (:wat::core::third st)))))))))
                              check-pack (seen-until seen (:fanout::Seen::Op::Check check-req) inert-check)
                              seen1 (:wat::core::second check-pack)
                              sreply (:wat::core::third check-pack)]
                             (:wat::core::match (:wat::core::first check-pack)
                               ((:fanout::SeenRetry::Exhausted _att)
                                 (:wat::core::Tuple (:wat::core::Tuple q seen1 outs) (:wat::core::Tuple (:wat::core::Tuple 1 0) (:wat::core::Tuple 0 0))))
                               ((:fanout::SeenRetry::Got)
                                 (:wat::core::match sreply
                                   ((:fanout::Seen::Reply::Check cresp)
                                     (:wat::core::match cresp
                                     ((:fanout::Seen::CheckResponse::Ok hits)
                                       (:wat::core::if (:wat::core::not (:wat::i64::= (:wat::core::count hits) (:wat::core::count envs)))
                                         (:wat::kernel::assertion-failed! "fanout worker: check hits not aligned to envs" :wat::core::None :wat::core::None)
                                         (:wat::core::let
                                           [outs1 (:wat::core::foldl
                                                      (:wat::core::fn
                                                        [outs0 <- (:wat::core::PersistentVector :- [:fanout::Outcome])
                                                         i   <- :wat::core::i64]
                                                        -> (:wat::core::PersistentVector :- [:fanout::Outcome])
                                                        (:wat::core::let
                                                          [e (:wat::core::nth envs i)
                                                           v (:wat::core::nth hits i)
                                                           absent? (:wat::core::match v
                                                                     ((:fanout::Seen::Verdict::Absent) true)
                                                                     ((:fanout::Seen::Verdict::Recorded) false)
                                                                     (_ (:wat::kernel::assertion-failed! "fanout worker: check not Absent/Recorded" :wat::core::None :wat::core::None)))
                                                           eid (:queue::Envelope/id e)
                                                           raw (:queue::Envelope/body e)
                                                           ebody (:wat::core::format "{b}|{t}" :b raw :t t4)
                                                           _work-nap (:wat::core::if (:wat::core::and (:wat::i64::> work-delay 0) absent?)
                                                                       (:wat::core::match
                                                                         (:wat::kernel::recv
                                                                           (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds work-delay) :done))
                                                                         ((:wat::kernel::RecvOutcome::Message _m) nil)
                                                                         (_ nil))
                                                                       nil)]
                                                          (:wat::core::if absent?
                                                            (:wat::vector::conj outs0
                                                              (:fanout::Outcome :worker wid :queue name :id eid :body ebody))
                                                            outs0)))
                                                      outs
                                                      (:wat::core::range 0 (:wat::core::count envs)))
                                            inert-mark (:fanout::Seen::Reply::Mark (:fanout::Seen::MarkResponse::Ok))
                                            ;; Mark the whole checked set after emit. Absents become
                                            ;; receipts; already-Recorded seqs increment skipped so
                                            ;; an absorbed redelivery is counted (the floor gate).
                                            mark-op (:fanout::Seen::Op::Mark
                                                      (:fanout::Seen::MarkRequest :queue name :seqs seqs))
                                            mark-pack (seen-until seen1 mark-op inert-mark)
                                            seen2 (:wat::core::second mark-pack)]
                                           (:wat::core::match (:wat::core::first mark-pack)
                                             ((:fanout::SeenRetry::Exhausted _matt)
                                               (:wat::core::Tuple (:wat::core::Tuple q seen2 outs1) (:wat::core::Tuple (:wat::core::Tuple 0 1) (:wat::core::Tuple 0 0))))
                                             ((:fanout::SeenRetry::Got)
                                               (:wat::core::let
                                                 [_nap (:wat::core::if (:wat::i64::> ack-delay 0)
                                                    (:wat::core::match
                                                      (:wat::kernel::recv
                                                        (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ack-delay) :done))
                                                      ((:wat::kernel::RecvOutcome::Message _m) nil)
                                                      (_ nil))
                                                    nil)
                                            redial-q (:wat::core::fn []
                                                        -> (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                        (:wat::core::match
                                                          (:wat::kernel::connect (:fanout::worker::Record/queue-addr rec))
                                                          ((:wat::kernel::ConnectOutcome::Connected p) p)
                                                          (_ (:wat::kernel::assertion-failed! "fanout worker: redial queue failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))))
                                            inert-ack (:queue::Queue::Reply::Ack (:queue::Queue::AckResponse::Ok))
                                            ack-op (:queue::Queue::Op::Ack
                                                      (:queue::Queue::AckRequest :queue name :ids ids))
                                            await-ms
                                              (:wat::core::fn [ms <- :wat::core::i64] -> :wat::core::nil
                                                (:wat::core::match
                                                  (:wat::kernel::recv
                                                    (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
                                                  ((:wat::kernel::RecvOutcome::Message _m) nil)
                                                  ((:wat::kernel::RecvOutcome::Lost _c) nil)
                                                  (:wat::kernel::RecvOutcome::Stopped nil)
                                                  (:wat::kernel::RecvOutcome::Closed nil)
                                                  (:wat::kernel::RecvOutcome::TimedOut nil)))
                                            ;; limit-ms = vis-ns / 1000000. After vis expiry the
                                            ;; message is visible again and retrying is pointless.
                                            ack-limit-ms (:wat::i64::/ vis 1000000)
                                            ack-start-ns (:wat::time::epoch-nanos (:wat::time::now))
                                            ack-first (:wat::service::call-by-deadline q ack-op 200 inert-ack)
                                            ack-pair
                                              (:wat::core::match ack-first
                                                ((:wat::service::CallOutcome::Answered _r)
                                                  (:wat::core::Tuple (:fanout::QueueRetry::Got) q 0))
                                                (_
                                                  (:wat::core::let
                                                    [elapsed0 (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) ack-start-ns) 1000000)]
                                                    (:wat::core::if (:wat::i64::>= elapsed0 ack-limit-ms)
                                                      (:wat::core::Tuple (:fanout::QueueRetry::Exhausted 0) (redial-q) 0)
                                                      (:wat::core::let
                                                        [drawn0 (:wat::rand::int-from ack-start-ns 1 2)
                                                         seed0 (:wat::core::first drawn0)
                                                         d0 (:wat::core::second drawn0)
                                                         _nap0 (await-ms d0)
                                                         ack-st0 (:wat::core::Tuple
                                                                    (:wat::core::Tuple (redial-q) 1)
                                                                    (:wat::core::Tuple seed0 1 false))
                                                         ack-st
                                                           (:wat::core::foldl
                                                             (:wat::core::fn
                                                               [st <- (:wat::core::Tuple :- [(:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply]) :wat::core::i64])
                                                                                            (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::bool])])
                                                                _i <- :wat::core::i64]
                                                               -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply]) :wat::core::i64])
                                                                                         (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::bool])])
                                                               (:wat::core::let
                                                                 [left (:wat::core::first st)
                                                                  right (:wat::core::second st)
                                                                  peer (:wat::core::first left)
                                                                  retries (:wat::core::second left)
                                                                  sd (:wat::core::first right)
                                                                  attempt (:wat::core::second right)
                                                                  done (:wat::core::third right)]
                                                                 (:wat::core::if done
                                                                   st
                                                                   (:wat::core::match
                                                                     (:wat::service::call-by-deadline peer ack-op 200 inert-ack)
                                                                     ((:wat::service::CallOutcome::Answered _r)
                                                                       (:wat::core::Tuple left (:wat::core::Tuple sd attempt true)))
                                                                     (_
                                                                       (:wat::core::let
                                                                         [elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) ack-start-ns) 1000000)]
                                                                         (:wat::core::if (:wat::i64::>= elapsed ack-limit-ms)
                                                                           (:wat::core::Tuple
                                                                             (:wat::core::Tuple (redial-q) retries)
                                                                             (:wat::core::Tuple sd attempt true))
                                                                           (:wat::core::let
                                                                             [shifted (:wat::core::if (:wat::i64::>= attempt 7)
                                                                                        100
                                                                                        (:wat::core::foldl
                                                                                          (:wat::core::fn [a <- :wat::core::i64  _j <- :wat::core::i64] -> :wat::core::i64
                                                                                            (:wat::i64::* a 2))
                                                                                          1
                                                                                          (:wat::core::range 0 attempt)))
                                                                              ceiling (:wat::core::if (:wat::i64::> shifted 100) 100 shifted)
                                                                              drawn (:wat::rand::int-from sd 1 (:wat::i64::+ ceiling 1))
                                                                              seed1 (:wat::core::first drawn)
                                                                              d (:wat::core::second drawn)
                                                                              _nap (await-ms d)]
                                                                             ;; one increment = one receive-limit-10 batch retry, not per message
                                                                             (:wat::core::Tuple
                                                                               (:wat::core::Tuple (redial-q) (:wat::i64::+ retries 1))
                                                                               (:wat::core::Tuple seed1 (:wat::i64::+ attempt 1) false))))))))))
                                                             ack-st0
                                                             ;; 8192 × 200 ms > vis 10^12 ns; elapsed check stops first.
                                                             (:wat::core::range 0 8192))
                                                         ack-done (:wat::core::third (:wat::core::second ack-st))
                                                         ack-elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) ack-start-ns) 1000000)]
                                                        (:wat::core::if (:wat::core::and ack-done (:wat::i64::< ack-elapsed ack-limit-ms))
                                                          (:wat::core::Tuple (:fanout::QueueRetry::Got)
                                                            (:wat::core::first (:wat::core::first ack-st))
                                                            (:wat::core::second (:wat::core::first ack-st)))
                                                          (:wat::core::Tuple (:fanout::QueueRetry::Exhausted (:wat::core::second (:wat::core::first ack-st)))
                                                            (:wat::core::first (:wat::core::first ack-st))
                                                            (:wat::core::second (:wat::core::first ack-st)))))))))
                                            ack-out (:wat::core::first ack-pair)
                                            q-acked (:wat::core::second ack-pair)
                                            ar-tick (:wat::core::third ack-pair)]
                                                 (:wat::core::match ack-out
                                                   ((:fanout::QueueRetry::Got)
                                                     (:wat::core::Tuple (:wat::core::Tuple q-acked seen2 outs1) (:wat::core::Tuple (:wat::core::Tuple 0 0) (:wat::core::Tuple ar-tick 0))))
                                                   ((:fanout::QueueRetry::Exhausted att)
                                                     (:wat::core::Tuple (:wat::core::Tuple q-acked seen2 outs1) (:wat::core::Tuple (:wat::core::Tuple 0 0) (:wat::core::Tuple att 1)))))))))))
                                     (_ (:wat::kernel::assertion-failed! "fanout worker: check not Ok" :wat::core::None :wat::core::None))))
                                   (_ (:wat::kernel::assertion-failed! "fanout worker: check reply misrouted" :wat::core::None :wat::core::None)))))))
                  folded (:wat::core::first triple)
                  tick-pair (:wat::core::second triple)
                  ce-tick (:wat::core::first (:wat::core::first tick-pair))
                  me-tick (:wat::core::second (:wat::core::first tick-pair))
                  ar-tick (:wat::core::first (:wat::core::second tick-pair))
                  ae-tick (:wat::core::second (:wat::core::second tick-pair))
                  rec' (:wat::core::if (:wat::core::or (:wat::i64::> ce-tick 0)
                                        (:wat::core::or (:wat::i64::> me-tick 0)
                                          (:wat::core::or (:wat::i64::> ar-tick 0) (:wat::i64::> ae-tick 0))))
                         (:fanout::worker::Record
                           :id (:fanout::worker::Record/id rec)
                           :queue-name (:fanout::worker::Record/queue-name rec)
                           :vis-ns (:fanout::worker::Record/vis-ns rec)
                           :ack-delay-ms (:fanout::worker::Record/ack-delay-ms rec)
                           :work-delay-ms (:fanout::worker::Record/work-delay-ms rec)
                           :queue-addr (:fanout::worker::Record/queue-addr rec)
                           :seen-addr (:fanout::worker::Record/seen-addr rec)
                           :disrupt-rate-bp (:fanout::worker::Record/disrupt-rate-bp rec)
                           :disrupt-seed (:fanout::worker::Record/disrupt-seed rec)
                           :disrupt-lo-ms (:fanout::worker::Record/disrupt-lo-ms rec)
                           :disrupt-hi-ms (:fanout::worker::Record/disrupt-hi-ms rec)
                           :disrupt-max-draws (:fanout::worker::Record/disrupt-max-draws rec)
                           :disrupt-hits (:fanout::worker::Record/disrupt-hits rec)
                           :disrupt-draws (:fanout::worker::Record/disrupt-draws rec)
                           :disrupt-points (:fanout::worker::Record/disrupt-points rec)
                           :check-exhausted (:wat::i64::+ (:fanout::worker::Record/check-exhausted rec) ce-tick)
                           :mark-exhausted (:wat::i64::+ (:fanout::worker::Record/mark-exhausted rec) me-tick)
                           :ack-retries (:wat::i64::+ (:fanout::worker::Record/ack-retries rec) ar-tick)
                           :ack-exhausted (:wat::i64::+ (:fanout::worker::Record/ack-exhausted rec) ae-tick))
                         rec)
                  s' (:fanout::worker::State :durable rec'
                       :q (:wat::core::first folded)
                       :seen (:wat::core::second folded)
                       :outcomes (:wat::core::third folded))]
                 (:wat::service::SelfOutcome::Continue s'
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)])))
             (_ (:wat::kernel::assertion-failed! "fanout worker: receive not Ok" :wat::core::None :wat::core::None))))
         ((:wat::service::CallOutcome::Lost _c)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:fanout::worker::Record/queue-addr rec))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "fanout worker: redial queue failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:fanout::worker::State :durable rec :q fresh :seen seen :outcomes outs)]
             (:wat::service::SelfOutcome::Continue s'
               (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)])))
         ((:wat::service::CallOutcome::Closed)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:fanout::worker::Record/queue-addr rec))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "fanout worker: redial queue failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:fanout::worker::State :durable rec :q fresh :seen seen :outcomes outs)]
             (:wat::service::SelfOutcome::Continue s'
               (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)])))
         ((:wat::service::CallOutcome::DeadlineFired)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:fanout::worker::Record/queue-addr rec))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "fanout worker: redial queue failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:fanout::worker::State :durable rec :q fresh :seen seen :outcomes outs)]
             (:wat::service::SelfOutcome::Continue s'
               (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)]))))))])

;; Delayed-ack worker: receive this tick, ack the next. Row 2 removes the in-flight
;; term from the drain condition and requires a loss — same-tick ack would hide it.
(:wat::service::defservice :fanout::held-worker
  :satisfies :fanout::Worker
  :durable   [id         <- :wat::core::String
              queue-name <- :wat::core::String
              queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  :ephemeral [q        <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
              outcomes <- (:wat::core::Vector :- [:fanout::Outcome])
              held     <- (:wat::core::Vector :- [:queue::Envelope])]
  :peers     [:queue::Queue]
  :init (:wat::core::fn
          [record <- :fanout::held-worker::Record]
          -> :fanout::held-worker::State
          (:fanout::held-worker::State :durable record
            :q (:wat::core::match (:wat::kernel::connect (:fanout::held-worker::Record/queue-addr record))
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
       (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)]))
   (disrupts [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:fanout::Worker::Reply::Disrupts
         (:fanout::Worker::DisruptsResponse::Ok 0 0 "" 0 0 0 0)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::held-worker::Op])])))
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
           [pair (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                                   (:wat::core::Vector :- [:fanout::Outcome])])
                                     e   <- :queue::Envelope]
                      -> (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                (:wat::core::Vector :- [:fanout::Outcome])])
                      (:wat::core::let
                        [q0    (:wat::core::first acc)
                         outs0 (:wat::core::second acc)
                         eid   (:queue::Envelope/id e)
                         ebody (:queue::Envelope/body e)
                         ar    (:queue::Queue/ack q0
                                 (:queue::Queue::AckRequest :queue name
                                   :ids (:wat::core::Vector :- [:wat::core::String] eid)))]
                        (:wat::core::match ar
                          ((:wat::kernel::RecvOutcome::Message _ar)
                            (:wat::core::Tuple q0
                              (:wat::core::conj outs0
                                (:fanout::Outcome :worker wid :queue name :id eid :body ebody))))
                          ((:wat::kernel::RecvOutcome::Lost _cause)
                            ;; Do not record; do not retry the ack. Vis is the retry.
                            (:wat::core::Tuple
                              (:wat::core::match
                                (:wat::kernel::connect (:fanout::held-worker::Record/queue-addr rec))
                                ((:wat::kernel::ConnectOutcome::Connected p) p)
                                (_ (:wat::kernel::assertion-failed! "held-worker: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                              outs0))
                          (:wat::kernel::RecvOutcome::Stopped
                            (:wat::kernel::assertion-failed! "held-worker: ack stopped" :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Closed
                            ;; Do not record; do not retry the ack. Vis is the retry.
                            (:wat::core::Tuple
                              (:wat::core::match
                                (:wat::kernel::connect (:fanout::held-worker::Record/queue-addr rec))
                                ((:wat::kernel::ConnectOutcome::Connected p) p)
                                (_ (:wat::kernel::assertion-failed! "held-worker: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                              outs0)) (:wat::kernel::RecvOutcome::TimedOut (:wat::core::Tuple (:wat::core::match (:wat::kernel::connect (:fanout::held-worker::Record/queue-addr rec)) ((:wat::kernel::ConnectOutcome::Connected p) p) (_ (:wat::kernel::assertion-failed! "held-worker: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))) outs0)))))
                    (:wat::core::Tuple q outs)
                    held)
            s' (:fanout::held-worker::State :durable rec
                 :q (:wat::core::first pair)
                 :outcomes (:wat::core::second pair)
                 :held (:wat::core::Vector :- [:queue::Envelope]))]
           (:wat::service::SelfOutcome::Continue s'
             (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 500) :op :-tick)]))
         (:wat::core::let
           [now (:wat::time::epoch-nanos (:wat::time::now))
            vis 1000000000000
            rr  (:queue::Queue/receive q
                  (:queue::Queue::ReceiveRequest
                    :queue name :now-ns now :visibility-ns vis :limit 10 :wait (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 50))))]
           (:wat::core::match rr
             ((:wat::kernel::RecvOutcome::Message r)
               (:wat::core::match r
                 ((:queue::Queue::ReceiveResponse::Ok envs)
                   (:wat::core::if (:wat::core::empty? envs)
                     (:wat::service::SelfOutcome::Continue s
                       (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)])
                     (:wat::core::let
                       [s' (:fanout::held-worker::State :durable rec :q q :outcomes outs :held envs)]
                       (:wat::service::SelfOutcome::Continue s'
                         (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 500) :op :-tick)]))))
                 (_ (:wat::kernel::assertion-failed! "held-worker: receive not Ok" :wat::core::None :wat::core::None))))
             ((:wat::kernel::RecvOutcome::Lost _cause)
               (:wat::core::let
                 [fresh (:wat::core::match
                          (:wat::kernel::connect (:fanout::held-worker::Record/queue-addr rec))
                          ((:wat::kernel::ConnectOutcome::Connected p) p)
                          (_ (:wat::kernel::assertion-failed! "held-worker: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                  s' (:fanout::held-worker::State :durable rec :q fresh :outcomes outs :held held)]
                 (:wat::service::SelfOutcome::Continue s'
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)])))
             (:wat::kernel::RecvOutcome::Stopped
               (:wat::kernel::assertion-failed! "held-worker: receive stopped" :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Closed
               (:wat::core::let
                 [fresh (:wat::core::match
                          (:wat::kernel::connect (:fanout::held-worker::Record/queue-addr rec))
                          ((:wat::kernel::ConnectOutcome::Connected p) p)
                          (_ (:wat::kernel::assertion-failed! "held-worker: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                  s' (:fanout::held-worker::State :durable rec :q fresh :outcomes outs :held held)]
                 (:wat::service::SelfOutcome::Continue s'
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)]))) (:wat::kernel::RecvOutcome::TimedOut (:wat::core::let [fresh (:wat::core::match (:wat::kernel::connect (:fanout::held-worker::Record/queue-addr rec)) ((:wat::kernel::ConnectOutcome::Connected p) p) (_ (:wat::kernel::assertion-failed! "held-worker: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))) s' (:fanout::held-worker::State :durable rec :q fresh :outcomes outs :held held)] (:wat::service::SelfOutcome::Continue s' (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)]))))))))])

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

(:wat::core::defn :fanout::dial-seen
  [a <- (:wat::kernel::Address :- [:fanout::Seen::Op :fanout::Seen::Reply])]
  -> :fanout::Seen
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::pids [pl <- :wat::spawn::ProcessLaunch]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

;; Start arms the tick and replies Ok. Lost is not death (a peer is dead only
;; when redial fails): the arm runs to completion, so a lost reply still means
;; the tick is armed. Proceed; a truly dead worker shows as unread (-1) on the
;; next drain. Stopped is the PARENT shutting down — certain, local, and
;; continuing would arm workers we are tearing down. Closed is a clean EOF
;; without a message; treated like Lost (proceed). Do not flip Closed to
;; assert — that is a behaviour change, not a rename.
(:wat::core::defn :fanout::start-worker!
  [w <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
  -> :wat::core::nil
  (:wat::core::match (:fanout::Worker/start w (:fanout::Worker::StartRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:fanout::Worker::StartResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "fanout: start not Ok" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost _cause) nil)
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "fanout: start stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil)))

(:wat::core::defn :fanout::arm-workers!
  [wpeers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])])]
  -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil
                     w   <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
      -> :wat::core::nil
      (:fanout::start-worker! w))
    nil
    wpeers))

;; Timer-channel recv, not a sleep — legal where mora forbids sleeping.
(:wat::core::defn :fanout::await-timer-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil)))

;; Parent-side constructor. Chaos fields default off (rate 0 arms nothing).
(:wat::core::defn :fanout::mk-worker
  [id         <- :wat::core::String
   queue-name <- :wat::core::String
   vis-ns     <- :wat::core::i64
   ack-delay-ms  <- :wat::core::i64
   work-delay-ms <- :wat::core::i64
   queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])
   seen-addr  <- (:wat::kernel::Address :- [:fanout::Seen::Op :fanout::Seen::Reply])
   rate-bp    <- :wat::core::i64
   seed       <- :wat::core::i64]
  -> :fanout::worker::Record
  (:fanout::worker::Record
    :id id :queue-name queue-name :vis-ns vis-ns
    :ack-delay-ms ack-delay-ms :work-delay-ms work-delay-ms
    :queue-addr queue-addr :seen-addr seen-addr
    :disrupt-rate-bp rate-bp :disrupt-seed seed
    :disrupt-lo-ms 50 :disrupt-hi-ms 150 :disrupt-max-draws 0
    :disrupt-hits 0 :disrupt-draws 0 :disrupt-points "" :check-exhausted 0 :mark-exhausted 0 :ack-retries 0 :ack-exhausted 0))

;; Sentinel: -1 means unread. Matches ticks-of / q-depth. (1,1) satisfied both waits.
(:wat::core::defn :fanout::depth-of
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible unacked _ _ _)
          (:wat::core::Tuple visible unacked))
        (_ (:wat::core::Tuple -1 -1))))
    (_ (:wat::core::Tuple -1 -1))))

(:wat::core::defn :fanout::topic-outbox [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok n _ticks _l _c _t) n)
        (_ -1)))
    (_ -1)))

;; TEMPORARY INSTRUMENT — how many -deliver ticks did the topic take for N messages?
;; One tick per message means a timer arm + fire + select wake is paid per message.
(:wat::core::defn :fanout::topic-ticks [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok _n ticks _l _c _t) ticks)
        (_ -1)))
    (_ -1)))

(:wat::core::defn :fanout::topic-inbox-fails
  [t <- :demo::Topic]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok _n _ticks lost closed timedout)
          (:wat::core::Tuple lost closed timedout))
        (_ (:wat::core::Tuple -1 -1 -1))))
    (_ (:wat::core::Tuple -1 -1 -1))))

(:wat::core::defn :fanout::require!
  [r <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:wat::core::= r "")
    nil
    (:wat::kernel::assertion-failed! r :wat::core::None :wat::core::None)))

(:wat::core::defn :fanout::elapsed-ms [start-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns) 1000000))

(:wat::core::defn :fanout::sweep-of
  [qclients <- (:wat::core::Vector :- [:queue::Queue])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
                     q   <- :queue::Queue]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
      (:wat::core::conj acc (:fanout::depth-of q)))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
    qclients))

(:wat::core::defn :fanout::snapshot-str
  [sweep <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
  -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String
                     d   <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])]
      -> :wat::core::String
      (:wat::core::format "{acc}[{v}/{u}]"
        :acc acc :v (:wat::core::first d) :u (:wat::core::second d)))
    ""
    sweep))

(:wat::core::defn :fanout::sweep-unread?
  [sweep <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool
                     d   <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])]
      -> :wat::core::bool
      (:wat::core::or acc (:wat::core::= (:wat::core::first d) -1)))
    false
    sweep))

(:wat::core::defn :fanout::sweep-drained?
  [sweep <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [ok <- :wat::core::bool
                     d  <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])]
      -> :wat::core::bool
      (:wat::core::and ok
        (:wat::core::and (:wat::core::= (:wat::core::first d) 0)
          (:wat::core::= (:wat::core::second d) 0))))
    true
    sweep))

(:wat::core::defn :fanout::sweep-filled?
  [sweep <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
   n     <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [ok <- :wat::core::bool
                     d  <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])]
      -> :wat::core::bool
      (:wat::core::and ok
        (:wat::core::and (:wat::core::= (:wat::core::first d) n)
          (:wat::core::= (:wat::core::second d) 0))))
    true
    sweep))

;; Publishers returning is not the fill: topic-workers may still be
;; fanning the last inbox rows. Poll until every subscriber queue holds
;; n visible, 0 unacked. Attempts = n×m, same hang bound as drain.
(:wat::core::defn :fanout::poll-until-filled*
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic
   n <- :wat::core::i64  left <- :wat::core::i64
   start-ns <- :wat::core::i64  total <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [sweep (:fanout::sweep-of qclients)
     box   (:fanout::topic-outbox t)]
    (:wat::core::if (:wat::core::or (:wat::core::= box -1) (:fanout::sweep-unread? sweep))
      (:wat::core::format "filled-unread: last={s} outbox={b} attempts={a} elapsed={ms}"
        :s (:fanout::snapshot-str sweep) :b box
        :a (:wat::i64::- total left) :ms (:fanout::elapsed-ms start-ns))
      (:wat::core::if (:wat::core::and (:fanout::sweep-filled? sweep n) (:wat::core::= box 0))
        ""
        (:wat::core::if (:wat::i64::<= left 1)
          (:wat::core::format "filled-never: last={s} outbox={b} want={n} attempts={a} elapsed={ms}"
            :s (:fanout::snapshot-str sweep) :b box :n n
            :a total :ms (:fanout::elapsed-ms start-ns))
          (:wat::core::let [_ (:fanout::await-timer-ms 5)]
            (:fanout::poll-until-filled* qclients t n (:wat::i64::- left 1) start-ns total)))))))

(:wat::core::defn :fanout::poll-until-filled
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic
   n <- :wat::core::i64  attempts <- :wat::core::i64]
  -> :wat::core::String
  (:fanout::poll-until-filled* qclients t n attempts
    (:wat::time::epoch-nanos (:wat::time::now)) attempts))

;; Conjunction across N queues plus the topic inbox. No single wire event.
;; Bounded, and it reports what it last saw — the check rung, taken only
;; where the shape rung is unavailable. One sweep per iteration; the four
;; facts are derived from it. rts counts Queue/stats + Topic/stats calls.
(:wat::core::defn :fanout::poll-until-drained*
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic
   left <- :wat::core::i64  start-ns <- :wat::core::i64  total <- :wat::core::i64
   rts <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])
  (:wat::core::let
    [sweep (:fanout::sweep-of qclients)
     box   (:fanout::topic-outbox t)
     rts'  (:wat::i64::+ rts (:wat::i64::+ (:wat::core::count qclients) 1))]
    (:wat::core::if (:wat::core::or (:wat::core::= box -1) (:fanout::sweep-unread? sweep))
      (:wat::core::Tuple
        (:wat::core::format "drained-unread: last={s} outbox={b} attempts={a} elapsed={ms}"
          :s (:fanout::snapshot-str sweep) :b box
          :a (:wat::i64::- total left) :ms (:fanout::elapsed-ms start-ns))
        rts')
      (:wat::core::if (:wat::core::and (:fanout::sweep-drained? sweep) (:wat::core::= box 0))
        (:wat::core::Tuple "" rts')
        (:wat::core::if (:wat::i64::<= left 1)
          (:wat::core::Tuple
            (:wat::core::format "drained-never: last={s} outbox={b} attempts={a} elapsed={ms}"
              :s (:fanout::snapshot-str sweep) :b box
              :a total :ms (:fanout::elapsed-ms start-ns))
            rts')
          (:wat::core::let [_ (:fanout::await-timer-ms 5)]
            (:fanout::poll-until-drained* qclients t (:wat::i64::- left 1) start-ns total rts')))))))

(:wat::core::defn :fanout::poll-until-drained
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic  attempts <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])
  (:fanout::poll-until-drained* qclients t attempts
    (:wat::time::epoch-nanos (:wat::time::now)) attempts 0))

;; LIVENESS BOUND — only a hang may trip this. Full is correct backpressure
;; (the queue is bounded; a waiting producer is the design). Giving up loses
;; the message. 60000 ms is the floor from
;; BRIEF-278-a-liveness-bound-only-catches-a-hang: a red here is STUCK, never
;; "the box was busy". Force-expire via publish-until-accepted!* with
;; limit-ms 0 against a full inbox.
(:wat::core::defn :fanout::drop-first
  [v <- (:wat::core::Vector :- [:wat::core::String])  n <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [len (:wat::core::count v)
     n0  (:wat::core::if (:wat::i64::< n 0) 0 n)
     rest (:wat::core::if (:wat::i64::>= n0 len) 0 (:wat::i64::- len n0))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::conj acc (:wat::core::nth v (:wat::i64::+ n0 i))))
      (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::range 0 rest))))

;; Bounds, not an operating point. BASE is the smallest legal wait; CAP sits
;; past the sweep's turn-up at 50 ms. The client discovers where to sit.
(:wat::core::def :fanout::BACKOFF-BASE-MS 1)
(:wat::core::def :fanout::BACKOFF-CAP-MS 100)
(:wat::core::def :fanout::BACKOFF-SEED 1)

(:wat::core::defn :fanout::pow2 [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= n 0)
    1
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
        (:wat::i64::* acc 2))
      1
      (:wat::core::range 0 n))))

;; ceiling = min(CAP, BASE << attempt); draw uniform in [1, ceiling].
;; int-from is [lo, hi), so the call is 1 .. ceiling+1. ceiling=1 → [1, 2) = {1}.
(:wat::core::defn :fanout::backoff-delay
  [seed <- :wat::core::i64  attempt <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [shifted (:wat::core::if (:wat::i64::>= attempt 7)
               :fanout::BACKOFF-CAP-MS
               (:wat::i64::* :fanout::BACKOFF-BASE-MS (:fanout::pow2 attempt)))
     ceiling (:wat::core::if (:wat::i64::> shifted :fanout::BACKOFF-CAP-MS)
               :fanout::BACKOFF-CAP-MS shifted)]
    (:wat::rand::int-from seed 1 (:wat::i64::+ ceiling 1))))

(:wat::core::defn :fanout::publish-until-accepted!*
  [t <- :demo::Topic  msgs <- (:wat::core::Vector :- [:wat::core::String])
   attempt <- :wat::core::i64  retries <- :wat::core::i64  seed <- :wat::core::i64
   start-ns <- :wat::core::i64  limit-ms <- :wat::core::i64  asleep <- :wat::core::i64
   attempts <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
  (:wat::core::match (:demo::Topic/publish t (:demo::Topic::PublishRequest :msgs msgs))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::PublishResponse::Accepted c)
          (:wat::core::let [n (:wat::core::count msgs)
                            att (:wat::i64::+ attempts 1)]
            (:wat::core::if (:wat::i64::>= c n)
              (:wat::core::Tuple retries asleep att)
              (:wat::core::if (:wat::i64::<= c 0)
                (:wat::core::let [elapsed (:fanout::elapsed-ms start-ns)]
                  (:wat::core::if (:wat::i64::>= elapsed limit-ms)
                    (:wat::kernel::assertion-failed!
                      (:wat::core::format "verdict=never-accepted;attempts={a};elapsed={ms}"
                        :a retries :ms elapsed)
                      :wat::core::None :wat::core::None)
                    (:wat::core::let
                      [drawn (:fanout::backoff-delay seed attempt)
                       seed1 (:wat::core::first drawn)
                       d     (:wat::core::second drawn)
                       _     (:fanout::await-timer-ms d)]
                      (:fanout::publish-until-accepted!* t msgs
                        (:wat::i64::+ attempt 1)
                        (:wat::i64::+ retries 1)
                        seed1 start-ns limit-ms
                        (:wat::i64::+ asleep d)
                        att))))
                (:fanout::publish-until-accepted!* t (:fanout::drop-first msgs c)
                  0 retries seed start-ns limit-ms asleep att)))))
        (_ (:wat::kernel::assertion-failed! "fanout: publish not Accepted" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "fanout: publish stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "fanout: publish closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::publish-until-accepted-from!
  [t <- :demo::Topic  msgs <- (:wat::core::Vector :- [:wat::core::String])
   seed <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
  (:fanout::publish-until-accepted!* t msgs 0 0 seed
    (:wat::time::epoch-nanos (:wat::time::now)) 60000 0 0))

(:wat::core::defn :fanout::publish-until-accepted!
  [t <- :demo::Topic  msgs <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
  (:fanout::publish-until-accepted-from! t msgs :fanout::BACKOFF-SEED))

;; Each message keeps its OWN t0. A shared origin would collapse e2e.
(:wat::core::defn :fanout::stamped-range
  [start <- :wat::core::i64  ntake <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  k <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc
        (:wat::core::format "{m}|{t0}"
          :m (:wat::core::str (:wat::i64::+ start k))
          :t0 (:wat::time::epoch-nanos (:wat::time::now)))))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 ntake)))

;; Chunk [lo, hi) into batches of at most 10. Last batch may be short.
;; Returns ((calls retries) (asleep-ms attempts)). No fourth Tuple accessor.
(:wat::core::defn :fanout::publish-share-until-accepted!
  [t <- :demo::Topic  lo <- :wat::core::i64  hi <- :wat::core::i64  seed <- :wat::core::i64]
  -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                           (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  (:wat::core::if (:wat::i64::>= lo hi)
    (:wat::core::Tuple (:wat::core::Tuple 0 0) (:wat::core::Tuple 0 0))
    (:wat::core::let
      [n (:wat::i64::- hi lo)
       nbatches (:wat::i64::/ (:wat::i64::+ n 9) 10)]
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                                                      (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
                         b   <- :wat::core::i64]
          -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                                    (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
          (:wat::core::let
            [start (:wat::i64::+ lo (:wat::i64::* b 10))
             ntake (:wat::core::if (:wat::i64::>= (:wat::i64::+ start 10) hi)
                      (:wat::i64::- hi start)
                      10)
             pair (:fanout::publish-until-accepted-from! t (:fanout::stamped-range start ntake) seed)]
            (:wat::core::Tuple
              (:wat::core::Tuple
                (:wat::i64::+ (:wat::core::first (:wat::core::first acc)) 1)
                (:wat::i64::+ (:wat::core::second (:wat::core::first acc)) (:wat::core::first pair)))
              (:wat::core::Tuple
                (:wat::i64::+ (:wat::core::first (:wat::core::second acc)) (:wat::core::second pair))
                (:wat::i64::+ (:wat::core::second (:wat::core::second acc)) (:wat::core::third pair))))))
        (:wat::core::Tuple (:wat::core::Tuple 0 0) (:wat::core::Tuple 0 0))
        (:wat::core::range 0 nbatches)))))

(:wat::core::defn :fanout::publish-n-until-accepted!
  [t <- :demo::Topic  n <- :wat::core::i64]
  -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                           (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  (:fanout::publish-share-until-accepted! t 0 n :fanout::BACKOFF-SEED))

(:wat::core::defn :fanout::share-lo
  [i <- :wat::core::i64  n <- :wat::core::i64  p <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::* i n) p))

(:wat::core::defn :fanout::share-hi
  [i <- :wat::core::i64  n <- :wat::core::i64  p <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::* (:wat::i64::+ i 1) n) p))

(:wat::core::defn :fanout::publisher-seed [i <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ :fanout::BACKOFF-SEED (:wat::i64::* i 7919)))

;; Named fields so a defservice impl can read them. first/second/third on a
;; Tuple inside :impls typechecks as an unsolved var; a process child also
;; cannot see a defn that is written after the defservice.
(:wat::core::defrecord :fanout::ShareStats
  [calls     <- :wat::core::i64
   retries   <- :wat::core::i64
   asleep    <- :wat::core::i64
   attempts  <- :wat::core::i64])

(:wat::core::defn :fanout::share-stats!
  [t <- :demo::Topic  lo <- :wat::core::i64  hi <- :wat::core::i64  seed <- :wat::core::i64]
  -> :fanout::ShareStats
  (:wat::core::let
    [pair (:fanout::publish-share-until-accepted! t lo hi seed)]
    (:fanout::ShareStats
      :calls     (:wat::core::first (:wat::core::first pair))
      :retries   (:wat::core::second (:wat::core::first pair))
      :asleep    (:wat::core::first (:wat::core::second pair))
      :attempts  (:wat::core::second (:wat::core::second pair)))))

;; ── publisher: one client, one share of the id range, its own seed ──────────
(:wat::core::defsurface :fanout::Publisher :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :fanout::Publisher::StartRequest [])
   (:wat::core::defenum :fanout::Publisher::StartResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :fanout::Publisher::StatsRequest [])
   (:wat::core::defenum :fanout::Publisher::StatsResponse :wat::enum::Pure
     :Ok [done <- :wat::core::bool  calls <- :wat::core::i64
          retries <- :wat::core::i64  asleep <- :wat::core::i64
          attempts <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start [self <- :fanout::Publisher  req <- :fanout::Publisher::StartRequest]
     -> :fanout::Publisher::StartResponse :max-request-bytes 524288)
   (stats [self <- :fanout::Publisher  req <- :fanout::Publisher::StatsRequest]
     -> :fanout::Publisher::StatsResponse :max-request-bytes 524288)])

(:wat::service::defservice :fanout::publisher
  :satisfies :fanout::Publisher
  ;; stats is the join: it must outlive one publisher's share (~20 s at N=2000).
  ;; Default 10000 would TimedOut mid -run and the parent would see a hang-shaped failure.
  :deadline-ms 120000
  :durable   [id         <- :wat::core::String
              topic-addr <- (:wat::kernel::Address :- [:demo::Topic::Op :demo::Topic::Reply])
              lo         <- :wat::core::i64
              hi         <- :wat::core::i64
              seed       <- :wat::core::i64
              done       <- :wat::core::bool
              calls      <- :wat::core::i64
              retries    <- :wat::core::i64
              asleep     <- :wat::core::i64
              attempt    <- :wat::core::i64
              attempts   <- :wat::core::i64]
  :ephemeral [topic      <- (:wat::kernel::Peer :- [:demo::Topic::Op :demo::Topic::Reply])
              remaining  <- (:wat::core::Vector :- [:wat::core::String])]
  :peers     [:demo::Topic]
  :init (:wat::core::fn
          [record <- :fanout::publisher::Record]
          -> :fanout::publisher::State
          (:fanout::publisher::State :durable record
            :topic
              (:wat::core::match (:wat::kernel::connect (:fanout::publisher::Record/topic-addr record))
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                ((:wat::kernel::ConnectOutcome::Refused c)
                  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Rejected c)
                  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Failed c)
                  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
            :remaining (:wat::core::Vector :- [:wat::core::String])))
  :impls
  [(start [s ctx req]
     (:wat::core::let
       [none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Publisher::Reply])])
        run (:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-run)]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:fanout::Publisher::Reply::Start (:fanout::Publisher::StartResponse::Ok)))
         none-sends
         [run])))
   (stats [s ctx req]
     (:wat::core::let
       [rec (:fanout::publisher::State/durable s)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Publisher::Reply])])
        none-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::publisher::Op])])]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:fanout::Publisher::Reply::Stats
           (:fanout::Publisher::StatsResponse::Ok
             (:fanout::publisher::Record/done rec)
             (:fanout::publisher::Record/calls rec)
             (:fanout::publisher::Record/retries rec)
             (:fanout::publisher::Record/asleep rec)
             (:fanout::publisher::Record/attempts rec))))
         none-sends none-arms)))
   (-run [s ctx]
     (:wat::core::let
       [none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Publisher::Reply])])
        none-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::publisher::Op])])
        rec (:fanout::publisher::State/durable s)
        t   (:fanout::publisher::State/topic s)
        lo  (:fanout::publisher::Record/lo rec)
        hi  (:fanout::publisher::Record/hi rec)
        seed0 (:fanout::publisher::Record/seed rec)
        stamp
          (:wat::core::fn [start <- :wat::core::i64  ntake <- :wat::core::i64]
            -> (:wat::core::Vector :- [:wat::core::String])
            (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  k <- :wat::core::i64]
                -> (:wat::core::Vector :- [:wat::core::String])
                (:wat::core::conj acc
                  (:wat::core::format "{m}|{t0}"
                    :m (:wat::core::str (:wat::i64::+ start k))
                    :t0 (:wat::time::epoch-nanos (:wat::time::now)))))
              (:wat::core::Vector :- [:wat::core::String])
              (:wat::core::range 0 ntake)))
        drop-first
          (:wat::core::fn [v <- (:wat::core::Vector :- [:wat::core::String])  n <- :wat::core::i64]
            -> (:wat::core::Vector :- [:wat::core::String])
            (:wat::core::let
              [len (:wat::core::count v)
               n0  (:wat::core::if (:wat::i64::< n 0) 0 n)
               rest (:wat::core::if (:wat::i64::>= n0 len) 0 (:wat::i64::- len n0))]
              (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
                  -> (:wat::core::Vector :- [:wat::core::String])
                  (:wat::core::conj acc (:wat::core::nth v (:wat::i64::+ n0 i))))
                (:wat::core::Vector :- [:wat::core::String])
                (:wat::core::range 0 rest))))
        await-ms
          (:wat::core::fn [ms <- :wat::core::i64] -> :wat::core::nil
            (:wat::core::match
              (:wat::kernel::recv
                (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
              ((:wat::kernel::RecvOutcome::Message _m) nil)
              ((:wat::kernel::RecvOutcome::Lost _c) nil)
              (:wat::kernel::RecvOutcome::Stopped nil)
              (:wat::kernel::RecvOutcome::Closed nil)
              (:wat::kernel::RecvOutcome::TimedOut nil)))
        n (:wat::core::if (:wat::i64::>= lo hi) 0 (:wat::i64::- hi lo))
        nbatches (:wat::i64::/ (:wat::i64::+ n 9) 10)
        acc0 (:wat::core::Tuple (:wat::core::Tuple 0 0 0) (:wat::core::Tuple 0 seed0))
        acc
          (:wat::core::foldl
            (:wat::core::fn
              [acc <- (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
                                            (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
               b <- :wat::core::i64]
              -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
                                        (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
              (:wat::core::let
                [calls (:wat::core::first (:wat::core::first acc))
                 retries (:wat::core::second (:wat::core::first acc))
                 attempts (:wat::core::third (:wat::core::first acc))
                 asleep (:wat::core::first (:wat::core::second acc))
                 start (:wat::i64::+ lo (:wat::i64::* b 10))
                 ntake (:wat::core::if (:wat::i64::>= (:wat::i64::+ start 10) hi)
                          (:wat::i64::- hi start)
                          10)
                 msgs (stamp start ntake)
                 start-ns (:wat::time::epoch-nanos (:wat::time::now))
                 st0 (:wat::core::Tuple
                        (:wat::core::Tuple msgs 0 0)
                        (:wat::core::Tuple seed0 0 false)
                        0)
                 st
                   (:wat::core::foldl
                     (:wat::core::fn
                       [st <- (:wat::core::Tuple :- [(:wat::core::Tuple :- [(:wat::core::Vector :- [:wat::core::String]) :wat::core::i64 :wat::core::i64])
                                                     (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::bool])
                                                     :wat::core::i64])
                        _i <- :wat::core::i64]
                       -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [(:wat::core::Vector :- [:wat::core::String]) :wat::core::i64 :wat::core::i64])
                                                 (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::bool])
                                                 :wat::core::i64])
                       (:wat::core::let
                         [left (:wat::core::first st)
                          right (:wat::core::second st)
                          att (:wat::core::third st)
                          remaining (:wat::core::first left)
                          rtry (:wat::core::second left)
                          aslp (:wat::core::third left)
                          sd (:wat::core::first right)
                          attempt (:wat::core::second right)
                          done (:wat::core::third right)]
                         (:wat::core::if done
                           st
                           (:wat::core::match (:demo::Topic/publish t (:demo::Topic::PublishRequest :msgs remaining))
                             ((:wat::kernel::RecvOutcome::Message r)
                               (:wat::core::match r
                                 ((:demo::Topic::PublishResponse::Accepted c)
                                   (:wat::core::let [nc (:wat::core::count remaining)
                                                    att1 (:wat::i64::+ att 1)]
                                     (:wat::core::if (:wat::i64::>= c nc)
                                       (:wat::core::Tuple left (:wat::core::Tuple sd attempt true) att1)
                                       (:wat::core::if (:wat::i64::<= c 0)
                                         (:wat::core::let
                                           [elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns) 1000000)]
                                           (:wat::core::if (:wat::i64::>= elapsed 60000)
                                             (:wat::kernel::assertion-failed!
                                               (:wat::core::format "verdict=never-accepted;attempts={a};elapsed={ms}"
                                                 :a rtry :ms elapsed)
                                               :wat::core::None :wat::core::None)
                                             (:wat::core::let
                                               [shifted (:wat::core::if (:wat::i64::>= attempt 7)
                                                          100
                                                          (:wat::core::foldl
                                                            (:wat::core::fn [a <- :wat::core::i64  _j <- :wat::core::i64] -> :wat::core::i64
                                                              (:wat::i64::* a 2))
                                                            1
                                                            (:wat::core::range 0 attempt)))
                                                ceiling (:wat::core::if (:wat::i64::> shifted 100) 100 shifted)
                                                drawn (:wat::rand::int-from sd 1 (:wat::i64::+ ceiling 1))
                                                seed1 (:wat::core::first drawn)
                                                d (:wat::core::second drawn)
                                                _nap (await-ms d)]
                                               (:wat::core::Tuple
                                                 (:wat::core::Tuple remaining (:wat::i64::+ rtry 1) (:wat::i64::+ aslp d))
                                                 (:wat::core::Tuple seed1 (:wat::i64::+ attempt 1) false)
                                                 att1))))
                                         (:wat::core::Tuple
                                           (:wat::core::Tuple (drop-first remaining c) rtry aslp)
                                           (:wat::core::Tuple sd 0 false)
                                           att1)))))
                                 (_ (:wat::kernel::assertion-failed! "fanout: publish not Accepted" :wat::core::None :wat::core::None))))
                             ((:wat::kernel::RecvOutcome::Lost cause)
                               (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                             (:wat::kernel::RecvOutcome::Stopped
                               (:wat::kernel::assertion-failed! "fanout: publish stopped" :wat::core::None :wat::core::None))
                             (:wat::kernel::RecvOutcome::Closed
                               (:wat::kernel::assertion-failed! "fanout: publish closed" :wat::core::None :wat::core::None))
                             (:wat::kernel::RecvOutcome::TimedOut
                               (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))))
                     st0
                     (:wat::core::range 0 256))
                 left (:wat::core::first st)
                 right (:wat::core::second st)
                 _ok (:wat::core::if (:wat::core::third right)
                       nil
                       (:wat::kernel::assertion-failed! "fanout: publisher batch never accepted" :wat::core::None :wat::core::None))]
                (:wat::core::Tuple
                  (:wat::core::Tuple (:wat::i64::+ calls 1) (:wat::i64::+ retries (:wat::core::second left))
                    (:wat::i64::+ attempts (:wat::core::third st)))
                  (:wat::core::Tuple (:wat::i64::+ asleep (:wat::core::third left)) (:wat::core::first right)))))
            acc0
            (:wat::core::range 0 nbatches))
        rec' (:fanout::publisher::Record
               :id (:fanout::publisher::Record/id rec)
               :topic-addr (:fanout::publisher::Record/topic-addr rec)
               :lo lo :hi hi
               :seed seed0
               :done true
               :calls (:wat::core::first (:wat::core::first acc))
               :retries (:wat::core::second (:wat::core::first acc))
               :asleep (:wat::core::first (:wat::core::second acc))
               :attempt 0
               :attempts (:wat::core::third (:wat::core::first acc)))
        s' (:fanout::publisher::State :durable rec' :topic t
             :remaining (:wat::core::Vector :- [:wat::core::String]))]
       (:wat::service::SelfOutcome::Continue s' none-sends none-arms)))])

(:wat::core::defn :fanout::dial-publisher
  [a <- (:wat::kernel::Address :- [:fanout::Publisher::Op :fanout::Publisher::Reply])]
  -> (:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::start-publisher!
  [w <- (:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])]
  -> :wat::core::nil
  (:wat::core::match (:fanout::Publisher/start w (:fanout::Publisher::StartRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:fanout::Publisher::StartResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "fanout: publisher start not Ok" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost _cause) nil)
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "fanout: publisher start stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil)))

(:wat::core::defn :fanout::publisher-stats
  [w <- (:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])]
  -> :fanout::Publisher::StatsResponse
  (:wat::core::let
    [inert (:fanout::Publisher::Reply::Stats
             (:fanout::Publisher::StatsResponse::Ok false 0 0 0 0))]
    (:wat::core::match
      (:wat::service::call-by-deadline w
        (:fanout::Publisher::Op::Stats (:fanout::Publisher::StatsRequest))
        120000 inert)
      ((:wat::service::CallOutcome::Answered m)
        (:wat::core::match m
          ((:fanout::Publisher::Reply::Stats r) r)
          (_ (:wat::kernel::assertion-failed! "fanout: publisher stats misrouted" :wat::core::None :wat::core::None))))
      ((:wat::service::CallOutcome::DeadlineFired)
        (:wat::kernel::assertion-failed! "fanout: publisher stats deadline" :wat::core::None :wat::core::None))
      ((:wat::service::CallOutcome::Lost _c)
        (:wat::kernel::assertion-failed! "fanout: publisher stats lost" :wat::core::None :wat::core::None))
      ((:wat::service::CallOutcome::Closed)
        (:wat::kernel::assertion-failed! "fanout: publisher stats closed" :wat::core::None :wat::core::None)))))

(:wat::core::defn :fanout::publishers-all-done?
  [peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])])]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [ok <- :wat::core::bool
                     w  <- (:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])]
      -> :wat::core::bool
      (:wat::core::match (:fanout::publisher-stats w)
        ((:fanout::Publisher::StatsResponse::Ok d _c _r _s _a) (:wat::core::and ok d))
        (_ false)))
    true
    peers))

(:wat::core::defn :fanout::sum-publisher-stats
  [peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])])]
  -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                           (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  (:wat::core::foldl
    (:wat::core::fn [a <- (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                                                (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
                     w <- (:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])]
      -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                                (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
      (:wat::core::match (:fanout::publisher-stats w)
        ((:fanout::Publisher::StatsResponse::Ok _d c r s att)
          (:wat::core::Tuple
            (:wat::core::Tuple
              (:wat::i64::+ (:wat::core::first (:wat::core::first a)) c)
              (:wat::i64::+ (:wat::core::second (:wat::core::first a)) r))
            (:wat::core::Tuple
              (:wat::i64::+ (:wat::core::first (:wat::core::second a)) s)
              (:wat::i64::+ (:wat::core::second (:wat::core::second a)) att))))
        (_ a)))
    (:wat::core::Tuple (:wat::core::Tuple 0 0) (:wat::core::Tuple 0 0))
    peers))

;; Poll until every publisher reports done. Returns ((calls retries) (asleep attempts)).
(:wat::core::defn :fanout::join-publishers*
  [peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])])
   left <- :wat::core::i64]
  -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                           (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  (:wat::core::if (:wat::i64::<= left 0)
    (:wat::kernel::assertion-failed! "fanout: publishers never done" :wat::core::None :wat::core::None)
    (:wat::core::if (:fanout::publishers-all-done? peers)
      (:fanout::sum-publisher-stats peers)
      (:wat::core::let [_ (:fanout::await-timer-ms 1)]
        (:fanout::join-publishers* peers (:wat::i64::- left 1))))))

(:wat::core::defn :fanout::join-publishers
  [peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])])]
  -> (:wat::core::Tuple :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                           (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  (:fanout::join-publishers* peers 120000))

(:wat::core::defn :fanout::poll-until-visible-zero*
  [q <- :queue::Queue  left <- :wat::core::i64  start-ns <- :wat::core::i64  total <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [d (:fanout::depth-of q)
     v (:wat::core::first d)
     u (:wat::core::second d)]
    (:wat::core::if (:wat::core::= v -1)
      (:wat::core::format "visible-unread: last={v}/{u} attempts={a} elapsed={ms}"
        :v v :u u :a (:wat::i64::- total left) :ms (:fanout::elapsed-ms start-ns))
      (:wat::core::if (:wat::core::= v 0)
        ""
        (:wat::core::if (:wat::i64::<= left 1)
          (:wat::core::format "visible-never-zero: last={v}/{u} attempts={a} elapsed={ms}"
            :v v :u u :a total :ms (:fanout::elapsed-ms start-ns))
          (:wat::core::let [_ (:fanout::await-timer-ms 5)]
            (:fanout::poll-until-visible-zero* q (:wat::i64::- left 1) start-ns total)))))))

(:wat::core::defn :fanout::poll-until-visible-zero
  [q <- :queue::Queue  attempts <- :wat::core::i64] -> :wat::core::String
  (:fanout::poll-until-visible-zero* q attempts (:wat::time::epoch-nanos (:wat::time::now)) attempts))

(:wat::core::defn :fanout::sum-calls
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  q <- :queue::Queue] -> :wat::core::i64
      (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:queue::Queue::StatsResponse::Ok calls _ticks _visible _unacked _ _ _)
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
            ((:queue::Queue::StatsResponse::Ok _calls ticks _visible _unacked _ _ _)
              (:wat::i64::+ acc ticks))
            (_ acc)))
        (_ acc)))
    0
    qclients))

(:wat::core::defn :fanout::sum-store-calls
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  q <- :queue::Queue] -> :wat::core::i64
      (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:queue::Queue::StatsResponse::Ok _calls _ticks _visible _unacked sc _ _)
              (:wat::i64::+ acc sc))
            (_ acc)))
        (_ acc)))
    0
    qclients))

(:wat::core::defn :fanout::sum-store-ns
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  q <- :queue::Queue] -> :wat::core::i64
      (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:queue::Queue::StatsResponse::Ok _calls _ticks _visible _unacked _ sns _)
              (:wat::i64::+ acc sns))
            (_ acc)))
        (_ acc)))
    0
    qclients))

(:wat::core::defn :fanout::sum-handler-ns
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  q <- :queue::Queue] -> :wat::core::i64
      (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:queue::Queue::StatsResponse::Ok _calls _ticks _visible _unacked _ _ hn)
              (:wat::i64::+ acc hn))
            (_ acc)))
        (_ acc)))
    0
    qclients))

(:wat::core::defn :fanout::seen-stats
  [seenh <- :fanout::seen::Handle]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [p (:fanout::dial-seen (:fanout::seen::Handle/addr seenh))]
    (:wat::core::match (:fanout::Seen/stats p (:fanout::Seen::StatsRequest))
      ((:wat::kernel::RecvOutcome::Message r)
        (:wat::core::match r
          ((:fanout::Seen::StatsResponse::Ok recorded skipped) (:wat::core::Tuple recorded skipped))
          ((:fanout::Seen::StatsResponse::RequestTooLarge _b _c)
            (:wat::kernel::assertion-failed! "fanout: seen stats too large" :wat::core::None :wat::core::None))
          ((:fanout::Seen::StatsResponse::RequestMalformed _p _e _g)
            (:wat::kernel::assertion-failed! "fanout: seen stats malformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost _c)
        (:wat::kernel::assertion-failed! "fanout: seen stats lost" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "fanout: seen stats stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "fanout: seen stats closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

(:wat::core::defn :fanout::sum-disrupts
  [wpeers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])])]
  -> (:wat::core::Tuple :- [:wat::core::i64 (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Tuple :- [:wat::core::i64 (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
                     w   <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
      -> (:wat::core::Tuple :- [:wat::core::i64 (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
      (:wat::core::match (:fanout::Worker/disrupts w (:fanout::Worker::DisruptsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:fanout::Worker::DisruptsResponse::Ok hits _draws _points ce me ars ae)
              (:wat::core::Tuple (:wat::i64::+ (:wat::core::first acc) hits)
                                 (:wat::core::Tuple
                                   (:wat::i64::+ (:wat::core::first (:wat::core::second acc)) ce)
                                   (:wat::i64::+ (:wat::core::second (:wat::core::second acc)) me))
                                 (:wat::core::Tuple
                                   (:wat::i64::+ (:wat::core::first (:wat::core::third acc)) ars)
                                   (:wat::i64::+ (:wat::core::second (:wat::core::third acc)) ae))))
            ((:fanout::Worker::DisruptsResponse::RequestTooLarge _b _c) acc)
            ((:fanout::Worker::DisruptsResponse::RequestMalformed _p _e _g) acc)))
        ((:wat::kernel::RecvOutcome::Lost _c) acc)
        (:wat::kernel::RecvOutcome::Stopped acc)
        (:wat::kernel::RecvOutcome::Closed acc) (:wat::kernel::RecvOutcome::TimedOut acc)))
    (:wat::core::Tuple 0 (:wat::core::Tuple 0 0) (:wat::core::Tuple 0 0))
    wpeers))

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

;; seq is the published identity — first field of the body, placed first so it
;; survives every hop. body-key keyed on the whole body (timestamps included) and
;; was never called: a redelivery would still look distinct. This is that instrument,
;; wired to the stable prefix.
(:wat::core::defn :fanout::seq-of [body <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [parts (:wat::string::split body "|")]
    (:wat::core::if (:wat::core::empty? parts) "" (:wat::core::first parts))))

(:wat::core::defn :fanout::key-of [o <- :fanout::Outcome] -> :wat::core::String
  (:wat::string::concat (:fanout::Outcome/queue o)
    (:wat::string::concat "/" (:fanout::seq-of (:fanout::Outcome/body o)))))

(:wat::core::defn :fanout::body-key [o <- :fanout::Outcome] -> :wat::core::String
  (:fanout::key-of o))

(:wat::core::defrecord :fanout::Hist
  [c0 <- :wat::core::i64
   c1 <- :wat::core::i64
   c2 <- :wat::core::i64
   c3 <- :wat::core::i64
   c4 <- :wat::core::i64
   c5 <- :wat::core::i64
   mx <- :wat::core::i64])

(:wat::core::defrecord :fanout::Traces
  [pub-work    <- :fanout::Hist
   inbox-wait  <- :fanout::Hist
   worker-proc <- :fanout::Hist
   fanout-work <- :fanout::Hist
   subq-wait   <- :fanout::Hist
   e2e         <- :fanout::Hist
   sample      <- :wat::core::String])

(:wat::core::defn :fanout::parse-i64 [s <- :wat::core::String] -> :wat::core::i64
  (:wat::edn::read s))

(:wat::core::defn :fanout::empty-hist [] -> :fanout::Hist
  (:fanout::Hist :c0 0 :c1 0 :c2 0 :c3 0 :c4 0 :c5 0 :mx 0))

(:wat::core::defn :fanout::hist-add
  [h <- :fanout::Hist  dt-ms <- :wat::core::i64]
  -> :fanout::Hist
  (:wat::core::let
    [dt (:wat::core::if (:wat::i64::< dt-ms 0) 0 dt-ms)
     mx (:wat::core::if (:wat::i64::> dt (:fanout::Hist/mx h)) dt (:fanout::Hist/mx h))]
    (:wat::core::if (:wat::i64::< dt 1)
      (:fanout::Hist :c0 (:wat::i64::+ (:fanout::Hist/c0 h) 1) :c1 (:fanout::Hist/c1 h) :c2 (:fanout::Hist/c2 h) :c3 (:fanout::Hist/c3 h) :c4 (:fanout::Hist/c4 h) :c5 (:fanout::Hist/c5 h) :mx mx)
      (:wat::core::if (:wat::i64::< dt 10)
        (:fanout::Hist :c0 (:fanout::Hist/c0 h) :c1 (:wat::i64::+ (:fanout::Hist/c1 h) 1) :c2 (:fanout::Hist/c2 h) :c3 (:fanout::Hist/c3 h) :c4 (:fanout::Hist/c4 h) :c5 (:fanout::Hist/c5 h) :mx mx)
        (:wat::core::if (:wat::i64::< dt 50)
          (:fanout::Hist :c0 (:fanout::Hist/c0 h) :c1 (:fanout::Hist/c1 h) :c2 (:wat::i64::+ (:fanout::Hist/c2 h) 1) :c3 (:fanout::Hist/c3 h) :c4 (:fanout::Hist/c4 h) :c5 (:fanout::Hist/c5 h) :mx mx)
          (:wat::core::if (:wat::i64::< dt 250)
            (:fanout::Hist :c0 (:fanout::Hist/c0 h) :c1 (:fanout::Hist/c1 h) :c2 (:fanout::Hist/c2 h) :c3 (:wat::i64::+ (:fanout::Hist/c3 h) 1) :c4 (:fanout::Hist/c4 h) :c5 (:fanout::Hist/c5 h) :mx mx)
            (:wat::core::if (:wat::i64::< dt 1000)
              (:fanout::Hist :c0 (:fanout::Hist/c0 h) :c1 (:fanout::Hist/c1 h) :c2 (:fanout::Hist/c2 h) :c3 (:fanout::Hist/c3 h) :c4 (:wat::i64::+ (:fanout::Hist/c4 h) 1) :c5 (:fanout::Hist/c5 h) :mx mx)
              (:fanout::Hist :c0 (:fanout::Hist/c0 h) :c1 (:fanout::Hist/c1 h) :c2 (:fanout::Hist/c2 h) :c3 (:fanout::Hist/c3 h) :c4 (:fanout::Hist/c4 h) :c5 (:wat::i64::+ (:fanout::Hist/c5 h) 1) :mx mx))))))))

(:wat::core::defn :fanout::hist-line [name <- :wat::core::String  h <- :fanout::Hist] -> :wat::core::String
  (:wat::core::format
    "{name} <1ms={c0} 1-10={c1} 10-50={c2} 50-250={c3} 250-1000={c4} >1000={c5} max={mx}ms"
    :name name
    :c0 (:fanout::Hist/c0 h) :c1 (:fanout::Hist/c1 h) :c2 (:fanout::Hist/c2 h)
    :c3 (:fanout::Hist/c3 h) :c4 (:fanout::Hist/c4 h) :c5 (:fanout::Hist/c5 h)
    :mx (:fanout::Hist/mx h)))

(:wat::core::defn :fanout::ns->ms [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::- b a) 1000000))

(:wat::core::defn :fanout::traces-add
  [tr <- :fanout::Traces  o <- :fanout::Outcome]
  -> :fanout::Traces
  (:wat::core::let
    [parts (:wat::string::split (:fanout::Outcome/body o) "|")]
    (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::count parts) 7))
      tr
      (:wat::core::let
        [t0  (:fanout::parse-i64 (:wat::core::nth parts 1))
         t0b (:fanout::parse-i64 (:wat::core::nth parts 2))
         t1  (:fanout::parse-i64 (:wat::core::nth parts 3))
         t3  (:fanout::parse-i64 (:wat::core::nth parts 4))
         t3b (:fanout::parse-i64 (:wat::core::nth parts 5))
         t4  (:fanout::parse-i64 (:wat::core::nth parts 6))
         sample (:wat::core::if (:wat::core::= (:fanout::Traces/sample tr) "")
                  (:fanout::Outcome/body o)
                  (:fanout::Traces/sample tr))]
        (:fanout::Traces
          :pub-work    (:fanout::hist-add (:fanout::Traces/pub-work tr)    (:fanout::ns->ms t0 t0b))
          :inbox-wait  (:fanout::hist-add (:fanout::Traces/inbox-wait tr)  (:fanout::ns->ms t0b t1))
          :worker-proc (:fanout::hist-add (:fanout::Traces/worker-proc tr) (:fanout::ns->ms t1 t3))
          :fanout-work (:fanout::hist-add (:fanout::Traces/fanout-work tr) (:fanout::ns->ms t3 t3b))
          :subq-wait   (:fanout::hist-add (:fanout::Traces/subq-wait tr)   (:fanout::ns->ms t3b t4))
          :e2e         (:fanout::hist-add (:fanout::Traces/e2e tr)         (:fanout::ns->ms t0 t4))
          :sample      sample)))))

(:wat::core::defn :fanout::traces-of
  [outs <- (:wat::core::Vector :- [:fanout::Outcome])]
  -> :fanout::Traces
  (:wat::core::foldl
    :fanout::traces-add
    (:fanout::Traces
      :pub-work    (:fanout::empty-hist)
      :inbox-wait  (:fanout::empty-hist)
      :worker-proc (:fanout::empty-hist)
      :fanout-work (:fanout::empty-hist)
      :subq-wait   (:fanout::empty-hist)
      :e2e         (:fanout::empty-hist)
      :sample      "")
    outs))

(:wat::core::defn :fanout::traces-report [tr <- :fanout::Traces] -> :wat::core::String
  (:wat::core::format
    "sample={s} ;; {pw} ;; {iw} ;; {wp} ;; {fw} ;; {sq} ;; {e}"
    :s (:fanout::Traces/sample tr)
    :pw (:fanout::hist-line "pub-work" (:fanout::Traces/pub-work tr))
    :iw (:fanout::hist-line "inbox   " (:fanout::Traces/inbox-wait tr))
    :wp (:fanout::hist-line "worker  " (:fanout::Traces/worker-proc tr))
    :fw (:fanout::hist-line "fanout  " (:fanout::Traces/fanout-work tr))
    :sq (:fanout::hist-line "subq    " (:fanout::Traces/subq-wait tr))
    :e  (:fanout::hist-line "e2e     " (:fanout::Traces/e2e tr))))

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
;; rate 0 (the default) arms no -disrupt alarm at all.
(:wat::core::defn :fanout::run-with
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64  p <- :wat::core::i64
   rate <- :wat::core::i64  seed <- :wat::core::i64
   drop-check-bp <- :wat::core::i64  drop-mark-bp <- :wat::core::i64
   drop-seed <- :wat::core::i64  drop-after? <- :wat::core::bool
   drop-recv-bp <- :wat::core::i64  drop-ack-bp <- :wat::core::i64
   sub-cap <- :wat::core::i64  fill-first? <- :wat::core::bool
   vis-ms <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:wat::core::let
    [t-setup0 (:wat::time::epoch-nanos (:wat::time::now))
     ;; Drop runs: 200 ms vis so an unacked envelope (no claim-reply) becomes
     ;; visible again. T1's 200 ms claim deadline retries the same worker;
     ;; vis expiry is the other worker. Both are retries of a dropped reply.
     ;; vis-ms > 0 is the sweep knob (milliseconds → nanoseconds). 0 means
     ;; exactly today's behaviour: 200 ms if any drop rate is set, else 1000 s.
     vis (:wat::core::if (:wat::i64::> vis-ms 0)
            (:wat::i64::* vis-ms 1000000)
            (:wat::core::if (:wat::core::or
                               (:wat::core::or (:wat::i64::> drop-check-bp 0) (:wat::i64::> drop-mark-bp 0))
                               (:wat::core::or (:wat::i64::> drop-recv-bp 0) (:wat::i64::> drop-ack-bp 0)))
              200000000 1000000000000))
     stores (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::sqlite-store::Handle])
                               _i  <- :wat::core::i64]
                -> (:wat::core::Vector :- [:wat::query::sqlite-store::Handle])
                (:wat::core::conj acc
                  (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
                    :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))))
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
                        :record (:queue::queue::Record :cap sub-cap :store-addr (:wat::query::sqlite-store::Handle/addr sh) :drop-recv-bp drop-recv-bp :drop-ack-bp drop-ack-bp :drop-seed drop-seed))]
                  (:wat::core::conj acc h)))
              (:wat::core::Vector :- [:queue::queue::Handle])
              (:wat::core::range 0 m))
     inbox-store (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
                   :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     inbox-qh (:queue::queue/start
                :locus (:wat::spawn::process/post-spawn
                         (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                           (:wat::query::sqlite-store/grant inbox-store (:fanout::pids pl))))
                :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr inbox-store) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     qaddrs (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
                (:wat::core::conj acc (:queue::queue::Handle/addr (:wat::core::nth queues i))))
              (:wat::core::Vector :- [(:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])])
              (:wat::core::range 0 m))
     th (:demo::topic/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:queue::queue/grant inbox-qh (:fanout::pids pl))))
          :record (:demo::topic::Record :nsubs m :inbox-addr (:queue::queue::Handle/addr inbox-qh) :inbox-lost 0 :inbox-closed 0 :inbox-timedout 0))
     twhandles (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:demo::topic-worker::Handle])
                                  _wi <- :wat::core::i64]
                   -> (:wat::core::Vector :- [:demo::topic-worker::Handle])
                   (:wat::core::conj acc
                     (:demo::topic-worker/start
                       :locus (:wat::spawn::process/post-spawn
                                (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                  (:wat::core::let
                                    [pids (:fanout::pids pl)
                                     _ (:queue::queue/grant inbox-qh pids)]
                                    (:wat::core::foldl
                                      (:wat::core::fn [a <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
                                        (:queue::queue/grant (:wat::core::nth queues i) pids))
                                      nil
                                      (:wat::core::range 0 m)))))
                       ;; 5s, not the 200ms row-3 vis: under a loaded floor, send+ack
                       ;; of one envelope can exceed 200ms, vis expires, a second
                       ;; worker re-sends, total > N×M. Refusal retry stays on the
                       ;; 200ms probe; the circuit happy path must not race its ack.
                       :record (:demo::mk-tw 5000000000 (:queue::queue::Handle/addr inbox-qh) qaddrs rate seed))))
                 (:wat::core::Vector :- [:demo::topic-worker::Handle])
                 (:wat::core::range 0 j))
     qclients (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::Queue])
                                 i   <- :wat::core::i64]
                  -> (:wat::core::Vector :- [:queue::Queue])
                  (:wat::core::conj acc
                    (:fanout::dial-queue (:queue::queue::Handle/addr (:wat::core::nth queues i)))))
                (:wat::core::Vector :- [:queue::Queue])
                (:wat::core::range 0 m))
     topic (:fanout::dial-topic (:demo::topic::Handle/addr th))
     phandles (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Vector :- [:fanout::publisher::Handle])
                                 i   <- :wat::core::i64]
                  -> (:wat::core::Vector :- [:fanout::publisher::Handle])
                  (:wat::core::conj acc
                    (:fanout::publisher/start
                      :locus (:wat::spawn::process/post-spawn
                               (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                 (:demo::topic/grant th (:fanout::pids pl))))
                      :record (:fanout::publisher::Record
                                :id (:wat::core::str i)
                                :topic-addr (:demo::topic::Handle/addr th)
                                :lo (:fanout::share-lo i n p)
                                :hi (:fanout::share-hi i n p)
                                :seed (:fanout::publisher-seed i)
                                :done false :calls 0 :retries 0 :asleep 0 :attempt 0 :attempts 0))))
                (:wat::core::Vector :- [:fanout::publisher::Handle])
                (:wat::core::range 0 p))
     _twgo (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
               (:demo::start-topic-worker!
                 (:demo::dial-topic-worker
                   (:demo::topic-worker::Handle/addr (:wat::core::nth twhandles i)))))
             nil
             (:wat::core::range 0 j))
     seenh (:fanout::seen/start :locus (:wat::spawn::process)
              :record (:fanout::seen::Record :recorded 0 :skipped 0
                        :drop-check-bp drop-check-bp :drop-mark-bp drop-mark-bp
                        :drop-seed drop-seed :drop-after? drop-after?))
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
                                     :locus (:wat::spawn::ProcessOpts
                                              :post-spawn-fn
                                                (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                                  (:wat::core::let
                                                    [pids (:fanout::pids pl)
                                                     _ (:queue::queue/grant qh pids)]
                                                    (:fanout::seen/grant seenh pids)))
                                              :env-fn "(:wat::program::EmptyEnv)"
                                              ;; Stop returns every first-seen Outcome. A bursty
                                              ;; queue can land all 2000 on one worker; 512 KiB
                                              ;; does not hold that vector.
                                              :max-message-bytes 2097152
                                              :runner-count (:wat::program::cpu-count)
                                              :label :wat::core::None)
                                     :record (:fanout::mk-worker
                                               (:fanout::wid qi wi)
                                               (:fanout::qname qi)
                                               vis 0 0
                                               (:queue::queue::Handle/addr qh)
                                               (:fanout::seen::Handle/addr seenh)
                                               rate seed))]
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
     _go-early (:wat::core::if fill-first? nil (:fanout::arm-workers! wpeers))
     t-pub0 (:wat::time::epoch-nanos (:wat::time::now))
     ppeers (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])])
                               i   <- :wat::core::i64]
                -> (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])])
                (:wat::core::conj acc
                  (:fanout::dial-publisher (:fanout::publisher::Handle/addr (:wat::core::nth phandles i)))))
              (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])])
              (:wat::core::range 0 p))
     _pgo (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::nil
                             w <- (:wat::kernel::Peer :- [:fanout::Publisher::Op :fanout::Publisher::Reply])]
              -> :wat::core::nil
              (:fanout::start-publisher! w))
            nil
            ppeers)
     pub-pair (:fanout::join-publishers ppeers)
     pub-calls (:wat::core::first (:wat::core::first pub-pair))
     pub-retries (:wat::core::second (:wat::core::first pub-pair))
     pub-asleep (:wat::core::first (:wat::core::second pub-pair))
     pub-attempts (:wat::core::second (:wat::core::second pub-pair))
     _filled (:wat::core::if fill-first?
               (:fanout::require! (:fanout::poll-until-filled qclients topic n (:wat::i64::* n m)))
               nil)
     fill-sweep (:fanout::sweep-of qclients)
     fill-depth (:fanout::snapshot-str fill-sweep)
     t-arm0 (:wat::time::epoch-nanos (:wat::time::now))
     _go-late (:wat::core::if fill-first? (:fanout::arm-workers! wpeers) nil)
     t-drain0 (:wat::time::epoch-nanos (:wat::time::now))
     ;; Drain-store samples sit inside the wall-clock span: before-sample
     ;; immediately after t-drain0, after-sample immediately before t-collect0.
     ;; After-sample's 8 stats calls land in the store delta; before-sample's
     ;; 8 do not (they are in sc-before). Timestamps were not moved.
     sc-before (:fanout::sum-store-calls qclients)
     ns-before (:fanout::sum-store-ns qclients)
     hn-before (:fanout::sum-handler-ns qclients)
     ;; One 5 ms poll slot per delivered pair (n×m). Hang if drain is slower
     ;; than 200 pairs/sec. Scales with the work; not a raised constant.
     drain-pair (:fanout::poll-until-drained qclients topic (:wat::i64::* n m))
     drain-err (:wat::core::first drain-pair)
     _drain (:fanout::require!
              (:wat::core::if (:wat::core::= drain-err "")
                ""
                (:wat::core::let [dp (:fanout::sum-disrupts wpeers)]
                  (:wat::core::format "{e};check-exhausted={ce};mark-exhausted={me};ack-retries={ar};ack-exhausted={ae}"
                    :e drain-err
                    :ce (:wat::core::first (:wat::core::second dp))
                    :me (:wat::core::second (:wat::core::second dp))
                    :ar (:wat::core::first (:wat::core::third dp))
                    :ae (:wat::core::second (:wat::core::third dp))))))
     poll-calls (:wat::core::second drain-pair)
     sc-after (:fanout::sum-store-calls qclients)
     ns-after (:fanout::sum-store-ns qclients)
     hn-after (:fanout::sum-handler-ns qclients)
     t-collect0 (:wat::time::epoch-nanos (:wat::time::now))
     calls (:fanout::sum-calls qclients)
     ticks (:fanout::sum-ticks qclients)
     store-calls (:fanout::sum-store-calls qclients)
     store-ns (:fanout::sum-store-ns qclients)
     tticks (:fanout::topic-ticks topic)
     ifails (:fanout::topic-inbox-fails topic)
     ilost  (:wat::core::first ifails)
     iclosed (:wat::core::second ifails)
     itimed (:wat::core::third ifails)
     dpair (:fanout::sum-disrupts wpeers)
     dhits (:wat::core::first dpair)
     ce    (:wat::core::first (:wat::core::second dpair))
     me    (:wat::core::second (:wat::core::second dpair))
     ars   (:wat::core::first (:wat::core::third dpair))
     aes   (:wat::core::second (:wat::core::third dpair))
     spair (:fanout::seen-stats seenh)
     sfirsts (:wat::core::first spair)
     sdups (:wat::core::second spair)
     outs (:fanout::collect-stop workers)
     empty-flags (:wat::core::foldl
                   (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
                     (:wat::core::let
                       [qp (:wat::core::nth qclients i)
                        now (:wat::time::epoch-nanos (:wat::time::now))
                        rr (:queue::Queue/receive qp
                             (:queue::Queue::ReceiveRequest
                               :queue (:fanout::qname i) :now-ns now :visibility-ns 1000000000000 :limit 1 :wait (:queue::Queue::Wait::Immediate)))]
                       (:wat::core::match rr
                         ((:wat::kernel::RecvOutcome::Message r)
                           (:wat::core::match r
                             ((:queue::Queue::ReceiveResponse::Ok envs)
                               (:wat::core::if (:wat::core::empty? envs) acc 0))
                             (_ 0)))
                         (_ 0))))
                   1
                   (:wat::core::range 0 m))
     summary0 (:fanout::summarize n m j outs empty-flags)
     summary (:wat::core::format "{s};seen-recorded={f};seen-skipped={d};check-exhausted={ce};mark-exhausted={me};ack-retries={ar};ack-exhausted={ae}"
               :s summary0 :f sfirsts :d sdups :ce ce :me me :ar ars :ae aes)
     t-stop0 (:wat::time::epoch-nanos (:wat::time::now))
     _stoptw (:wat::core::foldl
               (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
                 (:wat::core::let [_ (:demo::topic-worker/stop (:wat::core::nth twhandles i))]
                   nil))
               nil
               (:wat::core::range 0 j))
     t-end (:wat::time::epoch-nanos (:wat::time::now))
     ms (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
          (:wat::i64::/ (:wat::i64::- b a) 1000000))
     phases (:wat::core::format
              "setup={setup};fill={fill};arm={arm};drain={drain};collect={collect};stop={stop};fill-depth={fd};qticks={ticks};topic-ticks={tt};disrupts={dh};check-exhausted={ce};mark-exhausted={me};ack-retries={ar};ack-exhausted={ae};seen-recorded={sf};seen-skipped={sd};publish-calls={pc};full-retries={fr};inbox-lost={il};inbox-closed={ic};inbox-timedout={ito};asleep={asleep};publish-attempts={pa};poll-calls={polls};store-calls={sc};store-ms={sms};drain-store-calls={dsc};drain-store-ms={dsms};drain-busy-ms={dbms};total={total}"
              :setup (ms t-setup0 t-pub0)
              :fill (ms t-pub0 t-arm0)
              :arm (ms t-arm0 t-drain0)
              :drain (ms t-drain0 t-collect0)
              :collect (ms t-collect0 t-stop0)
              :stop (ms t-stop0 t-end)
              :fd fill-depth
              :ticks ticks
              :tt tticks
              :dh dhits
              :ce ce
              :me me
              :ar ars
              :ae aes
              :sf sfirsts
              :sd sdups
              :pc pub-calls
              :fr pub-retries
              :il ilost
              :ic iclosed
              :ito itimed
              :asleep pub-asleep
              :pa pub-attempts
              :polls poll-calls
              :sc store-calls
              :sms (:wat::i64::/ store-ns 1000000)
              :dsc (:wat::i64::/ (:wat::i64::- sc-after sc-before) m)
              :dsms (:wat::i64::/ (:wat::i64::- ns-after ns-before) (:wat::i64::* 1000000 m))
              :dbms (:wat::i64::/ (:wat::i64::- hn-after hn-before) (:wat::i64::* 1000000 m))
              :total (ms t-setup0 t-end))
     traces (:fanout::traces-report (:fanout::traces-of outs))]
    (:wat::core::Tuple summary calls
      (:wat::core::format "{p} ;; {tr}" :p phases :tr traces))))

(:wat::core::defn :user::run*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:fanout::run-with n m j 1 0 0 0 0 0 false 0 0 32 false 0))

(:wat::core::defn :user::run-p*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64  p <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:fanout::run-with n m j p 0 0 0 0 0 false 0 0 32 false 0))

(:wat::core::defn :user::run-chaos*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64
   rate <- :wat::core::i64  seed <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:fanout::run-with n m j 1 rate seed 0 0 0 false 0 0 32 false 0))

(:wat::core::defn :user::run-drop*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64
   drop-check-bp <- :wat::core::i64  drop-mark-bp <- :wat::core::i64
   drop-seed <- :wat::core::i64  drop-after? <- :wat::core::bool]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:fanout::run-with n m j 1 0 0 drop-check-bp drop-mark-bp drop-seed drop-after? 0 0 32 false 0))

(:wat::core::defn :user::drop-before-summary [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 2000 4 3 0 200 42 false)))

(:wat::core::defn :user::drop-after-summary [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 2000 4 3 0 200 42 true)))

(:wat::core::defn :user::drop-before-tiny [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 50 2 2 0 1000 42 false)))

(:wat::core::defn :user::drop-after-tiny [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 50 2 2 0 1000 42 true)))

(:wat::core::defn :user::drop-check-tiny [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 50 2 2 1000 0 42 true)))

(:wat::core::defn :user::drop-recv-tiny [] -> :wat::core::String
  (:wat::core::first (:fanout::run-with 50 2 2 1 0 0 0 0 42 true 1000 0 32 false 0)))

(:wat::core::defn :user::drop-ack-tiny [] -> :wat::core::String
  (:wat::core::first (:fanout::run-with 50 2 2 1 0 0 0 0 42 true 0 1000 32 false 0)))

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

;; Timeout, discard, redial, retry on the FRESH peer. Shown, not asserted.
;; Silent Hold never replies, so the deadline is honest (server is not behaving).
(:wat::core::defn :user::deadline-redial-is-fresh [] -> :wat::core::String
  (:wat::core::let
    [h (:fanout::hold/start :locus (:wat::spawn::process)
          :record (:fanout::hold::Record :tag 1))
     addr (:fanout::hold::Handle/addr h)
     kind :wat::program::PeerKind::process
     dummy (:fanout::Hold::Reply::Wait (:fanout::Hold::WaitResponse::Ok))
     first
       (:wat::core::let
         [p0 (:wat::core::match (:wat::kernel::connect addr)
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                (_ (:wat::kernel::assertion-failed! "deadline-redial: first dial failed" :wat::core::None :wat::core::None)))]
         (:wat::core::match (:wat::kernel::send p0 (:fanout::Hold::Op::Wait (:fanout::Hold::WaitRequest)))
           (:wat::kernel::SendOutcome::Sent
             (:wat::core::let
               [tmr (:wat::core::first
                      (:wat::core::conj
                        (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Hold::Op :fanout::Hold::Reply])])
                        (:wat::kernel::after kind (:wat::time::Milliseconds 200) dummy)))]
               (:wat::core::match (:wat::kernel::select [p0 tmr])
                 ((:wat::spawn::ServiceEvent::Message idx _m)
                   (:wat::core::if (:wat::i64::= idx 1) "timeout" "reply"))
                 ((:wat::spawn::ServiceEvent::Closed _i) "closed")
                 ((:wat::spawn::ServiceEvent::Lost _i _c) "lost")
                 (:wat::spawn::ServiceEvent::Shutdown "shutdown")
                 ((:wat::spawn::ServiceEvent::Admin _a) "admin")
                 ((:wat::spawn::ServiceEvent::Connection _p) "connection")
                 ((:wat::spawn::ServiceEvent::Malformed _i _c) "malformed")
                 ((:wat::spawn::ServiceEvent::Rejected _i _c) "rejected"))))
           (_ "send-failed")))
     retry
       (:wat::core::if (:wat::core::= first "timeout")
         ;; p0 from the inner let is gone — discarded. Redial, send the retry
         ;; on the new peer, and require that send to land (Sent).
         (:wat::core::match (:wat::kernel::connect addr)
           ((:wat::kernel::ConnectOutcome::Connected p1)
             (:wat::core::match (:wat::kernel::send p1 (:fanout::Hold::Op::Wait (:fanout::Hold::WaitRequest)))
               (:wat::kernel::SendOutcome::Sent "fresh")
               (:wat::kernel::SendOutcome::Closed "closed")
               (:wat::kernel::SendOutcome::Stopped "stopped")
               ((:wat::kernel::SendOutcome::Lost _c) "lost")))
           (_ "redial-failed"))
         "skipped")
     _stop (:fanout::hold/stop h)]
    (:wat::core::format
      "timeout={t};discarded=yes;redial=Connected;retry-on={r}"
      :t (:wat::core::if (:wat::core::= first "timeout") "yes" first)
      :r retry)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     proof (:user::deadline-redial-is-fresh)
     usage "usage: circuit.wat [n m j sub-cap fill-first? [vis-ms]]"
     triple
       (:wat::core::match (:wat::core::get argv 2)
         (:wat::core::None (:user::run* 2000 4 3))
         ((:wat::core::Some ns)
           (:fanout::run-with
             (:fanout::parse-i64 ns)
             (:fanout::parse-i64 (:wat::core::Option/expect (:wat::core::get argv 3) usage))
             (:fanout::parse-i64 (:wat::core::Option/expect (:wat::core::get argv 4) usage))
             1 0 0 0 0 0 false 0 0
             (:fanout::parse-i64 (:wat::core::Option/expect (:wat::core::get argv 5) usage))
             (:wat::core::= (:wat::core::Option/expect (:wat::core::get argv 6) usage) "true")
             (:wat::core::match (:wat::core::get argv 7)
               (:wat::core::None 0)
               ((:wat::core::Some vs) (:fanout::parse-i64 vs))))))]
    (:wat::core::let
      [_ (:wat::kernel::println proof)
       _ (:wat::kernel::println
           (:wat::core::format "queue-receive-calls={c}" :c (:wat::core::second triple)))
       _ (:wat::kernel::println (:wat::core::first triple))]
      (:wat::kernel::println (:wat::core::third triple)))))

(:wat::core::defn :user::chaos [] -> :wat::core::nil
  (:wat::core::let [triple (:user::run-chaos* 2000 4 3 200 42)]
    (:wat::core::let
      [_ (:wat::kernel::println
           (:wat::core::format "queue-receive-calls={c}" :c (:wat::core::second triple)))
       _ (:wat::kernel::println (:wat::core::first triple))]
      (:wat::kernel::println (:wat::core::third triple)))))

(:wat::core::defn :user::drop-after [] -> :wat::core::nil
  (:wat::core::let [triple (:user::run-drop* 2000 4 3 0 200 42 true)]
    (:wat::core::let
      [_ (:wat::kernel::println
           (:wat::core::format "queue-receive-calls={c}" :c (:wat::core::second triple)))
       _ (:wat::kernel::println (:wat::core::first triple))]
      (:wat::kernel::println (:wat::core::third triple)))))

(:wat::core::defn :user::drop-before [] -> :wat::core::nil
  (:wat::core::let [triple (:user::run-drop* 2000 4 3 0 200 42 false)]
    (:wat::core::let
      [_ (:wat::kernel::println
           (:wat::core::format "queue-receive-calls={c}" :c (:wat::core::second triple)))
       _ (:wat::kernel::println (:wat::core::first triple))]
      (:wat::kernel::println (:wat::core::third triple)))))

;; ★ Row 2: pending-only drain + delayed-ack worker MUST lose the held message.
(:wat::core::defn :user::pending-only-loses [] -> :wat::core::String
  (:wat::core::let
    [n 4
     msh (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh  (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::sqlite-store/grant msh (:fanout::pids pl))))
           :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::sqlite-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     hh  (:fanout::held-worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:queue::queue/grant qh (:fanout::pids pl))))
           :record (:fanout::held-worker::Record :id "held-0" :queue-name "q0"
                     :queue-addr (:queue::queue::Handle/addr qh)))
     q   (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     w   (:fanout::dial-worker (:fanout::held-worker::Handle/addr hh))
     _   (:fanout::start-worker! w)
     _pub (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
              (:wat::core::let
                [now (:wat::time::epoch-nanos (:wat::time::now))]
                (:wat::core::match
                  (:queue::Queue/send q
                    (:queue::Queue::SendRequest :queue "q0" :bodies (:wat::core::Vector :- [:wat::core::String] (:wat::core::str i)) :now-ns now))
                  ((:wat::kernel::RecvOutcome::Message _r) nil)
                  (_ nil))))
            nil
            (:wat::core::range 0 n))
     _ (:fanout::require! (:fanout::poll-until-visible-zero q 4000))
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
    [msh (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh  (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::sqlite-store/grant msh (:fanout::pids pl))))
           :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::sqlite-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     seenh (:fanout::seen/start :locus (:wat::spawn::process)
              :record (:fanout::seen::Record :recorded 0 :skipped 0 :drop-check-bp 0 :drop-mark-bp 0 :drop-seed 0 :drop-after? false))
     wh  (:fanout::worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::core::let
                        [pids (:fanout::pids pl)
                         _ (:queue::queue/grant qh pids)]
                        (:fanout::seen/grant seenh pids))))
           :record (:fanout::mk-worker "idle-0" "q0" 1000000000000 0 0
                     (:queue::queue::Handle/addr qh)
                     (:fanout::seen::Handle/addr seenh) 0 0))
     w   (:fanout::dial-worker (:fanout::worker::Handle/addr wh))
     _   (:fanout::start-worker! w)
     _   (:fanout::await-timer-ms 20)
     t0  (:wat::time::epoch-nanos (:wat::time::now))
     _   (:fanout::worker/stop wh)
     t1  (:wat::time::epoch-nanos (:wat::time::now))
     dt  (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    (:wat::core::format "dt-ms={dt}" :dt dt)))

;; ★ Row 3: drain without the inbox term MUST lose accepted-but-undelivered messages.
;; No topic-workers, so the N rows sit in the inbox while subscriber queues look empty.
(:wat::core::defn :user::outbox-term-loses [] -> :wat::core::String
  (:wat::core::let
    [n 4
     msh (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh  (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::sqlite-store/grant msh (:fanout::pids pl))))
           :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::sqlite-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     ish (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     iqh (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::sqlite-store/grant ish (:fanout::pids pl))))
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr ish) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     th  (:demo::topic/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:queue::queue/grant iqh (:fanout::pids pl))))
           :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh) :inbox-lost 0 :inbox-closed 0 :inbox-timedout 0))
     seenh (:fanout::seen/start :locus (:wat::spawn::process)
              :record (:fanout::seen::Record :recorded 0 :skipped 0 :drop-check-bp 0 :drop-mark-bp 0 :drop-seed 0 :drop-after? false))
     wh  (:fanout::worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::core::let
                        [pids (:fanout::pids pl)
                         _ (:queue::queue/grant qh pids)]
                        (:fanout::seen/grant seenh pids))))
           :record (:fanout::mk-worker "ob-0" "q0" 1000000000000 0 0
                     (:queue::queue::Handle/addr qh)
                     (:fanout::seen::Handle/addr seenh) 0 0))
     topic (:fanout::dial-topic (:demo::topic::Handle/addr th))
     q     (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     w     (:fanout::dial-worker (:fanout::worker::Handle/addr wh))
     _     (:fanout::start-worker! w)
     _pub  (:fanout::publish-n-until-accepted! topic n)
     _     (:fanout::require! (:fanout::poll-until-visible-zero q 4000))
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

;; ★ Row 1: two SENDs of the same seq (what a topic-worker retry actually does —
;; each send mints a new envelope uuid). Dedupe is off; parent records both.
;; Envelope ids differ; seq does not. distinct on seq is 1, total is 2, dup=1.
;; Keying on envelope id would report distinct=2 and hide the duplicate.
(:wat::core::defn :user::redelivery-is-visible [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     send1 (:wat::core::fn [] -> :wat::core::nil
             (:wat::core::match
               (:queue::Queue/send q
                 (:queue::Queue::SendRequest :queue "q0"
                   :bodies (:wat::core::Vector :- [:wat::core::String] "7|hello")
                   :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
               ((:wat::kernel::RecvOutcome::Message _r) nil)
               (_ (:wat::kernel::assertion-failed! "redelivery-visible: send failed" :wat::core::None :wat::core::None))))
     _ (send1)
     _ (send1)
     take (:wat::core::fn [] -> (:wat::core::Tuple :- [:wat::core::String :wat::core::String])
            (:wat::core::match
              (:queue::Queue/receive q
                (:queue::Queue::ReceiveRequest
                  :queue "q0" :now-ns (:wat::time::epoch-nanos (:wat::time::now))
                  :visibility-ns 1000000000000 :limit 1 :wait (:queue::Queue::Wait::Immediate)))
              ((:wat::kernel::RecvOutcome::Message r)
                (:wat::core::match r
                  ((:queue::Queue::ReceiveResponse::Ok envs)
                    (:wat::core::if (:wat::core::empty? envs)
                      (:wat::core::Tuple "" "")
                      (:wat::core::let [e (:wat::core::first envs)]
                        (:wat::core::Tuple (:queue::Envelope/id e)
                          (:fanout::seq-of (:queue::Envelope/body e))))))
                  (_ (:wat::kernel::assertion-failed! "redelivery-visible: receive not Ok" :wat::core::None :wat::core::None))))
              (_ (:wat::kernel::assertion-failed! "redelivery-visible: recv failed" :wat::core::None :wat::core::None))))
     first (take)
     second (take)
     id1 (:wat::core::first first)
     seq1 (:wat::core::second first)
     id2 (:wat::core::first second)
     seq2 (:wat::core::second second)
     total (:wat::core::if (:wat::core::= id2 "") 1 2)
     distinct (:wat::core::if (:wat::core::and (:wat::core::= seq1 seq2) (:wat::core::not (:wat::core::= seq1 ""))) 1 2)]
    (:wat::core::format
      "total={t};distinct={d};dup={dup};same-seq={s};envelopes-differ={e}"
      :t total :d distinct :dup (:wat::core::- total distinct)
      :s (:wat::core::if (:wat::core::= seq1 seq2) "yes" "no")
      :e (:wat::core::if (:wat::core::= id1 id2) "no" "yes"))))

;; ★ Row 2: the same redelivery, consumed. Two workers, vis 200ms, ack-delay 350ms,
;; shared seen. First claims; second sees Dup and drops. One outcome.
(:wat::core::defn :user::redelivery-is-absorbed [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     seenh (:fanout::seen/start :locus (:wat::spawn::thread)
              :record (:fanout::seen::Record :recorded 0 :skipped 0 :drop-check-bp 0 :drop-mark-bp 0 :drop-seed 0 :drop-after? false))
     w1 (:fanout::worker/start :locus (:wat::spawn::thread)
          :record (:fanout::mk-worker "a" "q0" 200000000 350 0
                    (:queue::queue::Handle/addr qh)
                    (:fanout::seen::Handle/addr seenh) 0 0))
     w2 (:fanout::worker/start :locus (:wat::spawn::thread)
          :record (:fanout::mk-worker "b" "q0" 200000000 350 0
                    (:queue::queue::Handle/addr qh)
                    (:fanout::seen::Handle/addr seenh) 0 0))
     q  (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     _  (:fanout::start-worker! (:fanout::dial-worker (:fanout::worker::Handle/addr w1)))
     _  (:fanout::start-worker! (:fanout::dial-worker (:fanout::worker::Handle/addr w2)))
     _  (:wat::core::match
          (:queue::Queue/send q
            (:queue::Queue::SendRequest :queue "q0"
              :bodies (:wat::core::Vector :- [:wat::core::String] "7|hello")
              :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
          ((:wat::kernel::RecvOutcome::Message _r) nil)
          (_ nil))
     _  (:fanout::await-timer-ms 800)
     o1 (:fanout::worker/stop w1)
     o2 (:fanout::worker/stop w2)
     outs (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:fanout::Outcome])
                             o   <- :fanout::Outcome]
              -> (:wat::core::PersistentVector :- [:fanout::Outcome])
              (:wat::vector::conj acc o))
            o1
            o2)
     total (:wat::core::count outs)
     distinct (:wat::core::count
                (:wat::hashmap::keys
                  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                                     o   <- :fanout::Outcome]
                      -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                      (:wat::hashmap::assoc acc (:fanout::key-of o) true))
                    (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                    outs)))
     spair (:fanout::seen-stats seenh)
     sfirsts (:wat::core::first spair)
     sdups (:wat::core::second spair)]
    (:wat::core::format
      "total={t};distinct={d};dup={dup};seen-recorded={f};seen-skipped={sd}"
      :t total :d distinct :dup (:wat::core::- total distinct)
      :f sfirsts :sd sdups)))

;; s3 window: redelivery arrives mid-processing. vis 200ms, work-delay 350ms,
;; ack-delay 0. A checks Absent and naps; vis expires; B checks Absent too.
;; Both emit. distinct=1 is the invariant (not a loss). total=2 today.
(:wat::core::defn :user::redelivery-mid-processing [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     seenh (:fanout::seen/start :locus (:wat::spawn::thread)
              :record (:fanout::seen::Record :recorded 0 :skipped 0 :drop-check-bp 0 :drop-mark-bp 0 :drop-seed 0 :drop-after? false))
     w1 (:fanout::worker/start :locus (:wat::spawn::thread)
          :record (:fanout::mk-worker "a" "q0" 200000000 0 350
                    (:queue::queue::Handle/addr qh)
                    (:fanout::seen::Handle/addr seenh) 0 0))
     w2 (:fanout::worker/start :locus (:wat::spawn::thread)
          :record (:fanout::mk-worker "b" "q0" 200000000 0 350
                    (:queue::queue::Handle/addr qh)
                    (:fanout::seen::Handle/addr seenh) 0 0))
     q  (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     _  (:fanout::start-worker! (:fanout::dial-worker (:fanout::worker::Handle/addr w1)))
     _  (:fanout::start-worker! (:fanout::dial-worker (:fanout::worker::Handle/addr w2)))
     _  (:wat::core::match
          (:queue::Queue/send q
            (:queue::Queue::SendRequest :queue "q0"
              :bodies (:wat::core::Vector :- [:wat::core::String] "7|hello")
              :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
          ((:wat::kernel::RecvOutcome::Message _r) nil)
          (_ nil))
     _  (:fanout::await-timer-ms 800)
     o1 (:fanout::worker/stop w1)
     o2 (:fanout::worker/stop w2)
     outs (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:fanout::Outcome])
                             o   <- :fanout::Outcome]
              -> (:wat::core::PersistentVector :- [:fanout::Outcome])
              (:wat::vector::conj acc o))
            o1
            o2)
     total (:wat::core::count outs)
     distinct (:wat::core::count
                (:wat::hashmap::keys
                  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                                     o   <- :fanout::Outcome]
                      -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                      (:wat::hashmap::assoc acc (:fanout::key-of o) true))
                    (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                    outs)))
     spair (:fanout::seen-stats seenh)
     sfirsts (:wat::core::first spair)
     sdups (:wat::core::second spair)]
    (:wat::core::format
      "total={t};distinct={d};dup={dup};seen-recorded={f};seen-skipped={sd}"
      :t total :d distinct :dup (:wat::core::- total distinct)
      :f sfirsts :sd sdups)))
