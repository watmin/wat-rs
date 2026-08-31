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
(:wat::core::defrecord :queue::Envelope
  [id   <- :wat::core::String
   body <- :wat::core::String])

(:wat::core::defsurface :queue::Queue :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :queue::Queue::SendRequest
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
      limit         <- :wat::core::i64])
   (:wat::core::defenum :queue::Queue::ReceiveResponse :wat::enum::Pure
     :Ok [envelopes <- (:wat::core::Vector :- [:queue::Envelope])]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])

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
     -> :queue::Queue::AckResponse :max-request-bytes 524288)])

;; ── service (holds a Store peer; init declares the GSI) ─────────────────────────
(:wat::service::defservice :queue::queue
  :satisfies :queue::Queue
  :durable   []
  :ephemeral [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
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
                                    :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))]
            (:queue::queue::State :durable record :store store)))
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
               (:wat::service::Outcome::Reply s (:queue::Queue::SendResponse::Ok)))
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
        lo     (:wat::edn::write (:wat::time::at-nanos 0))
        hi     (:wat::edn::write (:wat::time::at-nanos now-ns))
        scan  (:wat::query::Store/scan-index store
                (:wat::query::Store::ScanIndexRequest
                  :index "by-visible-at" :ipk q :isk-lo lo :isk-hi hi :limit lim :cursor :wat::core::None))]
       (:wat::core::match scan
         ((:wat::kernel::RecvOutcome::Message sresp)
           (:wat::core::match sresp
             ((:wat::query::Store::ScanIndexResponse::Success irows _c)
               (:wat::core::if (:wat::core::empty? irows)
                 (:wat::service::Outcome::Reply s
                   (:queue::Queue::ReceiveResponse::Ok
                     (:wat::core::Vector :- [:queue::Envelope])))
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
                         ((:wat::query::Store::PutResponse::Success)
                           (:wat::service::Outcome::Reply s (:queue::Queue::ReceiveResponse::Ok envs)))
                         (_ (:wat::kernel::assertion-failed! "queue.receive: re-put failed" :wat::core::None :wat::core::None))))
                     ((:wat::kernel::RecvOutcome::Lost cause)
                       (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Stopped
                       (:wat::kernel::assertion-failed! "queue.receive: stop requested mid re-put — the store peer was ALIVE" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Closed
                       (:wat::kernel::assertion-failed! "queue.receive: store peer closed" :wat::core::None :wat::core::None))))))
             (_ (:wat::kernel::assertion-failed! "queue.receive: scan-index failed" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost cause)
           (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "queue.receive: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "queue.receive: store peer closed" :wat::core::None :wat::core::None)))))

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
               (:wat::service::Outcome::Reply s (:queue::Queue::AckResponse::Ok)))
             (_ (:wat::kernel::assertion-failed! "queue.ack: store delete failed" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost cause)
           (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "queue.ack: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::assertion-failed! "queue.ack: store peer closed" :wat::core::None :wat::core::None)))))])

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
      (:queue::Queue::ReceiveRequest :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive: recv failed" :wat::core::None :wat::core::None))))

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
