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
  [(:wat::core::defrecord :fanout::Seen::ClaimRequest
     [queue <- :wat::core::String
      seq   <- :wat::core::String])
   (:wat::core::defenum :fanout::Seen::ClaimResponse :wat::enum::Pure
     :First []
     :Dup []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :fanout::Seen::StatsRequest [])
   (:wat::core::defenum :fanout::Seen::StatsResponse :wat::enum::Pure
     :Ok [firsts <- :wat::core::i64  dups <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(claim [self <- :fanout::Seen  req <- :fanout::Seen::ClaimRequest]
     -> :fanout::Seen::ClaimResponse :max-request-bytes 524288)
   (stats [self <- :fanout::Seen  req <- :fanout::Seen::StatsRequest]
     -> :fanout::Seen::StatsResponse :max-request-bytes 524288)])

(:wat::service::defservice :fanout::seen
  :satisfies :fanout::Seen
  ;; 256: small enough that a 2KB disrupt claim severs ONE worker's connection.
  ;; Normal claims (`q0/1999`) fit. Contract cap stays 524288. Thread locus
  ;; does not tear — that is a property, not a second mechanism.
  :max-frame-bytes 256
  ;; Counters are durable so a stats read is a fact about this run.
  ;; claimed stays ephemeral: the ledger does not cross the wire and does not
  ;; survive hibernation (S31). Restart seen and every message looks First again.
  :durable   [firsts        <- :wat::core::i64
              dups          <- :wat::core::i64
              drop-rate-bp  <- :wat::core::i64
              drop-seed     <- :wat::core::i64
              drop-after?   <- :wat::core::bool]
  :ephemeral [claimed <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])]
  :init (:wat::core::fn [record <- :fanout::seen::Record] -> :fanout::seen::State
          (:fanout::seen::State :durable record
            :claimed (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])))
  :impls
  [(claim [s ctx req]
     (:wat::core::let
       [key (:wat::string::concat (:fanout::Seen::ClaimRequest/queue req)
               (:wat::string::concat "/" (:fanout::Seen::ClaimRequest/seq req)))
        claimed (:fanout::seen::State/claimed s)
        rec     (:fanout::seen::State/durable s)
        rate   (:fanout::seen::Record/drop-rate-bp rec)
        after? (:fanout::seen::Record/drop-after? rec)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Seen::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::seen::Op])])
        pair (:wat::core::if (:wat::i64::> rate 0)
               (:wat::rand::int-from (:fanout::seen::Record/drop-seed rec) 0 10000)
               (:wat::core::Tuple (:fanout::seen::Record/drop-seed rec) 0))
        seed1 (:wat::core::first pair)
        bp    (:wat::core::second pair)
        hit?  (:wat::core::and (:wat::i64::> rate 0) (:wat::i64::< bp rate))
        already? (:wat::core::match (:wat::hashmap::get claimed key)
                   ((:wat::core::Some _) true)
                   (:wat::core::None false))
        write? (:wat::core::or (:wat::core::not hit?) after?)
        firsts' (:wat::core::if (:wat::core::and write? (:wat::core::not already?))
                  (:wat::i64::+ (:fanout::seen::Record/firsts rec) 1)
                  (:fanout::seen::Record/firsts rec))
        dups' (:wat::core::if (:wat::core::and write? already?)
                (:wat::i64::+ (:fanout::seen::Record/dups rec) 1)
                (:fanout::seen::Record/dups rec))
        claimed' (:wat::core::if (:wat::core::and write? (:wat::core::not already?))
                   (:wat::hashmap::assoc claimed key true)
                   claimed)
        rec' (:fanout::seen::Record
               :firsts firsts' :dups dups'
               :drop-rate-bp rate :drop-seed seed1 :drop-after? after?)
        s' (:fanout::seen::State :durable rec' :claimed claimed')
        reply (:wat::core::if hit?
                :wat::core::None
                (:wat::core::Some
                  (:fanout::Seen::Reply::Claim
                    (:wat::core::if already?
                      (:fanout::Seen::ClaimResponse::Dup)
                      (:fanout::Seen::ClaimResponse::First)))))]
       (:wat::service::Outcome::Continue s' reply sends none-alarms)))
   (stats [s ctx req]
     (:wat::core::let
       [rec (:fanout::seen::State/durable s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Seen::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:fanout::seen::Op])])]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:fanout::Seen::Reply::Stats
           (:fanout::Seen::StatsResponse::Ok
             (:fanout::seen::Record/firsts rec)
             (:fanout::seen::Record/dups rec))))
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
     :Ok [hits <- :wat::core::i64  draws <- :wat::core::i64  points <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
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
              delay-ms   <- :wat::core::i64
              queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])
              seen-addr  <- (:wat::kernel::Address :- [:fanout::Seen::Op :fanout::Seen::Reply])
              disrupt-rate-bp   <- :wat::core::i64
              disrupt-seed      <- :wat::core::i64
              disrupt-lo-ms     <- :wat::core::i64
              disrupt-hi-ms     <- :wat::core::i64
              disrupt-max-draws <- :wat::core::i64
              disrupt-hits      <- :wat::core::i64
              disrupt-draws     <- :wat::core::i64
              disrupt-points    <- :wat::core::String]
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
                    :delay-ms (:fanout::worker::Record/delay-ms rec)
                    :queue-addr (:fanout::worker::Record/queue-addr rec)
                    :seen-addr (:fanout::worker::Record/seen-addr rec)
                    :disrupt-rate-bp rate
                    :disrupt-seed seed1
                    :disrupt-lo-ms (:fanout::worker::Record/disrupt-lo-ms rec)
                    :disrupt-hi-ms (:fanout::worker::Record/disrupt-hi-ms rec)
                    :disrupt-max-draws (:fanout::worker::Record/disrupt-max-draws rec)
                    :disrupt-hits (:fanout::worker::Record/disrupt-hits rec)
                    :disrupt-draws (:fanout::worker::Record/disrupt-draws rec)
                    :disrupt-points (:fanout::worker::Record/disrupt-points rec))
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
             (:fanout::worker::Record/disrupt-points rec))))
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
                      (:fanout::Seen/claim old
                        (:fanout::Seen::ClaimRequest :queue "disrupt" :seq pad))
                      ((:wat::kernel::RecvOutcome::Message _r) "message")
                      ((:wat::kernel::RecvOutcome::Lost _c) "lost")
                      (:wat::kernel::RecvOutcome::Closed "closed")
                      (:wat::kernel::RecvOutcome::Stopped
                        (:wat::kernel::assertion-failed! "fanout worker: disrupt poison stopped" :wat::core::None :wat::core::None)))
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
                :delay-ms (:fanout::worker::Record/delay-ms rec)
                :queue-addr (:fanout::worker::Record/queue-addr rec)
                :seen-addr (:fanout::worker::Record/seen-addr rec)
                :disrupt-rate-bp rate
                :disrupt-seed seed2
                :disrupt-lo-ms lo
                :disrupt-hi-ms hi
                :disrupt-max-draws maxd
                :disrupt-hits hits'
                :disrupt-draws draws
                :disrupt-points points')
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
        delay (:fanout::worker::Record/delay-ms rec)
        outs (:fanout::worker::State/outcomes s)
        now  (:wat::time::epoch-nanos (:wat::time::now))
        rr   (:queue::Queue/receive q
               (:queue::Queue::ReceiveRequest
                 :queue name :now-ns now :visibility-ns vis :limit 10 :wait (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 250))))]
       (:wat::core::match rr
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:queue::Queue::ReceiveResponse::Ok envs)
               (:wat::core::let
                 [t4 (:wat::time::epoch-nanos (:wat::time::now))
                  triple (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                                         (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                                                                         (:wat::core::PersistentVector :- [:fanout::Outcome])])
                                           e   <- :queue::Envelope]
                            -> (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
                                                      (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                                                      (:wat::core::PersistentVector :- [:fanout::Outcome])])
                            (:wat::core::let
                              [q0    (:wat::core::first acc)
                               seen0 (:wat::core::second acc)
                               outs0 (:wat::core::third acc)
                               eid   (:queue::Envelope/id e)
                               raw   (:queue::Envelope/body e)
                               parts (:wat::string::split raw "|")
                               seq   (:wat::core::if (:wat::core::empty? parts) "" (:wat::core::first parts))
                               req   (:fanout::Seen::ClaimRequest :queue name :seq seq)
                               addr  (:fanout::worker::Record/seen-addr rec)
                               kind  (:wat::program::Env/peer-kind (:wat::program::env))
                               ;; Local so the process impl's checker sees the return type.
                               ;; Same-file defns are unsolved :? across the fork (thread is fine).
                               redial (:wat::core::fn []
                                         -> (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                                         (:wat::core::match (:wat::kernel::connect addr)
                                           ((:wat::kernel::ConnectOutcome::Connected p) p)
                                           (_ (:wat::kernel::assertion-failed! "fanout worker: redial seen failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))))
                               once  (:wat::core::fn
                                       [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])]
                                       -> (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                                                                 (:wat::core::Option :- [:fanout::Seen::ClaimResponse])
                                                                 :wat::core::i64])
                                       ;; third: 0 replied, 1 noreply (redialed), 2 timeout (redialed, retry)
                                       (:wat::core::match (:wat::kernel::send peer (:fanout::Seen::Op::Claim req))
                                         (:wat::kernel::SendOutcome::Sent
                                           (:wat::core::let
                                             [tmr (:wat::core::first
                                                    (:wat::core::conj
                                                      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])])
                                                      ;; 200 ms: well above a healthy hashmap claim; short
                                                      ;; enough that a drop run does not stall publish
                                                      ;; (5000 ms * ~2% of 8000 backed the inbox to never-accepted).
                                                      (:wat::kernel::after kind (:wat::time::Milliseconds 200)
                                                        (:fanout::Seen::Reply::Claim (:fanout::Seen::ClaimResponse::First)))))]
                                             (:wat::core::match (:wat::kernel::select [peer tmr])
                                               ((:wat::spawn::ServiceEvent::Message idx m)
                                                 (:wat::core::if (:wat::i64::= idx 0)
                                                   (:wat::core::match m
                                                     ((:fanout::Seen::Reply::Claim resp)
                                                       (:wat::core::Tuple peer (:wat::core::Some resp) 0))
                                                     (_ (:wat::kernel::assertion-failed! "fanout worker: claim reply misrouted" :wat::core::None :wat::core::None)))
                                                   (:wat::core::Tuple (redial) :wat::core::None 2)))
                                               ((:wat::spawn::ServiceEvent::Closed idx)
                                                 (:wat::core::if (:wat::i64::= idx 0)
                                                   (:wat::core::Tuple (redial) :wat::core::None 1)
                                                   (:wat::core::Tuple (redial) :wat::core::None 2)))
                                               ((:wat::spawn::ServiceEvent::Lost idx _c)
                                                 (:wat::core::if (:wat::i64::= idx 0)
                                                   (:wat::core::Tuple (redial) :wat::core::None 1)
                                                   (:wat::core::Tuple (redial) :wat::core::None 2)))
                                               (:wat::spawn::ServiceEvent::Shutdown
                                                 (:wat::kernel::assertion-failed! "fanout worker: claim select shutdown" :wat::core::None :wat::core::None))
                                               ((:wat::spawn::ServiceEvent::Admin _msg)
                                                 (:wat::kernel::assertion-failed! "fanout worker: claim select admin" :wat::core::None :wat::core::None))
                                               ((:wat::spawn::ServiceEvent::Connection _p)
                                                 (:wat::kernel::assertion-failed! "fanout worker: claim select connection" :wat::core::None :wat::core::None))
                                               ((:wat::spawn::ServiceEvent::Malformed idx _c)
                                                 (:wat::core::if (:wat::i64::= idx 0)
                                                   (:wat::core::Tuple (redial) :wat::core::None 1)
                                                   (:wat::kernel::assertion-failed! "fanout worker: claim deadline timer malformed" :wat::core::None :wat::core::None)))
                                               ((:wat::spawn::ServiceEvent::Rejected idx _c)
                                                 (:wat::core::if (:wat::i64::= idx 0)
                                                   (:wat::core::Tuple (redial) :wat::core::None 1)
                                                   (:wat::kernel::assertion-failed! "fanout worker: claim deadline timer rejected" :wat::core::None :wat::core::None))))))
                                         (:wat::kernel::SendOutcome::Closed
                                           (:wat::core::Tuple (redial) :wat::core::None 1))
                                         (:wat::kernel::SendOutcome::Stopped
                                           (:wat::kernel::assertion-failed! "fanout worker: claim send stopped" :wat::core::None :wat::core::None))
                                         ((:wat::kernel::SendOutcome::Lost _c)
                                           (:wat::core::Tuple (redial) :wat::core::None 1))))
                               started-ns (:wat::time::epoch-nanos (:wat::time::now))
                               a1    (once seen0)
                               a2    (:wat::core::if (:wat::i64::= (:wat::core::third a1) 2) (once (:wat::core::first a1)) a1)
                               a3    (:wat::core::if (:wat::i64::= (:wat::core::third a2) 2) (once (:wat::core::first a2)) a2)
                               pair  (:wat::core::if (:wat::i64::= (:wat::core::third a3) 2)
                                       (:wat::kernel::assertion-failed!
                                         (:wat::core::format
                                           "fanout worker: claim deadline exhausted;depth=3;attempts=3;elapsed={e}"
                                           :e (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) started-ns) 1000000))
                                         :wat::core::None :wat::core::None)
                                       a3)
                               seen1 (:wat::core::first pair)]
                              (:wat::core::match (:wat::core::second pair)
                                ((:wat::core::Some cresp)
                                  (:wat::core::let
                                    [first? (:wat::core::match cresp
                                              ((:fanout::Seen::ClaimResponse::First) true)
                                              ((:fanout::Seen::ClaimResponse::Dup) false)
                                              (_ (:wat::kernel::assertion-failed! "fanout worker: claim not First/Dup" :wat::core::None :wat::core::None)))
                                     _nap (:wat::core::if (:wat::i64::> delay 0)
                                             (:wat::core::match
                                               (:wat::kernel::recv
                                                 (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds delay) :done))
                                               ((:wat::kernel::RecvOutcome::Message _m) nil)
                                               (_ nil))
                                             nil)
                                     ebody (:wat::core::format "{b}|{t}" :b raw :t t4)
                                     ar    (:queue::Queue/ack q0
                                             (:queue::Queue::AckRequest :queue name :id eid))
                                     outs1 (:wat::core::if first?
                                             (:wat::vector::conj outs0
                                               (:fanout::Outcome :worker wid :queue name :id eid :body ebody))
                                             outs0)]
                                    (:wat::core::match ar
                                      ((:wat::kernel::RecvOutcome::Message _ar)
                                        (:wat::core::Tuple q0 seen1 outs1))
                                      ((:wat::kernel::RecvOutcome::Lost _cause)
                                        ;; Claim landed; record First. Do not retry ack —
                                        ;; vis + Dup absorb if the delete did not.
                                        (:wat::core::Tuple
                                          (:wat::core::match
                                            (:wat::kernel::connect (:fanout::worker::Record/queue-addr rec))
                                            ((:wat::kernel::ConnectOutcome::Connected p) p)
                                            (_ (:wat::kernel::assertion-failed! "fanout worker: redial queue failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                                          seen1
                                          outs1))
                                      (:wat::kernel::RecvOutcome::Stopped
                                        (:wat::kernel::assertion-failed! "fanout worker: ack stopped" :wat::core::None :wat::core::None))
                                      (:wat::kernel::RecvOutcome::Closed
                                        ;; Claim landed; record First. Do not retry ack —
                                        ;; vis + Dup absorb if the delete did not.
                                        (:wat::core::Tuple
                                          (:wat::core::match
                                            (:wat::kernel::connect (:fanout::worker::Record/queue-addr rec))
                                            ((:wat::kernel::ConnectOutcome::Connected p) p)
                                            (_ (:wat::kernel::assertion-failed! "fanout worker: redial queue failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
                                          seen1
                                          outs1)))))
                                (:wat::core::None
                                  ;; Lost/Closed (and send-fail) already redialed. Do not ack.
                                  ;; If the claim landed, vis + Dup absorb.
                                  (:wat::core::Tuple q0 seen1 outs0)))))
                          (:wat::core::Tuple q seen outs)
                          envs)
                  s' (:fanout::worker::State :durable rec
                       :q (:wat::core::first triple)
                       :seen (:wat::core::second triple)
                       :outcomes (:wat::core::third triple))]
                 (:wat::service::SelfOutcome::Continue s'
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)])))
             (_ (:wat::kernel::assertion-failed! "fanout worker: receive not Ok" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost _cause)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:fanout::worker::Record/queue-addr rec))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "fanout worker: redial queue failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:fanout::worker::State :durable rec :q fresh :seen seen :outcomes outs)]
             (:wat::service::SelfOutcome::Continue s'
               (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)])))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "fanout worker: receive stopped" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
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
         (:fanout::Worker::DisruptsResponse::Ok 0 0 "")))
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
                                 (:queue::Queue::AckRequest :queue name :id eid))]
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
                              outs0)))))
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
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:fanout::Worker::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)]))))))))])

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
    (:wat::kernel::RecvOutcome::Closed nil)))

;; Timer-channel recv, not a sleep — legal where mora forbids sleeping.
(:wat::core::defn :fanout::await-timer-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

;; Parent-side constructor. Chaos fields default off (rate 0 arms nothing).
(:wat::core::defn :fanout::mk-worker
  [id         <- :wat::core::String
   queue-name <- :wat::core::String
   vis-ns     <- :wat::core::i64
   delay-ms   <- :wat::core::i64
   queue-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])
   seen-addr  <- (:wat::kernel::Address :- [:fanout::Seen::Op :fanout::Seen::Reply])
   rate-bp    <- :wat::core::i64
   seed       <- :wat::core::i64]
  -> :fanout::worker::Record
  (:fanout::worker::Record
    :id id :queue-name queue-name :vis-ns vis-ns :delay-ms delay-ms
    :queue-addr queue-addr :seen-addr seen-addr
    :disrupt-rate-bp rate-bp :disrupt-seed seed
    :disrupt-lo-ms 50 :disrupt-hi-ms 150 :disrupt-max-draws 0
    :disrupt-hits 0 :disrupt-draws 0 :disrupt-points ""))

;; Sentinel: -1 means unread. Matches ticks-of / q-depth. (1,1) satisfied both waits.
(:wat::core::defn :fanout::depth-of
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible unacked)
          (:wat::core::Tuple visible unacked))
        (_ (:wat::core::Tuple -1 -1))))
    (_ (:wat::core::Tuple -1 -1))))

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
        (_ -1)))
    (_ -1)))

;; TEMPORARY INSTRUMENT — how many -deliver ticks did the topic take for N messages?
;; One tick per message means a timer arm + fire + select wake is paid per message.
(:wat::core::defn :fanout::topic-ticks [t <- :demo::Topic] -> :wat::core::i64
  (:wat::core::match (:demo::Topic/stats t (:demo::Topic::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::StatsResponse::Ok _n ticks) ticks)
        (_ -1)))
    (_ -1)))

(:wat::core::defn :fanout::any-unread?
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  q <- :queue::Queue] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::= (:wat::core::first (:fanout::depth-of q)) -1)))
    false
    qclients))

(:wat::core::defn :fanout::fully-drained?
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic] -> :wat::core::bool
  (:wat::core::and (:fanout::all-drained? qclients)
    (:wat::core::= (:fanout::topic-outbox t) 0)))

(:wat::core::defn :fanout::require!
  [r <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:wat::core::= r "")
    nil
    (:wat::kernel::assertion-failed! r :wat::core::None :wat::core::None)))

(:wat::core::defn :fanout::elapsed-ms [start-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns) 1000000))

(:wat::core::defn :fanout::depth-snapshot
  [qclients <- (:wat::core::Vector :- [:queue::Queue])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  q <- :queue::Queue] -> :wat::core::String
      (:wat::core::let [d (:fanout::depth-of q)]
        (:wat::core::format "{acc}[{v}/{u}]"
          :acc acc :v (:wat::core::first d) :u (:wat::core::second d))))
    ""
    qclients))

;; Conjunction across N queues plus the topic inbox. No single wire event.
;; Bounded, and it reports what it last saw — the check rung, taken only
;; where the shape rung is unavailable.
(:wat::core::defn :fanout::poll-until-drained*
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic
   left <- :wat::core::i64  start-ns <- :wat::core::i64  total <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [snap (:fanout::depth-snapshot qclients)
     box  (:fanout::topic-outbox t)]
    (:wat::core::if (:wat::core::or (:wat::core::= box -1) (:fanout::any-unread? qclients))
      (:wat::core::format "drained-unread: last={s} outbox={b} attempts={a} elapsed={ms}"
        :s snap :b box :a (:wat::i64::- total left) :ms (:fanout::elapsed-ms start-ns))
      (:wat::core::if (:fanout::fully-drained? qclients t)
        ""
        (:wat::core::if (:wat::i64::<= left 1)
          (:wat::core::format "drained-never: last={s} outbox={b} attempts={a} elapsed={ms}"
            :s snap :b box :a total :ms (:fanout::elapsed-ms start-ns))
          (:wat::core::let [_ (:fanout::await-timer-ms 5)]
            (:fanout::poll-until-drained* qclients t (:wat::i64::- left 1) start-ns total)))))))

(:wat::core::defn :fanout::poll-until-drained
  [qclients <- (:wat::core::Vector :- [:queue::Queue])  t <- :demo::Topic  attempts <- :wat::core::i64]
  -> :wat::core::String
  (:fanout::poll-until-drained* qclients t attempts (:wat::time::epoch-nanos (:wat::time::now)) attempts))

;; LIVENESS BOUND — only a hang may trip this. Full is correct backpressure
;; (the queue is bounded; a waiting producer is the design). Giving up loses
;; the message. 60000 ms is the floor from
;; BRIEF-278-a-liveness-bound-only-catches-a-hang: a red here is STUCK, never
;; "the box was busy". Force-expire via publish-until-accepted!* with
;; limit-ms 0 against a full inbox.
(:wat::core::defn :fanout::publish-until-accepted!*
  [t <- :demo::Topic  msg <- :wat::core::String
   attempts <- :wat::core::i64  start-ns <- :wat::core::i64  limit-ms <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match (:demo::Topic/publish t (:demo::Topic::PublishRequest :msg msg))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::PublishResponse::Ok) "")
        ((:demo::Topic::PublishResponse::Full d c)
          (:wat::core::let [elapsed (:fanout::elapsed-ms start-ns)]
            (:wat::core::if (:wat::i64::>= elapsed limit-ms)
              (:wat::core::format "verdict=never-accepted;depth={d};cap={c};attempts={a};elapsed={ms}"
                :d d :c c :a attempts :ms elapsed)
              (:wat::core::let [_ (:fanout::await-timer-ms 1)]
                (:fanout::publish-until-accepted!* t msg (:wat::i64::+ attempts 1) start-ns limit-ms)))))
        (_ (:wat::kernel::assertion-failed! "fanout: publish not Ok/Full" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "fanout: publish stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "fanout: publish closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :fanout::publish-until-accepted!
  [t <- :demo::Topic  msg <- :wat::core::String] -> :wat::core::nil
  (:fanout::require!
    (:fanout::publish-until-accepted!* t msg 1 (:wat::time::epoch-nanos (:wat::time::now)) 60000)))

(:wat::core::defn :fanout::publish-stamped-until-accepted!
  [t <- :demo::Topic  msg <- :wat::core::String] -> :wat::core::nil
  (:fanout::publish-until-accepted! t
    (:wat::core::format "{m}|{t0}"
      :m msg
      :t0 (:wat::time::epoch-nanos (:wat::time::now)))))

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
            ((:queue::Queue::StatsResponse::Ok calls _ticks _visible _unacked)
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
            ((:queue::Queue::StatsResponse::Ok _calls ticks _visible _unacked)
              (:wat::i64::+ acc ticks))
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
          ((:fanout::Seen::StatsResponse::Ok firsts dups) (:wat::core::Tuple firsts dups))
          ((:fanout::Seen::StatsResponse::RequestTooLarge _b _c)
            (:wat::kernel::assertion-failed! "fanout: seen stats too large" :wat::core::None :wat::core::None))
          ((:fanout::Seen::StatsResponse::RequestMalformed _p _e _g)
            (:wat::kernel::assertion-failed! "fanout: seen stats malformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost _c)
        (:wat::kernel::assertion-failed! "fanout: seen stats lost" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "fanout: seen stats stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "fanout: seen stats closed" :wat::core::None :wat::core::None)))))

(:wat::core::defn :fanout::sum-disrupts
  [wpeers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])])]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64
                     w   <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])]
      -> :wat::core::i64
      (:wat::core::match (:fanout::Worker/disrupts w (:fanout::Worker::DisruptsRequest))
        ((:wat::kernel::RecvOutcome::Message r)
          (:wat::core::match r
            ((:fanout::Worker::DisruptsResponse::Ok hits _draws _points) (:wat::i64::+ acc hits))
            ((:fanout::Worker::DisruptsResponse::RequestTooLarge _b _c) acc)
            ((:fanout::Worker::DisruptsResponse::RequestMalformed _p _e _g) acc)))
        ((:wat::kernel::RecvOutcome::Lost _c) acc)
        (:wat::kernel::RecvOutcome::Stopped acc)
        (:wat::kernel::RecvOutcome::Closed acc)))
    0
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
  [outbox  <- :fanout::Hist
   hop12   <- :fanout::Hist
   hop23   <- :fanout::Hist
   pending <- :fanout::Hist
   e2e     <- :fanout::Hist
   sample  <- :wat::core::String])

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
    (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::count parts) 6))
      tr
      (:wat::core::let
        [t0 (:fanout::parse-i64 (:wat::core::nth parts 1))
         t1 (:fanout::parse-i64 (:wat::core::nth parts 2))
         t2 (:fanout::parse-i64 (:wat::core::nth parts 3))
         t3 (:fanout::parse-i64 (:wat::core::nth parts 4))
         t4 (:fanout::parse-i64 (:wat::core::nth parts 5))
         sample (:wat::core::if (:wat::core::= (:fanout::Traces/sample tr) "")
                  (:fanout::Outcome/body o)
                  (:fanout::Traces/sample tr))]
        (:fanout::Traces
          :outbox  (:fanout::hist-add (:fanout::Traces/outbox tr)  (:fanout::ns->ms t0 t1))
          :hop12   (:fanout::hist-add (:fanout::Traces/hop12 tr)   (:fanout::ns->ms t1 t2))
          :hop23   (:fanout::hist-add (:fanout::Traces/hop23 tr)   (:fanout::ns->ms t2 t3))
          :pending (:fanout::hist-add (:fanout::Traces/pending tr) (:fanout::ns->ms t3 t4))
          :e2e     (:fanout::hist-add (:fanout::Traces/e2e tr)     (:fanout::ns->ms t0 t4))
          :sample  sample)))))

(:wat::core::defn :fanout::traces-of
  [outs <- (:wat::core::Vector :- [:fanout::Outcome])]
  -> :fanout::Traces
  (:wat::core::foldl
    :fanout::traces-add
    (:fanout::Traces
      :outbox  (:fanout::empty-hist)
      :hop12   (:fanout::empty-hist)
      :hop23   (:fanout::empty-hist)
      :pending (:fanout::empty-hist)
      :e2e     (:fanout::empty-hist)
      :sample  "")
    outs))

(:wat::core::defn :fanout::traces-report [tr <- :fanout::Traces] -> :wat::core::String
  (:wat::core::format
    "sample={s} ;; {o} ;; {a} ;; {b} ;; {c} ;; {e}"
    :s (:fanout::Traces/sample tr)
    :o (:fanout::hist-line "outbox  " (:fanout::Traces/outbox tr))
    :a (:fanout::hist-line "t1->t2  " (:fanout::Traces/hop12 tr))
    :b (:fanout::hist-line "t2->t3  " (:fanout::Traces/hop23 tr))
    :c (:fanout::hist-line "t3->t4  " (:fanout::Traces/pending tr))
    :e (:fanout::hist-line "e2e     " (:fanout::Traces/e2e tr))))

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
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64
   rate <- :wat::core::i64  seed <- :wat::core::i64
   drop-rate <- :wat::core::i64  drop-seed <- :wat::core::i64  drop-after? <- :wat::core::bool]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:wat::core::let
    [t-setup0 (:wat::time::epoch-nanos (:wat::time::now))
     ;; Drop runs: 200 ms vis so an unacked envelope (no claim-reply) becomes
     ;; visible again. T1's 200 ms claim deadline retries the same worker;
     ;; vis expiry is the other worker. Both are retries of a dropped reply.
     vis (:wat::core::if (:wat::i64::> drop-rate 0) 200000000 1000000000000)
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
                        :record (:queue::queue::Record :cap 32 :store-addr (:wat::query::sqlite-store::Handle/addr sh)))]
                  (:wat::core::conj acc h)))
              (:wat::core::Vector :- [:queue::queue::Handle])
              (:wat::core::range 0 m))
     inbox-store (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
                   :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     inbox-qh (:queue::queue/start
                :locus (:wat::spawn::process/post-spawn
                         (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                           (:wat::query::sqlite-store/grant inbox-store (:fanout::pids pl))))
                :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr inbox-store)))
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
          :record (:demo::topic::Record :nsubs m :inbox-addr (:queue::queue::Handle/addr inbox-qh)))
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
     _twgo (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
               (:demo::start-topic-worker!
                 (:demo::dial-topic-worker
                   (:demo::topic-worker::Handle/addr (:wat::core::nth twhandles i)))))
             nil
             (:wat::core::range 0 j))
     seenh (:fanout::seen/start :locus (:wat::spawn::process)
              :record (:fanout::seen::Record :firsts 0 :dups 0
                        :drop-rate-bp drop-rate :drop-seed drop-seed :drop-after? drop-after?))
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
                                                (:wat::core::let
                                                  [pids (:fanout::pids pl)
                                                   _ (:queue::queue/grant qh pids)]
                                                  (:fanout::seen/grant seenh pids))))
                                     :record (:fanout::mk-worker
                                               (:fanout::wid qi wi)
                                               (:fanout::qname qi)
                                               vis 0
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
     _go (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::nil  w <- (:wat::kernel::Peer :- [:fanout::Worker::Op :fanout::Worker::Reply])] -> :wat::core::nil
             (:fanout::start-worker! w))
           nil
           wpeers)
     t-pub0 (:wat::time::epoch-nanos (:wat::time::now))
     _pub (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
              (:fanout::publish-stamped-until-accepted! topic (:wat::core::str i)))
            nil
            (:wat::core::range 0 n))
     t-drain0 (:wat::time::epoch-nanos (:wat::time::now))
     _drain (:fanout::require! (:fanout::poll-until-drained qclients topic 4000))
     t-stop0 (:wat::time::epoch-nanos (:wat::time::now))
     calls (:fanout::sum-calls qclients)
     ticks (:fanout::sum-ticks qclients)
     tticks (:fanout::topic-ticks topic)
     dhits (:fanout::sum-disrupts wpeers)
     spair (:fanout::seen-stats seenh)
     sfirsts (:wat::core::first spair)
     sdups (:wat::core::second spair)
     outs (:fanout::collect-stop workers)
     _stoptw (:wat::core::foldl
               (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
                 (:wat::core::let [_ (:demo::topic-worker/stop (:wat::core::nth twhandles i))]
                   nil))
               nil
               (:wat::core::range 0 j))
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
     summary (:wat::core::format "{s};seen-firsts={f};seen-dups={d}"
               :s summary0 :f sfirsts :d sdups)
     t-end (:wat::time::epoch-nanos (:wat::time::now))
     ms (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
          (:wat::i64::/ (:wat::i64::- b a) 1000000))
     phases (:wat::core::format
              "setup={setup};publish={pub};drain={drain};stop={stop};qticks={ticks};topic-ticks={tt};disrupts={dh};seen-firsts={sf};seen-dups={sd}"
              :setup (ms t-setup0 t-pub0)
              :pub (ms t-pub0 t-drain0)
              :drain (ms t-drain0 t-stop0)
              :stop (ms t-stop0 t-end)
              :ticks ticks
              :tt tticks
              :dh dhits
              :sf sfirsts
              :sd sdups)
     traces (:fanout::traces-report (:fanout::traces-of outs))]
    (:wat::core::Tuple summary calls
      (:wat::core::format "{p} ;; {tr}" :p phases :tr traces))))

(:wat::core::defn :user::run*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:fanout::run-with n m j 0 0 0 0 false))

(:wat::core::defn :user::run-chaos*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64
   rate <- :wat::core::i64  seed <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:fanout::run-with n m j rate seed 0 0 false))

(:wat::core::defn :user::run-drop*
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64
   drop-rate <- :wat::core::i64  drop-seed <- :wat::core::i64  drop-after? <- :wat::core::bool]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64 :wat::core::String])
  (:fanout::run-with n m j 0 0 drop-rate drop-seed drop-after?))

(:wat::core::defn :user::drop-before-summary [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 2000 4 3 200 42 false)))

(:wat::core::defn :user::drop-after-summary [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 2000 4 3 200 42 true)))

(:wat::core::defn :user::drop-before-tiny [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 50 2 2 1000 42 false)))

(:wat::core::defn :user::drop-after-tiny [] -> :wat::core::String
  (:wat::core::first (:user::run-drop* 50 2 2 1000 42 true)))

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
    [proof (:user::deadline-redial-is-fresh)
     triple (:user::run* 2000 4 3)]
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
  (:wat::core::let [triple (:user::run-drop* 2000 4 3 200 42 true)]
    (:wat::core::let
      [_ (:wat::kernel::println
           (:wat::core::format "queue-receive-calls={c}" :c (:wat::core::second triple)))
       _ (:wat::kernel::println (:wat::core::first triple))]
      (:wat::kernel::println (:wat::core::third triple)))))

(:wat::core::defn :user::drop-before [] -> :wat::core::nil
  (:wat::core::let [triple (:user::run-drop* 2000 4 3 200 42 false)]
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
           :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::sqlite-store::Handle/addr msh)))
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
           :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::sqlite-store::Handle/addr msh)))
     seenh (:fanout::seen/start :locus (:wat::spawn::process)
              :record (:fanout::seen::Record :firsts 0 :dups 0 :drop-rate-bp 0 :drop-seed 0 :drop-after? false))
     wh  (:fanout::worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::core::let
                        [pids (:fanout::pids pl)
                         _ (:queue::queue/grant qh pids)]
                        (:fanout::seen/grant seenh pids))))
           :record (:fanout::mk-worker "idle-0" "q0" 1000000000000 0
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
           :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::sqlite-store::Handle/addr msh)))
     ish (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     iqh (:queue::queue/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::query::sqlite-store/grant ish (:fanout::pids pl))))
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr ish)))
     th  (:demo::topic/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:queue::queue/grant iqh (:fanout::pids pl))))
           :record (:demo::topic::Record :nsubs 1 :inbox-addr (:queue::queue::Handle/addr iqh)))
     seenh (:fanout::seen/start :locus (:wat::spawn::process)
              :record (:fanout::seen::Record :firsts 0 :dups 0 :drop-rate-bp 0 :drop-seed 0 :drop-after? false))
     wh  (:fanout::worker/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:wat::core::let
                        [pids (:fanout::pids pl)
                         _ (:queue::queue/grant qh pids)]
                        (:fanout::seen/grant seenh pids))))
           :record (:fanout::mk-worker "ob-0" "q0" 1000000000000 0
                     (:queue::queue::Handle/addr qh)
                     (:fanout::seen::Handle/addr seenh) 0 0))
     topic (:fanout::dial-topic (:demo::topic::Handle/addr th))
     q     (:fanout::dial-queue (:queue::queue::Handle/addr qh))
     w     (:fanout::dial-worker (:fanout::worker::Handle/addr wh))
     _     (:fanout::start-worker! w)
     _pub  (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
               (:fanout::publish-stamped-until-accepted! topic (:wat::core::str i)))
             nil
             (:wat::core::range 0 n))
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
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr msh)))
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

;; ★ Row 2: the same redelivery, consumed. Two workers, vis 200ms, delay 350ms,
;; shared seen. First claims; second sees Dup and drops. One outcome.
(:wat::core::defn :user::redelivery-is-absorbed [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::sqlite-store::Handle/addr msh)))
     seenh (:fanout::seen/start :locus (:wat::spawn::thread)
              :record (:fanout::seen::Record :firsts 0 :dups 0 :drop-rate-bp 0 :drop-seed 0 :drop-after? false))
     w1 (:fanout::worker/start :locus (:wat::spawn::thread)
          :record (:fanout::mk-worker "a" "q0" 200000000 350
                    (:queue::queue::Handle/addr qh)
                    (:fanout::seen::Handle/addr seenh) 0 0))
     w2 (:fanout::worker/start :locus (:wat::spawn::thread)
          :record (:fanout::mk-worker "b" "q0" 200000000 350
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
      "total={t};distinct={d};dup={dup};seen-firsts={f};seen-dups={sd}"
      :t total :d distinct :dup (:wat::core::- total distinct)
      :f sfirsts :sd sdups)))
