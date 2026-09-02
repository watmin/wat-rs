;; wat-scripts/queue/sqs.wat — wat-queue: send / receive / ack with a visibility timeout.
;;
;; Lives in userland on the wat-grep / wat-gen precedent — built here, promoted to
;; wat/queue.wat when it demonstrates excellence, and that promotion is the builder's
;; ruling. See wat-scripts/queue/README.md. Sibling: wat-scripts/topic/ (wat-topic).
;;
;; ★ THE CLOCK IS AN ARGUMENT. `:wat::time::now` is real and cannot be stepped, and
;;   `mora` forbids a sleep. send/receive therefore take `now-ns` (epoch nanos) so a
;;   fixture can drive the visibility window as a value: receive at T, then receive
;;   at T+timeout, with no wall-clock wait. Callers pass
;;   `(:wat::time::epoch-nanos (:wat::time::now))`. Instant/Duration on the request
;;   record is avoided — journal's wire-proven i64 time-ns is the precedent.
;;
;; Design (every primitive already ships):
;;   pk  = the queue name
;;   sk  = a STABLE message id (`:wat::uuid::v4`, written; `ack` names it forever)
;;   GSI "by-visible-at": ipk = queue name, isk = (:wat::edn::write Instant)
;;   send    → put, isk = now
;;   receive → scan-index isk <= now, take N, RE-PUT each with isk = now + timeout
;;             (one atomic put, no lock, no timer, no base-table read — IndexRow
;;             carries pk sk ipk isk data, wat/query.wat:46)
;;   ack     → delete by (pk, sk)
;;
;; isk-hi is `write(now)` — inclusive, so a message visible at exactly `now` is
;; returned. Demonstrated by the bound= row of :user::compute, not argued.
;;
;; :user::compute runs the full lifecycle against mem-store AND sqlite-store
;; (:memory:, :index-names ["by-visible-at"]) and returns the agreed summary
;; (or DIFFERENTIAL-MISMATCH). :user::main prints it. Shape copied from
;; tests/services/probe_ex001_journal_same_ns.wat and wat-scripts/topic/sns-fanout.wat.
;;
;; No bijection-anchor: the one ephemeral peer field (`store`) is a scalar Peer,
;; so `:peers [:wat::query::Store]` is a true bijection (journal.wat's shape).

;; ── surface ─────────────────────────────────────────────────────────────────────
(:wat::core::defsurface :queue::Queue :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :queue::Envelope
     [id   <- :wat::core::String
      body <- :wat::core::String])

   (:wat::core::defrecord :queue::Queue::SendRequest
     [queue <- :wat::core::String
      body  <- :wat::core::String
      now-ns <- :wat::core::i64])
   (:wat::core::defenum :queue::Queue::SendResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :queue::Queue::ReceiveRequest
     [queue         <- :wat::core::String
      now-ns        <- :wat::core::i64
      visibility-ns <- :wat::core::i64
      limit         <- :wat::core::i64
      wait-ns       <- :wat::core::i64])
   (:wat::core::defrecord :queue::Queue::StatsRequest [])
   ;; pending = visible (not yet received). in-flight = received, not yet acked.
   ;; Both: stopping a worker that holds an unacked message loses that outcome —
   ;; the message stays invisible until its visibility timeout and the run ends
   ;; first. SQS exposes the same pair for the same reason.
   (:wat::core::defenum :queue::Queue::StatsResponse :wat::enum::Pure
     :Ok [receive-calls <- :wat::core::i64  ticks <- :wat::core::i64
          pending <- :wat::core::i64  in-flight <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defenum :queue::Queue::ReceiveResponse :wat::enum::Pure
     :Ok [envelopes <- (:wat::core::Vector :- [:queue::Envelope])]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :queue::Waiter
     [conn-id        <- :wat::core::i64
      queue          <- :wat::core::String
      limit          <- :wat::core::i64
      visibility-ns  <- :wat::core::i64
      deadline-ns    <- :wat::core::i64])

   (:wat::core::defrecord :queue::Queue::AckRequest
     [queue <- :wat::core::String
      id    <- :wat::core::String])
   (:wat::core::defenum :queue::Queue::AckResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(send    [self <- :queue::Queue  req <- :queue::Queue::SendRequest]
     -> :queue::Queue::SendResponse :max-request-bytes 524288)
   (receive [self <- :queue::Queue  req <- :queue::Queue::ReceiveRequest]
     -> :queue::Queue::ReceiveResponse :max-request-bytes 524288)
   (ack     [self <- :queue::Queue  req <- :queue::Queue::AckRequest]
     -> :queue::Queue::AckResponse :max-request-bytes 524288)
   (stats   [self <- :queue::Queue  req <- :queue::Queue::StatsRequest]
     -> :queue::Queue::StatsResponse :max-request-bytes 524288)])

;; ── service (holds a Store peer; init declares the GSI) ─────────────────────────
(:wat::service::defservice :queue::queue
  :satisfies :queue::Queue
  :durable   []
  :ephemeral [store         <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
              take          <- [:wat::core::String :wat::core::i64 :wat::core::i64 :wat::core::i64 :-> (:wat::core::Vector :- [:queue::Envelope])]
              waiters       <- (:wat::core::PersistentVector :- [:queue::Waiter])
              outbox        <- (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
              receive-calls <- :wat::core::i64
              ticks         <- :wat::core::i64
              pending       <- :wat::core::i64
              in-flight     <- :wat::core::i64]
  :peers     [:wat::query::Store]
  :init (:wat::core::fn
          [record     <- :queue::queue::Record
           store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
          -> :queue::queue::State
          (:wat::core::let
            [store (:wat::core::match (:wat::kernel::connect store-addr)
                     ((:wat::kernel::ConnectOutcome::Connected p) p)
                     ((:wat::kernel::ConnectOutcome::Refused c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                     ((:wat::kernel::ConnectOutcome::Rejected c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                     ((:wat::kernel::ConnectOutcome::Failed c)
                       (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
             _es   (:wat::query::Store/ensure-schema store
                     (:wat::query::Store::EnsureSchemaRequest
                       :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
                       :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                                  (:wat::query::IndexSchema
                                    :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
             ;; Closed over `store`. The one receive path — process children do not
             ;; see sibling defns, so the body lives here, called via State/take.
             take (:wat::core::fn
                    [q <- :wat::core::String  now-ns <- :wat::core::i64
                     vis-ns <- :wat::core::i64  lim <- :wat::core::i64]
                    -> (:wat::core::Vector :- [:queue::Envelope])
                    (:wat::core::let
                      [lo (:wat::edn::write (:wat::time::at-nanos 0))
                       hi (:wat::edn::write (:wat::time::at-nanos now-ns))
                       scan (:wat::query::Store/scan-index store
                               (:wat::query::Store::ScanIndexRequest
                                 :index "by-visible-at" :ipk q :isk-lo lo :isk-hi hi :limit lim :cursor :wat::core::None))]
                      (:wat::core::match scan
                        ((:wat::kernel::RecvOutcome::Message sresp)
                          (:wat::core::match sresp
                            ((:wat::query::Store::ScanIndexResponse::Success irows _c)
                              (:wat::core::if (:wat::core::empty? irows)
                                (:wat::core::Vector :- [:queue::Envelope])
                                (:wat::core::let
                                  [hide-at (:wat::edn::write (:wat::time::at-nanos (:wat::core::+ now-ns vis-ns)))
                                   put-rows (:wat::core::foldl
                                              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::StoredRow])
                                                               r   <- :wat::query::IndexRow]
                                                -> (:wat::core::Vector :- [:wat::query::StoredRow])
                                                (:wat::core::conj acc
                                                  (:wat::query::StoredRow
                                                    :pk (:wat::query::IndexRow/pk r)
                                                    :sk (:wat::query::IndexRow/sk r)
                                                    :data (:wat::query::IndexRow/data r)
                                                    :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                                                                  "by-visible-at" (:wat::query::IndexKey
                                                                                    :ipk (:wat::query::IndexRow/ipk r)
                                                                                    :isk hide-at)))))
                                              (:wat::core::Vector :- [:wat::query::StoredRow])
                                              irows)
                                   envs (:wat::core::foldl
                                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:queue::Envelope])
                                                           r   <- :wat::query::IndexRow]
                                            -> (:wat::core::Vector :- [:queue::Envelope])
                                            (:wat::core::conj acc
                                              (:queue::Envelope
                                                :id (:wat::query::IndexRow/sk r)
                                                :body (:wat::query::IndexRow/data r))))
                                          (:wat::core::Vector :- [:queue::Envelope])
                                          irows)
                                   put-resp (:wat::query::Store/put store
                                              (:wat::query::Store::PutRequest put-rows))]
                                  (:wat::core::match put-resp
                                    ((:wat::kernel::RecvOutcome::Message presp)
                                      (:wat::core::match presp
                                        ((:wat::query::Store::PutResponse::Success) envs)
                                        (_ (:wat::kernel::assertion-failed! "queue.take: re-put failed" :wat::core::None :wat::core::None))))
                                    ((:wat::kernel::RecvOutcome::Lost cause)
                                      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                                    (:wat::kernel::RecvOutcome::Stopped
                                      (:wat::kernel::assertion-failed! "queue.take: stop requested mid re-put" :wat::core::None :wat::core::None))
                                    (:wat::kernel::RecvOutcome::Closed
                                      (:wat::kernel::assertion-failed! "queue.take: store peer closed" :wat::core::None :wat::core::None))))))
                            (_ (:wat::kernel::assertion-failed! "queue.take: scan-index failed" :wat::core::None :wat::core::None))))
                        ((:wat::kernel::RecvOutcome::Lost cause)
                          (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Stopped
                          (:wat::kernel::assertion-failed! "queue.take: stop requested" :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Closed
                          (:wat::kernel::assertion-failed! "queue.take: store peer closed" :wat::core::None :wat::core::None)))))]
            (:queue::queue::State
              :durable record
              :store store
              :take take
              :waiters (:wat::core::PersistentVector :- [:queue::Waiter])
              :outbox (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
              :receive-calls 0
              :ticks 0
              :pending 0
              :in-flight 0)))
  :impls
  [(send [s ctx req]
     (:wat::core::let
       [store (:queue::queue::State/store s)
        q      (:queue::Queue::SendRequest/queue req)
        body   (:queue::Queue::SendRequest/body req)
        now-ns (:queue::Queue::SendRequest/now-ns req)
        sk     (:wat::edn::write (:wat::uuid::v4))
        isk    (:wat::edn::write (:wat::time::at-nanos now-ns))
        row   (:wat::query::StoredRow
                :pk q :sk sk :data body
                :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                              "by-visible-at" (:wat::query::IndexKey :ipk q :isk isk)))
        put-resp (:wat::query::Store/put store
                   (:wat::query::Store::PutRequest
                     (:wat::core::Vector :- [:wat::query::StoredRow] row)))]
       (:wat::core::match put-resp
         ((:wat::kernel::RecvOutcome::Message sresp)
           (:wat::core::match sresp
             ((:wat::query::Store::PutResponse::Success)
               (:wat::core::let
                 [s' (:queue::queue::State
                       :durable (:queue::queue::State/durable s)
                       :store store
                       :take (:queue::queue::State/take s)
                       :waiters (:queue::queue::State/waiters s)
                       :outbox (:queue::queue::State/outbox s)
                       :receive-calls (:queue::queue::State/receive-calls s)
                       :ticks (:queue::queue::State/ticks s)
                       :pending (:wat::i64::+ (:queue::queue::State/pending s) 1)
                       :in-flight (:queue::queue::State/in-flight s))]
                 (:wat::core::if (:wat::core::empty? (:queue::queue::State/waiters s'))
                   (:wat::service::Outcome::Continue s' (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))
                   (:wat::core::let
                     [pair (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::Tuple :- [(:wat::core::PersistentVector :- [:queue::Waiter])
                                                                            (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                                                                            :wat::core::i64])
                                              w   <- :queue::Waiter]
                               -> (:wat::core::Tuple :- [(:wat::core::PersistentVector :- [:queue::Waiter])
                                                         (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                                                         :wat::core::i64])
                               (:wat::core::let
                                 [keep (:wat::core::first acc)
                                  box  (:wat::core::second acc)
                                  taken (:wat::core::third acc)
                                  empty-ok (:queue::Queue::Reply::Receive
                                             (:queue::Queue::ReceiveResponse::Ok
                                               (:wat::core::Vector :- [:queue::Envelope])))]
                                 (:wat::core::if (:wat::i64::<= (:queue::Waiter/deadline-ns w) now-ns)
                                   (:wat::core::Tuple keep
                                     (:wat::core::conj box
                                       (:wat::service::Directed :conn-id (:queue::Waiter/conn-id w) :reply empty-ok))
                                     taken)
                                   (:wat::core::let
                                     [envs (:wat::core::apply (:queue::queue::State/take s')
                                              (:queue::Waiter/queue w)
                                              [now-ns
                                               (:queue::Waiter/visibility-ns w)
                                               (:queue::Waiter/limit w)])]
                                     (:wat::core::if (:wat::core::empty? envs)
                                       (:wat::core::Tuple (:wat::vector::conj keep w) box taken)
                                       (:wat::core::Tuple keep
                                         (:wat::core::conj box
                                           (:wat::service::Directed
                                             :conn-id (:queue::Waiter/conn-id w)
                                             :reply (:queue::Queue::Reply::Receive
                                                      (:queue::Queue::ReceiveResponse::Ok envs))))
                                         (:wat::i64::+ taken (:wat::core::count envs))))))))
                             (:wat::core::Tuple
                               (:wat::core::PersistentVector :- [:queue::Waiter])
                               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                               0)
                             (:queue::queue::State/waiters s'))
                      keep (:wat::core::first pair)
                      box  (:wat::core::second pair)
                      taken (:wat::core::third pair)
                      p0 (:queue::queue::State/pending s')
                      f0 (:queue::queue::State/in-flight s')
                      p1 (:wat::core::if (:wat::i64::< p0 taken) 0 (:wat::i64::- p0 taken))
                      f1 (:wat::i64::+ f0 taken)
                      delay (:wat::core::foldl
                              (:wat::core::fn [d <- :wat::core::i64  w <- :queue::Waiter] -> :wat::core::i64
                                (:wat::core::let [rem (:wat::core::- (:queue::Waiter/deadline-ns w) now-ns)]
                                  (:wat::core::if (:wat::i64::< rem d) rem d)))
                              1000000000000000
                              keep)
                      delay0 (:wat::core::if (:wat::i64::< delay 1000000) 1000000 delay)
                      s2 (:queue::queue::State
                           :durable (:queue::queue::State/durable s')
                           :store store
                           :take (:queue::queue::State/take s')
                           :waiters keep
                           :outbox (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                           :receive-calls (:queue::queue::State/receive-calls s')
                           :ticks (:queue::queue::State/ticks s')
                           :pending p1
                           :in-flight f1)
                      ok (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Ok)))]
                     (:wat::core::if (:wat::core::empty? keep)
                       (:wat::service::Outcome::Continue s2 ok box
                         (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))
                       (:wat::service::Outcome::Continue s2 ok box
                         [(:wat::service::Alarm :after (:wat::time::Nanosecond delay0) :op :-tick)]))))))
             (_ (:wat::kernel::assertion-failed! "queue.send: store put failed" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost cause)
           (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "queue.send: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "queue.send: store peer closed" :wat::core::None :wat::core::None)))))

   (receive [s ctx req]
     (:wat::core::let
       [store (:queue::queue::State/store s)
        q      (:queue::Queue::ReceiveRequest/queue req)
        now-ns (:queue::Queue::ReceiveRequest/now-ns req)
        vis-ns (:queue::Queue::ReceiveRequest/visibility-ns req)
        lim    (:queue::Queue::ReceiveRequest/limit req)
        wait   (:queue::Queue::ReceiveRequest/wait-ns req)
        calls  (:wat::i64::+ (:queue::queue::State/receive-calls s) 1)
        envs   (:wat::core::apply (:queue::queue::State/take s) q [now-ns vis-ns lim])
        n      (:wat::core::count envs)
        p0     (:queue::queue::State/pending s)
        f0     (:queue::queue::State/in-flight s)
        p1     (:wat::core::if (:wat::core::empty? envs) p0
                 (:wat::core::if (:wat::i64::< p0 n) 0 (:wat::i64::- p0 n)))
        f1     (:wat::core::if (:wat::core::empty? envs) f0 (:wat::i64::+ f0 n))
        s-n    (:queue::queue::State
                 :durable (:queue::queue::State/durable s)
                 :store store
                 :take (:queue::queue::State/take s)
                 :waiters (:queue::queue::State/waiters s)
                 :outbox (:queue::queue::State/outbox s)
                 :receive-calls calls
                 :ticks (:queue::queue::State/ticks s)
                 :pending p1
                 :in-flight f1)]
       (:wat::core::if (:wat::core::not (:wat::core::empty? envs))
         (:wat::service::Outcome::Continue s-n (:wat::core::Some (:queue::Queue::Reply::Receive (:queue::Queue::ReceiveResponse::Ok envs))) (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))
         (:wat::core::if (:wat::i64::<= wait 0)
           (:wat::service::Outcome::Continue s-n
             (:wat::core::Some (:queue::Queue::Reply::Receive (:queue::Queue::ReceiveResponse::Ok
               (:wat::core::Vector :- [:queue::Envelope])))) (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))
           (:wat::core::let
             [was-empty? (:wat::core::empty? (:queue::queue::State/waiters s-n))
              w (:queue::Waiter
                  :conn-id (:wat::service::Invocation/conn-id ctx)
                  :queue q
                  :limit lim
                  :visibility-ns vis-ns
                  :deadline-ns (:wat::core::+ (:wat::service::Invocation/start-ns ctx) wait))
              s-w (:queue::queue::State
                    :durable (:queue::queue::State/durable s-n)
                    :store store
                    :take (:queue::queue::State/take s-n)
                    :waiters (:wat::vector::conj (:queue::queue::State/waiters s-n) w)
                    :outbox (:queue::queue::State/outbox s-n)
                    :receive-calls calls
                    :ticks (:queue::queue::State/ticks s-n)
                    :pending p1
                    :in-flight f1)]
             (:wat::core::if was-empty?
               (:wat::service::Outcome::Continue s-w
                 :wat::core::None (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) [(:wat::service::Alarm :after (:wat::time::Nanosecond wait) :op :-tick)])
               (:wat::service::Outcome::Continue s-w :wat::core::None (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))))))))

   (ack [s ctx req]
     (:wat::core::let
       [store (:queue::queue::State/store s)
        q     (:queue::Queue::AckRequest/queue req)
        id    (:queue::Queue::AckRequest/id req)
        del   (:wat::query::Store/delete store
                (:wat::query::Store::DeleteRequest
                  (:wat::core::Vector :- [:wat::query::Key]
                    (:wat::query::Key :pk q :sk id))))]
       (:wat::core::match del
         ((:wat::kernel::RecvOutcome::Message sresp)
           (:wat::core::match sresp
             ((:wat::query::Store::DeleteResponse::Success)
               (:wat::core::let
                 [f0 (:queue::queue::State/in-flight s)
                  f1 (:wat::core::if (:wat::i64::<= f0 0) 0 (:wat::i64::- f0 1))
                  s' (:queue::queue::State
                       :durable (:queue::queue::State/durable s)
                       :store store
                       :take (:queue::queue::State/take s)
                       :waiters (:queue::queue::State/waiters s)
                       :outbox (:queue::queue::State/outbox s)
                       :receive-calls (:queue::queue::State/receive-calls s)
                       :ticks (:queue::queue::State/ticks s)
                       :pending (:queue::queue::State/pending s)
                       :in-flight f1)]
                 (:wat::service::Outcome::Continue s' (:wat::core::Some (:queue::Queue::Reply::Ack (:queue::Queue::AckResponse::Ok))) (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))))
             (_ (:wat::kernel::assertion-failed! "queue.ack: store delete failed" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost cause)
           (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "queue.ack: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "queue.ack: store peer closed" :wat::core::None :wat::core::None)))))

   (stats [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:queue::Queue::Reply::Stats (:queue::Queue::StatsResponse::Ok
         (:queue::queue::State/receive-calls s)
         (:queue::queue::State/ticks s)
         (:queue::queue::State/pending s)
         (:queue::queue::State/in-flight s)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])))

   ;; Scanning tick: expire past-deadline waiters and try-receive the rest
   ;; (same take-visible the receive arm uses). Sends and re-arm compose in
   ;; one SelfOutcome — no extra arm, no 1 ms stand-in for "and".
   (-tick [s ctx]
     (:wat::core::let
       [now   (:wat::service::SelfInvocation/start-ns ctx)
        store (:queue::queue::State/store s)
        ticks (:wat::i64::+ (:queue::queue::State/ticks s) 1)
        pair  (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Tuple :- [(:wat::core::PersistentVector :- [:queue::Waiter])
                                                               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                                                               :wat::core::i64])
                                 w   <- :queue::Waiter]
                  -> (:wat::core::Tuple :- [(:wat::core::PersistentVector :- [:queue::Waiter])
                                            (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                                            :wat::core::i64])
                  (:wat::core::let
                    [keep (:wat::core::first acc)
                     box  (:wat::core::second acc)
                     taken (:wat::core::third acc)
                     empty-ok (:queue::Queue::Reply::Receive
                                (:queue::Queue::ReceiveResponse::Ok
                                  (:wat::core::Vector :- [:queue::Envelope])))]
                    (:wat::core::if (:wat::i64::<= (:queue::Waiter/deadline-ns w) now)
                      (:wat::core::Tuple keep
                        (:wat::core::conj box
                          (:wat::service::Directed :conn-id (:queue::Waiter/conn-id w) :reply empty-ok))
                        taken)
                      (:wat::core::let
                        [envs (:wat::core::apply (:queue::queue::State/take s)
                                 (:queue::Waiter/queue w)
                                 [now
                                  (:queue::Waiter/visibility-ns w)
                                  (:queue::Waiter/limit w)])]
                        (:wat::core::if (:wat::core::empty? envs)
                          (:wat::core::Tuple (:wat::vector::conj keep w) box taken)
                          (:wat::core::Tuple keep
                            (:wat::core::conj box
                              (:wat::service::Directed
                                :conn-id (:queue::Waiter/conn-id w)
                                :reply (:queue::Queue::Reply::Receive
                                         (:queue::Queue::ReceiveResponse::Ok envs))))
                            (:wat::i64::+ taken (:wat::core::count envs))))))))
                (:wat::core::Tuple
                  (:wat::core::PersistentVector :- [:queue::Waiter])
                  (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                  0)
                (:queue::queue::State/waiters s))
        keep (:wat::core::first pair)
        box  (:wat::core::second pair)
        taken (:wat::core::third pair)
        p0 (:queue::queue::State/pending s)
        f0 (:queue::queue::State/in-flight s)
        p1 (:wat::core::if (:wat::i64::< p0 taken) 0 (:wat::i64::- p0 taken))
        f1 (:wat::i64::+ f0 taken)
        delay (:wat::core::foldl
                (:wat::core::fn [d <- :wat::core::i64  w <- :queue::Waiter] -> :wat::core::i64
                  (:wat::core::let [rem (:wat::core::- (:queue::Waiter/deadline-ns w) now)]
                    (:wat::core::if (:wat::i64::< rem d) rem d)))
                1000000000000000
                keep)
        ;; Process-tier timerfd treats it_value=0 as disarm, and a 1ns
        ;; expiry can be missed by edge-triggered poll. 1ms is the
        ;; smallest duration proven to fire at both loci.
        delay0 (:wat::core::if (:wat::i64::< delay 1000000) 1000000 delay)
        s' (:queue::queue::State
             :durable (:queue::queue::State/durable s)
             :store store
             :take (:queue::queue::State/take s)
             :waiters keep
             :outbox (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
             :receive-calls (:queue::queue::State/receive-calls s)
             :ticks ticks
             :pending p1
             :in-flight f1)]
       (:wat::service::SelfOutcome::Continue s' box
         (:wat::core::if (:wat::core::empty? keep)
           (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])
           [(:wat::service::Alarm :after (:wat::time::Nanosecond delay0) :op :-tick)]))))])

;; ── client helpers (the gate; Handle stays in the same let as the ops) ──────────
(:wat::core::defn :user::dial-queue
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :user::do-send
  [q <- :queue::Queue  name <- :wat::core::String  body <- :wat::core::String  now-ns <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:queue::Queue/send q (:queue::Queue::SendRequest :queue name :body body :now-ns now-ns))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "send not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "send: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::do-receive
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64]
  -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait-ns 0))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::do-receive-wait
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64  wait-ns <- :wat::core::i64]
  -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait-ns wait-ns))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "receive-wait not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive-wait: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::do-stats
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok calls ticks _pending _inflight)
          (:wat::core::Tuple calls ticks))
        (_ (:wat::kernel::assertion-failed! "stats not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "stats: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::do-depth
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks pending inflight)
          (:wat::core::Tuple pending inflight))
        (_ (:wat::kernel::assertion-failed! "depth not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "depth: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::do-ack
  [q <- :queue::Queue  name <- :wat::core::String  id <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::match (:queue::Queue/ack q (:queue::Queue::AckRequest :queue name :id id))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::AckResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "ack not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "ack: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::join-bodies
  [envs <- (:wat::core::Vector :- [:queue::Envelope])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String e <- :queue::Envelope] -> :wat::core::String
      (:wat::core::let [b (:queue::Envelope/body e)]
        (:wat::core::if (:wat::core::= acc "")
          b
          (:wat::string::concat acc (:wat::string::concat "," b)))))
    ""
    envs))

(:wat::core::defn :user::dial-queue-peer
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :user::send-ok!
  [st <- :wat::kernel::SendOutcome] -> :wat::core::nil
  (:wat::core::match st
    (:wat::kernel::SendOutcome::Sent nil)
    (_ (:wat::kernel::assertion-failed! "send-ok: not Sent" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::park-receive!
  [c <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64  wait-ns <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:user::send-ok!
         (:wat::kernel::send c
           (:queue::Queue::Op::Receive
             (:queue::Queue::ReceiveRequest
               :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait-ns wait-ns))))
     _st (:user::send-ok!
           (:wat::kernel::send c (:queue::Queue::Op::Stats (:queue::Queue::StatsRequest))))
     _   (:wat::core::match (:wat::kernel::recv c)
           ((:wat::kernel::RecvOutcome::Message recvd)
             (:wat::core::match recvd
               ((:queue::Queue::Reply::Stats _s) nil)
               (_ (:wat::kernel::assertion-failed! "park-receive: expected Stats reply as barrier" :wat::core::None :wat::core::None))))
           (_ (:wat::kernel::assertion-failed! "park-receive: stats barrier recv failed" :wat::core::None :wat::core::None)))]
    nil))

(:wat::core::defn :user::recv-envelopes!
  [c <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])] -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match (:wat::kernel::recv c)
    ((:wat::kernel::RecvOutcome::Message recvd)
      (:wat::core::match recvd
        ((:queue::Queue::Reply::Receive resp)
          (:wat::core::match resp
            ((:queue::Queue::ReceiveResponse::Ok envs) envs)
            (_ (:wat::kernel::assertion-failed! "recv-envelopes: not Ok" :wat::core::None :wat::core::None))))
        (_ (:wat::kernel::assertion-failed! "recv-envelopes: expected Reply::Receive" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (_ (:wat::kernel::assertion-failed! "recv-envelopes: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::nap-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

;; lifecycle against ONE store. Handle lives in this let (same-ns lesson).
(:wat::core::defn :user::lifecycle
  [store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::String
  (:wat::core::let
    [qh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record) :store-addr store-addr)
     q  (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     vis 100
     Ta  1000000001
     Tb  1000000002
     Tr  1000000002
     Tw  1000000102
     ;; STOP-3: a message whose isk equals now must be returned (inclusive hi).
     _bx (:user::do-send q "bound-q" "x" T0)
     bound (:user::do-receive q "bound-q" T0 vis 10)
     ;; send 3, staggered by 1ns so isk order is a,b,c (limit-2 among equal isk is unspecified).
     _sa (:user::do-send q "q" "a" T0)
     _sb (:user::do-send q "q" "b" Ta)
     _sc (:user::do-send q "q" "c" Tb)
     r1 (:user::do-receive q "q" Tr vis 2)
     r2 (:user::do-receive q "q" Tr vis 2)
     ;; ack one of the first receive (a) and the third (c), leaving b unacked.
     ;; c is acked so redelivery is exactly the unacked one — and so equal-isk
     ;; order between b and c cannot make the backends disagree on the summary.
     _a1 (:wat::core::if (:wat::core::empty? r1) nil
            (:user::do-ack q "q" (:queue::Envelope/id (:wat::core::first r1))))
     _a2 (:wat::core::if (:wat::core::empty? r2) nil
            (:user::do-ack q "q" (:queue::Envelope/id (:wat::core::first r2))))
     r3 (:user::do-receive q "q" Tr vis 10)
     re (:user::do-receive q "q" Tw vis 10)]
    (:wat::core::format
      "bound={bound};r1={r1};r2={r2};r3={r3};redel={redel}"
      :bound (:user::join-bodies bound)
      :r1 (:user::join-bodies r1)
      :r2 (:user::join-bodies r2)
      :r3 (:user::join-bodies r3)
      :redel (:user::join-bodies re))))

;; ★ row 1: send wakes a parked receive; the visibility re-put is applied.
(:wat::core::defn :user::lp-send-wakes [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record)
            :store-addr (:wat::query::mem-store::Handle/addr msh))
     a   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     vis 100
     _   (:user::park-receive! a "q" T0 vis 1 200000000)
     _   (:user::do-send b "q" "hello" T0)
     got (:user::recv-envelopes! a)
     again (:user::do-receive b "q" T0 vis 10)]
    (:wat::core::format "got={got};hidden={hidden}"
      :got (:user::join-bodies got)
      :hidden (:wat::core::if (:wat::core::empty? again) "yes" (:user::join-bodies again)))))

;; ★ row 2: parked receive times out empty; queue keeps serving.
(:wat::core::defn :user::lp-timeout [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record)
            :store-addr (:wat::query::mem-store::Handle/addr msh))
     a   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::park-receive! a "q" T0 100 1 5000000)
     got (:user::recv-envelopes! a)
     ping (:user::do-receive b "q" T0 100 10)]
    (:wat::core::format "empty={empty};serving={serving}"
      :empty (:wat::core::if (:wat::core::empty? got) "yes" "no")
      :serving (:wat::core::if (:wat::core::empty? ping) "yes" "no"))))

;; row 8: FIFO — first parked is served.
(:wat::core::defn :user::lp-fifo [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record)
            :store-addr (:wat::query::mem-store::Handle/addr msh))
     a   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     c   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::park-receive! a "q" T0 100 1 200000000)
     _   (:user::park-receive! c "q" T0 100 1 200000000)
     _   (:user::do-send b "q" "first" T0)
     ga  (:user::recv-envelopes! a)
     _   (:user::do-send b "q" "second" T0)
     gc  (:user::recv-envelopes! c)]
    (:wat::core::format "a={a};c={c}"
      :a (:user::join-bodies ga)
      :c (:user::join-bodies gc))))

;; row 5: a drain that would have spun N times makes far fewer receive calls.
(:wat::core::defn :user::lp-fewer-receives [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record)
            :store-addr (:wat::query::mem-store::Handle/addr msh))
     q   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::do-send q "q" "a" T0)
     _   (:user::do-send q "q" "b" T0)
     _   (:user::do-send q "q" "c" T0)
     got (:user::do-receive-wait q "q" T0 1000000000 10 20000000)
     _   (:user::do-receive-wait q "q" T0 1000000000 10 5000000)
     st  (:user::do-stats q)]
    (:wat::core::format "n={n};calls={calls}"
      :n (:wat::core::count got)
      :calls (:wat::core::first st))))

;; row 6: idle queue never ticks.
(:wat::core::defn :user::lp-idle [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record)
            :store-addr (:wat::query::mem-store::Handle/addr msh))
     q   (:user::dial-queue (:queue::queue::Handle/addr qh))
     _   (:user::nap-ms 20)
     st  (:user::do-stats q)]
    (:wat::core::format "ticks={ticks}"
      :ticks (:wat::core::second st))))

;; depth counters: send increments pending; receive moves pending → in-flight; ack decrements in-flight.
(:wat::core::defn :user::depth [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record)
            :store-addr (:wat::query::mem-store::Handle/addr msh))
     q   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     vis 1000000000
     _   (:user::do-send q "q" "a" T0)
     _   (:user::do-send q "q" "b" T0)
     _   (:user::do-send q "q" "c" T0)
     d0  (:user::do-depth q)
     r   (:user::do-receive q "q" T0 vis 2)
     d1  (:user::do-depth q)
     _   (:user::do-ack q "q" (:queue::Envelope/id (:wat::core::first r)))
     d2  (:user::do-depth q)]
    (:wat::core::format
      "send=p={p0},f={f0};recv=p={p1},f={f1};ack=p={p2},f={f2}"
      :p0 (:wat::core::first d0) :f0 (:wat::core::second d0)
      :p1 (:wat::core::first d1) :f1 (:wat::core::second d1)
      :p2 (:wat::core::first d2) :f2 (:wat::core::second d2))))

(:wat::core::defn :user::long-poll [] -> :wat::core::String
  (:wat::core::format
    "wakes={wakes};timeout={timeout};fifo={fifo};fewer={fewer};idle={idle}"
    :wakes (:user::lp-send-wakes)
    :timeout (:user::lp-timeout)
    :fifo (:user::lp-fifo)
    :fewer (:user::lp-fewer-receives)
    :idle (:user::lp-idle)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     ssh   (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::sqlite-store::Record
                       :path ":memory:"
                       :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     saddr (:wat::query::sqlite-store::Handle/addr ssh)
     mem   (:user::lifecycle maddr)
     sql   (:user::lifecycle saddr)]
    (:wat::core::if (:wat::core::= mem sql)
      mem
      (:wat::core::format "DIFFERENTIAL-MISMATCH mem={mem} sqlite={sql}" :mem mem :sql sql))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
