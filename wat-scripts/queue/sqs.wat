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
;;   `(:wat::time::epoch-nanos (:wat::time::now))`. Time types cross a service
;;   boundary now (Stone B-pre). `now-ns` / `visibility-ns` stay i64 so a fixture
;;   can drive the clock as a value. `wait` is the exception: it is a mode
;;   (`:Immediate` / `:UpTo [NonZeroDuration]`), not a magnitude.
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
     [queue  <- :wat::core::String
      bodies <- (:wat::core::Vector :- [:wat::core::String])
      now-ns <- :wat::core::i64])
   (:wat::core::defenum :queue::Queue::SendResponse :wat::enum::Pure
     :Accepted [count <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestTooManyEntries [entries <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])

   ;; The wait names its verb. Zero is not a short wait — it is a different
   ;; operation (sweep, do not park). The mode is the constructor, never a
   ;; comparison against a magnitude.
   (:wat::core::defenum :queue::Queue::Wait :wat::enum::Pure
     :Immediate []
     :UpTo [d <- :wat::time::NonZeroDuration])

   (:wat::core::defrecord :queue::Queue::ReceiveRequest
     [queue         <- :wat::core::String
      now-ns        <- :wat::core::i64
      visibility-ns <- :wat::core::i64
      limit         <- :wat::core::i64
      wait          <- :queue::Queue::Wait])
   (:wat::core::defrecord :queue::Queue::StatsRequest [])
   ;; visible / unacked are derived from by-visible-at at the moment of the
   ;; question (isk <= now vs isk > now). StatsRequest carries no clock and no
   ;; queue name: the arm uses Invocation/start-ns and the name remembered
   ;; from send/receive (one name per service instance).
   (:wat::core::defenum :queue::Queue::StatsResponse :wat::enum::Pure
     :Ok [stats <- :queue::Stats]
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
   ;; Fold accumulator. defstruct: holds a live Store Peer. box stays
   ;; outside as slot two of (Tuple TakeAcc box) — naming Queue::Reply
   ;; here would trip S4c. A fifth field is a named slot, not a paren.
   ;; `calls`/`ns` stay the aggregate; the four op-scoped pairs beside them are
   ;; the split's transport out of the fold (arc 278, the store reports time per
   ;; operation). Both are accumulated independently so the split can be
   ;; RECONCILED against the aggregate rather than defined as it.
   (:wat::core::defstruct :queue::TakeAcc
     [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
      keep  <- (:wat::core::PersistentVector :- [:queue::Waiter])
      calls <- :wat::core::i64
      ns    <- :wat::core::i64
      scan-calls <- :wat::core::i64
      scan-ns    <- :wat::core::i64
      put-calls  <- :wat::core::i64
      put-ns     <- :wat::core::i64])
   ;; Retry helper result. Script-level Tuple is unknown in the process
   ;; child; a :messages struct is not. n is the extra-call count.
   (:wat::core::defstruct :queue::RetryAcc
     [n  <- :wat::core::i64
      ns <- :wat::core::i64])
   ;; ── the COLD counters, riding one carrier ──────────────────────────────────
   ;; `:queue::queue::State` is rebuilt at 30 sites. These six are CHANGED at 1–2 of
   ;; them (ticks 2, acks 2, the other four 1 each) and passed through at the other
   ;; 28–29, so a carrier lets ≥22 sites copy them by reference instead of copying six
   ;; fields by hand. ⛔ This is NOT a home for every counter: `handler-ns` is changed
   ;; at 27 of 30, so a carrier would make nearly every rebuild allocate MORE
   ;; (24-field State + 6-field carrier > 29-field State). The win is a function of
   ;; change FREQUENCY, not of tidiness — see DESIGN.md of
   ;; docs/excursus/2026/08/001-sns-sqs/the-cold-counters-ride-one-carrier.
   ;; Declared in :messages for the same reason TakeAcc is: a script-level type is
   ;; unknown in the process child, a surface type is not.
   ;; ⛔ `recv-drops` / `ack-drops` COUNT THE INJECTOR'S OWN FIRES, and they exist because
   ;; the builder asked "what is our induced failure rate per component?" and the tree could
   ;; not answer. The drop DECISION (`hit?`, :833 and :1022) was computed and then discarded,
   ;; so the only evidence a fault had fired was DOWNSTREAM — `ack-retries`, `redeliveries`,
   ;; `seen-skipped` — which is inference, not measurement. A rate you set and cannot observe
   ;; is a rate you are guessing about.
   ;;
   ;; ⭑ The denominators are already here: `receive-calls` for recv-drops, and `acks` counts
   ;; IDS not calls, so ack-drops' denominator is NOT `acks` — see the report line's
   ;; `ack-calls`. Counting the numerator without its denominator would have reproduced the
   ;; exact counter-unit error this stone's own measurement was meant to avoid.
   (:wat::core::defrecord :queue::Counters
     [ticks <- :wat::core::i64
      acks  <- :wat::core::i64
      sends-accepted <- :wat::core::i64
      sends-refused  <- :wat::core::i64
      redeliveries   <- :wat::core::i64
      expired-waiters <- :wat::core::i64
      recv-drops <- :wat::core::i64
      ack-drops  <- :wat::core::i64
      ack-calls  <- :wat::core::i64
      ;; ⛔ THE DENOMINATOR IS NOT `receive-calls`. `receive` has THREE exit paths and only
      ;; two of them reach a reply the injector can suppress: the take path and the
      ;; immediate-empty path. The third PARKS a waiter, so a draw taken there is discarded
      ;; — the RNG advances and no reply exists to drop. Dividing fires by `receive-calls`
      ;; therefore understates the rate by exactly the park traffic, which for the inbox is
      ;; most of it: the first version of this counter read 1.87% where 5% was set, and the
      ;; inbox was the only tier that showed it because it is the one that parks.
      recv-replies <- :wat::core::i64])
   ;; Named aggregate for stats. EDN-expressible i64s only — same shape as
   ;; TakeAcc/RetryAcc: a record, not a 12-wide positional variant.
   ;; ⛔ Stats stays FLAT and 19 fields wide, deliberately: `:queue::Stats/<field>` is
   ;; read 32 times across 8 files, and Stats is constructed at exactly ONE site
   ;; (the `stats` impl below), once per call — so its width never touches `drain`.
   ;; Nesting `counters` here would cost a 32-site corpus migration for zero gain.
   (:wat::core::defrecord :queue::Stats
     [receive-calls <- :wat::core::i64  ticks <- :wat::core::i64
      visible <- :wat::core::i64  unacked <- :wat::core::i64
      store-calls <- :wat::core::i64  store-ns <- :wat::core::i64  handler-ns <- :wat::core::i64
      ;; The aggregate above, split by store operation. `ensure-schema` is called
      ;; once in :init, BEFORE store-calls/store-ns exist, so it is in neither.
      put-calls <- :wat::core::i64     put-ns <- :wat::core::i64
      delete-calls <- :wat::core::i64  delete-ns <- :wat::core::i64
      count-calls <- :wat::core::i64   count-ns <- :wat::core::i64
      scan-calls <- :wat::core::i64    scan-ns <- :wat::core::i64
      sends-accepted <- :wat::core::i64  sends-refused <- :wat::core::i64  acks <- :wat::core::i64
      redeliveries <- :wat::core::i64  expired-waiters <- :wat::core::i64
      ;; The injector's own fires, with the denominator that belongs to each:
      ;; recv-drops / receive-calls, and ack-drops / ack-calls. NOT ack-drops/acks —
      ;; `acks` counts IDS, ack-calls counts CALLS, and the drop is per call.
      recv-drops <- :wat::core::i64  ack-drops <- :wat::core::i64  ack-calls <- :wat::core::i64
      ;; NOT `receive-calls` — see the note on :queue::Counters/recv-replies.
      recv-replies <- :wat::core::i64])

   (:wat::core::defrecord :queue::Queue::AckRequest
     [queue <- :wat::core::String
      ids   <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :queue::Queue::AckResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(send    [self <- :queue::Queue  req <- :queue::Queue::SendRequest]
     -> :queue::Queue::SendResponse :max-request-bytes 524288 :max-entries [bodies 64])
   (receive [self <- :queue::Queue  req <- :queue::Queue::ReceiveRequest]
     -> :queue::Queue::ReceiveResponse :max-request-bytes 524288 :max-page [envelopes 64])
   (ack     [self <- :queue::Queue  req <- :queue::Queue::AckRequest]
     -> :queue::Queue::AckResponse :max-request-bytes 524288)
   (stats   [self <- :queue::Queue  req <- :queue::Queue::StatsRequest]
     -> :queue::Queue::StatsResponse :max-request-bytes 524288)])

;; ── service (holds a Store peer; init declares the GSI) ─────────────────────────
(:wat::service::defservice :queue::queue
  :satisfies :queue::Queue
  ;; 8192: a 10KB disrupt send severs the sender. Normal circuit envelopes fit.
  ;; Contract cap stays 524288. Thread locus does not tear.
  :max-frame-bytes 8192
  :durable   [cap <- :wat::core::i64
              store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])
              drop-recv-bp <- :wat::core::i64
              drop-ack-bp  <- :wat::core::i64
              drop-seed    <- :wat::core::i64]
  :ephemeral [store         <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
              ;; Third slot is (scan-ns put-ns) — the two ops `take` performs, kept
              ;; APART. There is no fourth Tuple accessor (wat/core.wat:1737), so the
              ;; pair nests rather than widening the tuple.
              take          <- [(:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply]) :wat::core::String :wat::core::i64 :wat::core::i64 :wat::core::i64 :-> (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply]) (:wat::core::Vector :- [:queue::Envelope]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
              waiters       <- (:wat::core::PersistentVector :- [:queue::Waiter])
              outbox        <- (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
              receive-calls <- :wat::core::i64
              store-calls   <- :wat::core::i64
              store-ns      <- :wat::core::i64
              ;; ── the aggregate above, split by store operation ──────────────────
              ;; Attributed AT THE CALL SITE, never inferred from a response type.
              ;; Accumulated independently of store-calls/store-ns so the two can be
              ;; reconciled. `ensure-schema` is :init-only and in neither.
              put-calls     <- :wat::core::i64
              put-ns        <- :wat::core::i64
              delete-calls  <- :wat::core::i64
              delete-ns     <- :wat::core::i64
              count-calls   <- :wat::core::i64
              count-ns      <- :wat::core::i64
              scan-calls    <- :wat::core::i64
              scan-ns       <- :wat::core::i64
              handler-ns    <- :wat::core::i64
              depth         <- [(:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply]) :wat::core::String :wat::core::i64 :wat::core::i64 :-> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])]
              total         <- [(:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply]) :wat::core::String :wat::core::i64 :wat::core::i64 :-> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])]
              q-name        <- :wat::core::String
              tick-armed?   <- :wat::core::bool
              arm-tick      <- [:wat::core::bool :wat::core::i64 :wat::core::i64 :-> (:wat::core::Tuple :- [:wat::core::bool (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])])]
              ;; ── the six COLD counters, one carrier ─────────────────────────────
              ;; ticks · acks · sends-accepted · sends-refused · redeliveries ·
              ;; expired-waiters. Changed at 1–2 of the 30 rebuild sites each; the
              ;; other 22+ copy this ONE field instead of six. `handler-ns` above
              ;; stays FLAT on purpose — 27 of 30 sites change it, so a carrier there
              ;; makes nearly every rebuild allocate more, not less.
              counters       <- :queue::Counters
              seen-ids       <- (:wat::core::PersistentSet :- [:wat::core::String])]
  :peers     [:wat::query::Store]
  :init (:wat::core::fn
          [record     <- :queue::queue::Record]
          -> :queue::queue::State
          (:wat::core::let
            [store-addr (:queue::queue::Record/store-addr record)
             dial-store (:wat::core::fn [] -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
                           (:wat::core::match (:wat::kernel::connect store-addr)
                             ((:wat::kernel::ConnectOutcome::Connected p) p)
                             (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))))
             store (:wat::core::match (:wat::kernel::connect store-addr)
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
             empty-envs (:wat::core::Vector :- [:queue::Envelope])
             take (:wat::core::fn
                    [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
                     q <- :wat::core::String  now-ns <- :wat::core::i64
                     vis-ns <- :wat::core::i64  lim <- :wat::core::i64]
                    -> (:wat::core::Tuple :- [(:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
                                              (:wat::core::Vector :- [:queue::Envelope])
                                              (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
                    (:wat::core::let
                      [lo (:wat::edn::write (:wat::time::at-nanos 0))
                       hi (:wat::edn::write (:wat::time::at-nanos now-ns))
                       t-scan (:wat::time::epoch-nanos (:wat::time::now))
                       scan (:wat::query::Store/scan-index st
                               (:wat::query::Store::ScanIndexRequest
                                 :index "by-visible-at" :ipk q :isk-lo lo :isk-hi hi :limit lim :cursor :wat::core::None))
                       scan-ns (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t-scan)
                       ;; scan happened, no put yet: the put slot is 0, not a share.
                       scan-only (:wat::core::Tuple scan-ns 0)]
                      (:wat::core::match scan
                        ((:wat::kernel::RecvOutcome::Message sresp)
                          (:wat::core::match sresp
                            ((:wat::query::Store::ScanIndexResponse::Success irows _c)
                              (:wat::core::if (:wat::core::empty? irows)
                                (:wat::core::Tuple st empty-envs scan-only)
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
                                   t-put (:wat::time::epoch-nanos (:wat::time::now))
                                   put-resp (:wat::query::Store/put st
                                              (:wat::query::Store::PutRequest put-rows))
                                   put-ns (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t-put)
                                   ;; SAME elapsed values as before, no longer summed:
                                   ;; scan and the re-put are two operations.
                                   both (:wat::core::Tuple scan-ns put-ns)]
                                  (:wat::core::match put-resp
                                    ((:wat::kernel::RecvOutcome::Message presp)
                                      (:wat::core::match presp
                                        ((:wat::query::Store::PutResponse::Success) (:wat::core::Tuple st envs both))
                                        ((:wat::query::Store::PutResponse::Transient _e)
                                          (:wat::kernel::assertion-failed! "queue.take: re-put Transient" :wat::core::None :wat::core::None))
                                        ((:wat::query::Store::PutResponse::Constraint _e)
                                          (:wat::kernel::assertion-failed! "queue.take: re-put Constraint" :wat::core::None :wat::core::None))
                                        ((:wat::query::Store::PutResponse::Fatal _e)
                                          (:wat::kernel::assertion-failed! "queue.take: re-put Fatal" :wat::core::None :wat::core::None))
                                        ((:wat::query::Store::PutResponse::RequestTooLarge _b _c)
                                          (:wat::kernel::assertion-failed! "queue.take: re-put RequestTooLarge" :wat::core::None :wat::core::None))
                                        ((:wat::query::Store::PutResponse::RequestMalformed _p _e _g)
                                          (:wat::kernel::assertion-failed! "queue.take: re-put RequestMalformed" :wat::core::None :wat::core::None))))
                                    ((:wat::kernel::RecvOutcome::Lost _cause)
                                      (:wat::core::Tuple (dial-store) empty-envs both))
                                    (:wat::kernel::RecvOutcome::Stopped
                                      (:wat::kernel::assertion-failed! "queue.take: stop requested mid re-put" :wat::core::None :wat::core::None))
                                    (:wat::kernel::RecvOutcome::Closed
                                      (:wat::core::Tuple (dial-store) empty-envs both))
                                    (:wat::kernel::RecvOutcome::TimedOut
                                      (:wat::core::Tuple (dial-store) empty-envs both)) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None))))))
                            ((:wat::query::Store::ScanIndexResponse::Transient _e)
                              (:wat::kernel::assertion-failed! "queue.take: scan-index Transient" :wat::core::None :wat::core::None))
                            ((:wat::query::Store::ScanIndexResponse::Fatal _e)
                              (:wat::kernel::assertion-failed! "queue.take: scan-index Fatal" :wat::core::None :wat::core::None))
                            ((:wat::query::Store::ScanIndexResponse::RequestTooLarge _b _c)
                              (:wat::kernel::assertion-failed! "queue.take: scan-index RequestTooLarge" :wat::core::None :wat::core::None))
                            ((:wat::query::Store::ScanIndexResponse::RequestMalformed _p _e _g)
                              (:wat::kernel::assertion-failed! "queue.take: scan-index RequestMalformed" :wat::core::None :wat::core::None))))
                        ((:wat::kernel::RecvOutcome::Lost _cause)
                          (:wat::core::Tuple (dial-store) empty-envs scan-only))
                        (:wat::kernel::RecvOutcome::Stopped
                          (:wat::kernel::assertion-failed! "queue.take: stop requested" :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Closed
                          (:wat::core::Tuple (dial-store) empty-envs scan-only))
                        (:wat::kernel::RecvOutcome::TimedOut
                          (:wat::core::Tuple (dial-store) empty-envs scan-only)) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))))
             ;; Closed over nothing extra. Process children do not see sibling
             ;; defns, so the body lives here, called via State/depth.
             ;; (visible unacked): |isk in [0, now]| and |isk in [0, +inf)| minus vis.
             ;; lim is cap+1 at every call site — overflow is visible, never truncated.
             depth (:wat::core::fn
                    [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
                     q <- :wat::core::String  now-ns <- :wat::core::i64  lim <- :wat::core::i64]
                    -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
                    (:wat::core::let
                      [lo (:wat::edn::write (:wat::time::at-nanos 0))
                       inf-ns 4000000000000000000
                       count-hi (:wat::core::fn
                                  [st2 <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
                                   hi-ns <- :wat::core::i64]
                                  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                                  (:wat::core::let
                                    [t0 (:wat::time::epoch-nanos (:wat::time::now))
                                     resp (:wat::query::Store/count-index st2
                                             (:wat::query::Store::CountIndexRequest
                                               :index "by-visible-at" :ipk q
                                               :isk-lo lo
                                               :isk-hi (:wat::edn::write (:wat::time::at-nanos hi-ns))
                                               :limit lim))
                                     elapsed (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0)]
                                    (:wat::core::match resp
                                      ((:wat::kernel::RecvOutcome::Message sresp)
                                        (:wat::core::match sresp
                                          ((:wat::query::Store::CountIndexResponse::Ok n) (:wat::core::Tuple n elapsed))
                                          ((:wat::query::Store::CountIndexResponse::Transient _e)
                                            (:wat::kernel::assertion-failed! "queue.depth: count-index Transient" :wat::core::None :wat::core::None))
                                          ((:wat::query::Store::CountIndexResponse::Fatal _e)
                                            (:wat::kernel::assertion-failed! "queue.depth: count-index Fatal" :wat::core::None :wat::core::None))
                                          ((:wat::query::Store::CountIndexResponse::RequestTooLarge _b _c)
                                            (:wat::kernel::assertion-failed! "queue.depth: count-index RequestTooLarge" :wat::core::None :wat::core::None))
                                          ((:wat::query::Store::CountIndexResponse::RequestMalformed _p _e _g)
                                            (:wat::kernel::assertion-failed! "queue.depth: count-index RequestMalformed" :wat::core::None :wat::core::None))))
                                      ((:wat::kernel::RecvOutcome::Lost _cause)
                                        (:wat::kernel::assertion-failed! "queue.depth: store lost" :wat::core::None :wat::core::None))
                                      (:wat::kernel::RecvOutcome::Stopped
                                        (:wat::kernel::assertion-failed! "queue.depth: stop requested" :wat::core::None :wat::core::None))
                                      (:wat::kernel::RecvOutcome::Closed
                                        (:wat::kernel::assertion-failed! "queue.depth: store closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))))
                       vis-pair (:wat::core::apply count-hi st [now-ns])
                       all-pair (:wat::core::apply count-hi st [inf-ns])]
                      (:wat::core::Tuple
                        (:wat::core::first vis-pair)
                        (:wat::i64::- (:wat::core::first all-pair) (:wat::core::first vis-pair))
                        (:wat::i64::+ (:wat::core::second vis-pair) (:wat::core::second all-pair)))))
             ;; One scan over [0, +inf). visible+unacked IS total; the send path
             ;; wants that integer and was buying a split. now-ns is unused; same
             ;; arity as depth so apply sites swap. lim is cap+1 — overflow visible.
             total (:wat::core::fn
                     [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
                      q <- :wat::core::String  _now-ns <- :wat::core::i64  lim <- :wat::core::i64]
                     -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                     (:wat::core::let
                       [t0 (:wat::time::epoch-nanos (:wat::time::now))
                        resp (:wat::query::Store/count-index st
                                (:wat::query::Store::CountIndexRequest
                                  :index "by-visible-at" :ipk q
                                  :isk-lo (:wat::edn::write (:wat::time::at-nanos 0))
                                  :isk-hi (:wat::edn::write (:wat::time::at-nanos 4000000000000000000))
                                  :limit lim))
                        elapsed (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0)]
                       (:wat::core::match resp
                         ((:wat::kernel::RecvOutcome::Message sresp)
                           (:wat::core::match sresp
                             ((:wat::query::Store::CountIndexResponse::Ok n) (:wat::core::Tuple n elapsed))
                             ((:wat::query::Store::CountIndexResponse::Transient _e)
                               (:wat::kernel::assertion-failed! "queue.total: count-index Transient" :wat::core::None :wat::core::None))
                             ((:wat::query::Store::CountIndexResponse::Fatal _e)
                               (:wat::kernel::assertion-failed! "queue.total: count-index Fatal" :wat::core::None :wat::core::None))
                             ((:wat::query::Store::CountIndexResponse::RequestTooLarge _b _c)
                               (:wat::kernel::assertion-failed! "queue.total: count-index RequestTooLarge" :wat::core::None :wat::core::None))
                             ((:wat::query::Store::CountIndexResponse::RequestMalformed _p _e _g)
                               (:wat::kernel::assertion-failed! "queue.total: count-index RequestMalformed" :wat::core::None :wat::core::None))))
                         ((:wat::kernel::RecvOutcome::Lost _cause)
                           (:wat::kernel::assertion-failed! "queue.total: store lost" :wat::core::None :wat::core::None))
                         (:wat::kernel::RecvOutcome::Stopped
                           (:wat::kernel::assertion-failed! "queue.total: stop requested" :wat::core::None :wat::core::None))
                         (:wat::kernel::RecvOutcome::Closed
                           (:wat::kernel::assertion-failed! "queue.total: store closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))))
             ;; Closed over nothing. Process children do not see sibling defns,
             ;; so the body lives here. ONE place decides whether to arm.
             none (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])
             arm-tick (:wat::core::fn
                        [armed? <- :wat::core::bool
                         n      <- :wat::core::i64
                         delay0 <- :wat::core::i64]
                        -> (:wat::core::Tuple :- [:wat::core::bool
                             (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])])
                        (:wat::core::if (:wat::i64::<= n 0)
                          (:wat::core::Tuple false none)
                          (:wat::core::if armed?
                            (:wat::core::Tuple true none)
                            (:wat::core::Tuple true
                              [(:wat::service::Alarm :delay (:wat::time::Nanoseconds delay0)
                                 :op (:queue::queue::Op::-Tick))]))))]
            (:queue::queue::State
              :durable record
              :store store
              :take take
              :waiters (:wat::core::PersistentVector :- [:queue::Waiter])
              :outbox (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
              :receive-calls 0
              :store-calls 0
              :store-ns 0
              :put-calls 0
              :put-ns 0
              :delete-calls 0
              :delete-ns 0
              :count-calls 0
              :count-ns 0
              :scan-calls 0
              :scan-ns 0
              :handler-ns 0
              :depth depth
              :total total
              :q-name ""
              :tick-armed? false
              :arm-tick arm-tick
              :counters (:queue::Counters :ticks 0 :acks 0 :sends-accepted 0
                          :sends-refused 0 :redeliveries 0 :expired-waiters 0
                          :recv-drops 0 :ack-drops 0 :ack-calls 0 :recv-replies 0)
              :seen-ids (:wat::core::PersistentSet :- [:wat::core::String]))))
  :impls
  [(send [s ctx req]
     (:wat::core::let
       [start-ns (:wat::service::Invocation/start-ns ctx)
        store (:queue::queue::State/store s)
        q      (:queue::Queue::SendRequest/queue req)
        now-ns (:queue::Queue::SendRequest/now-ns req)
        bodies (:queue::Queue::SendRequest/bodies req)
        n0     (:wat::core::count bodies)
        cap    (:queue::queue::Record/cap (:queue::queue::State/durable s))
        lim    (:wat::i64::+ cap 1)
        tot-pair (:wat::core::apply (:queue::queue::State/total s) store q [now-ns lim])
        depth  (:wat::core::first tot-pair)
        total-ns (:wat::core::second tot-pair)
        sc0    (:wat::i64::+ (:queue::queue::State/store-calls s) 1)
        sn0    (:wat::i64::+ (:queue::queue::State/store-ns s) total-ns)
        ;; `total` is ONE count-index (:wat::query::Store/count-index, the `total`
        ;; closure). sc0/sn0 already bank it in the aggregate; cc0/cn0 bank the
        ;; SAME call and the SAME elapsed value under its own operation.
        cc0    (:wat::i64::+ (:queue::queue::State/count-calls s) 1)
        cn0    (:wat::i64::+ (:queue::queue::State/count-ns s) total-ns)
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])
        sends  (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
        room   (:wat::i64::- cap depth)
        room   (:wat::core::if (:wat::i64::< room 0) 0 room)
        take   (:wat::core::if (:wat::i64::> n0 cap)
                 room
                 (:wat::core::if (:wat::i64::<= n0 room) n0 0))]
       (:wat::core::if (:wat::core::= take 0)
         (:wat::core::let
           [cold (:queue::queue::State/counters s)
            s0 (:queue::queue::State
                 :durable (:queue::queue::State/durable s)
                 :store store
                 :take (:queue::queue::State/take s)
                 :waiters (:queue::queue::State/waiters s)
                 :outbox (:queue::queue::State/outbox s)
                 :receive-calls (:queue::queue::State/receive-calls s)
                 :store-calls sc0 :store-ns sn0
                 :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
                 :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
                 :count-calls cc0 :count-ns cn0
                 :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                 :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                 :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                 :q-name q
                 :tick-armed? (:queue::queue::State/tick-armed? s)
                 :arm-tick (:queue::queue::State/arm-tick s)
                 ;; One of the ≤8 sites that CHANGES a cold counter: the carrier is
                 ;; rebuilt here (24 + 6 = 30 fields) so the other 22+ sites can copy
                 ;; it by reference (24 fields). That asymmetry IS the win.
                 :counters (:queue::Counters
                             :ticks (:queue::Counters/ticks cold)
                             :acks (:queue::Counters/acks cold)
                             :sends-accepted (:queue::Counters/sends-accepted cold)
                             :sends-refused (:wat::i64::+ (:queue::Counters/sends-refused cold) 1)
                             :redeliveries (:queue::Counters/redeliveries cold)
                             :expired-waiters (:queue::Counters/expired-waiters cold)
                             :recv-drops (:queue::Counters/recv-drops cold)
                             :ack-drops (:queue::Counters/ack-drops cold)
                             :ack-calls (:queue::Counters/ack-calls cold) :recv-replies (:queue::Counters/recv-replies cold))
                 :seen-ids (:queue::queue::State/seen-ids s))]
           (:wat::service::Outcome::Continue s0
             (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Accepted 0)))
             sends
             none-alarms))
         (:wat::core::let
           [rows   (:wat::core::foldl
                 (:wat::core::fn
                   [acc <- (:wat::core::Vector :- [:wat::query::StoredRow])
                    i   <- :wat::core::i64]
                   -> (:wat::core::Vector :- [:wat::query::StoredRow])
                   (:wat::core::let
                     [body (:wat::core::nth bodies i)
                      sk   (:wat::edn::write (:wat::uuid::v4))
                      ;; 1ns stagger: equal isk makes scan-index :limit unspecified
                      ;; (sqs.wat header). A batch of 10 sharing now-ns would hide.
                      isk  (:wat::edn::write (:wat::time::at-nanos (:wat::i64::+ now-ns i)))]
                     (:wat::core::conj acc
                       (:wat::query::StoredRow
                         :pk q :sk sk :data body
                         :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                                       "by-visible-at" (:wat::query::IndexKey :ipk q :isk isk))))))
                 (:wat::core::Vector :- [:wat::query::StoredRow])
                 (:wat::core::range 0 take))
        t-put (:wat::time::epoch-nanos (:wat::time::now))
        put-resp (:wat::query::Store/put store
                   (:wat::query::Store::PutRequest rows))
        put-ns (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t-put)]
       (:wat::core::match put-resp
         ((:wat::kernel::RecvOutcome::Message sresp)
           (:wat::core::match sresp
             ((:wat::query::Store::PutResponse::Success)
               (:wat::core::let
                 [cold (:queue::queue::State/counters s)
                  s' (:queue::queue::State
                       :durable (:queue::queue::State/durable s)
                       :store store
                       :take (:queue::queue::State/take s)
                       :waiters (:queue::queue::State/waiters s)
                       :outbox (:queue::queue::State/outbox s)
                       :receive-calls (:queue::queue::State/receive-calls s)
                       :store-calls (:wat::i64::+ sc0 1) :store-ns (:wat::i64::+ sn0 put-ns)
                       :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s) 1) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s) put-ns)
                       :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
                       :count-calls cc0 :count-ns cn0
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                       :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                       :q-name q
                       :tick-armed? (:queue::queue::State/tick-armed? s)
                       :arm-tick (:queue::queue::State/arm-tick s)
                       ;; Changes a cold counter → the carrier is rebuilt here.
                       :counters (:queue::Counters
                                   :ticks (:queue::Counters/ticks cold)
                                   :acks (:queue::Counters/acks cold)
                                   :sends-accepted (:wat::i64::+ (:queue::Counters/sends-accepted cold) take)
                                   :sends-refused (:queue::Counters/sends-refused cold)
                                   :redeliveries (:queue::Counters/redeliveries cold)
                                   :expired-waiters (:queue::Counters/expired-waiters cold)
                                   :recv-drops (:queue::Counters/recv-drops cold)
                                   :ack-drops (:queue::Counters/ack-drops cold)
                                   :ack-calls (:queue::Counters/ack-calls cold) :recv-replies (:queue::Counters/recv-replies cold))
                       :seen-ids (:queue::queue::State/seen-ids s))]
                 (:wat::core::if (:wat::core::empty? (:queue::queue::State/waiters s'))
                   (:wat::core::let
                     [pair (:wat::core::apply (:queue::queue::State/arm-tick s')
                              (:queue::queue::State/tick-armed? s')
                              [(:wat::core::count (:queue::queue::State/waiters s')) 1000000])
                      s2 (:queue::queue::State
                           :durable (:queue::queue::State/durable s')
                           :store store
                           :take (:queue::queue::State/take s')
                           :waiters (:queue::queue::State/waiters s')
                           :outbox (:queue::queue::State/outbox s')
                           :receive-calls (:queue::queue::State/receive-calls s') :store-calls (:queue::queue::State/store-calls s') :store-ns (:queue::queue::State/store-ns s')
              :put-calls (:queue::queue::State/put-calls s') :put-ns (:queue::queue::State/put-ns s')
              :delete-calls (:queue::queue::State/delete-calls s') :delete-ns (:queue::queue::State/delete-ns s')
              :count-calls (:queue::queue::State/count-calls s') :count-ns (:queue::queue::State/count-ns s')
              :scan-calls (:queue::queue::State/scan-calls s') :scan-ns (:queue::queue::State/scan-ns s')
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                           :depth (:queue::queue::State/depth s') :total (:queue::queue::State/total s')
                           :q-name (:queue::queue::State/q-name s')
                           :tick-armed? (:wat::core::first pair)
                           :arm-tick (:queue::queue::State/arm-tick s')
              :counters (:queue::queue::State/counters s')
              :seen-ids (:queue::queue::State/seen-ids s'))]
                     (:wat::service::Outcome::Continue s2
                       (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Accepted take)))
                       (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                       (:wat::core::second pair)))
                   (:wat::core::let
                     [wpair (:wat::core::foldl
                              (:wat::core::fn [acc <- (:wat::core::Tuple :- [:queue::TakeAcc
                                                                             (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])])
                                               w   <- :queue::Waiter]
                                -> (:wat::core::Tuple :- [:queue::TakeAcc
                                                          (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])])
                                (:wat::core::let
                                  [ta   (:wat::core::first acc)
                                   box  (:wat::core::second acc)
                                   st   (:queue::TakeAcc/store ta)
                                   keep (:queue::TakeAcc/keep ta)
                                   taken (:queue::TakeAcc/calls ta)
                      ns-acc (:queue::TakeAcc/ns ta)
                      sc-acc (:queue::TakeAcc/scan-calls ta)
                      sn-acc (:queue::TakeAcc/scan-ns ta)
                      pc-acc (:queue::TakeAcc/put-calls ta)
                      pn-acc (:queue::TakeAcc/put-ns ta)
                                   empty-ok (:queue::Queue::Reply::Receive
                                              (:queue::Queue::ReceiveResponse::Ok
                                                (:wat::core::Vector :- [:queue::Envelope])))]
                                  (:wat::core::if (:wat::i64::<= (:queue::Waiter/deadline-ns w) now-ns)
                                    (:wat::core::Tuple
                                      (:queue::TakeAcc :store st :keep keep :calls taken :ns ns-acc :scan-calls sc-acc :scan-ns sn-acc :put-calls pc-acc :put-ns pn-acc)
                                      (:wat::core::conj box
                                        (:wat::service::Directed :conn-id (:queue::Waiter/conn-id w) :reply empty-ok)))
                                    (:wat::core::let
                                      [taken-pair (:wat::core::apply (:queue::queue::State/take s')
                                                     st
                                                     (:queue::Waiter/queue w)
                                                     [now-ns
                                                      (:queue::Waiter/visibility-ns w)
                                                      (:queue::Waiter/limit w)])
                                       st' (:wat::core::first taken-pair)
                                       envs (:wat::core::second taken-pair)
                          take-split (:wat::core::third taken-pair)
                          take-scan-ns (:wat::core::first take-split)
                          take-put-ns (:wat::core::second take-split)
                          take-ns (:wat::i64::+ take-scan-ns take-put-ns)]
                                      (:wat::core::if (:wat::core::empty? envs)
                                        (:wat::core::Tuple
                                          (:queue::TakeAcc :store st' :keep (:wat::vector::conj keep w) :calls (:wat::i64::+ taken 1) :ns (:wat::i64::+ ns-acc take-ns) :scan-calls (:wat::i64::+ sc-acc 1) :scan-ns (:wat::i64::+ sn-acc take-scan-ns) :put-calls pc-acc :put-ns pn-acc)
                                          box)
                                        (:wat::core::Tuple
                                          (:queue::TakeAcc :store st' :keep keep :calls (:wat::i64::+ taken 2) :ns (:wat::i64::+ ns-acc take-ns) :scan-calls (:wat::i64::+ sc-acc 1) :scan-ns (:wat::i64::+ sn-acc take-scan-ns) :put-calls (:wat::i64::+ pc-acc 1) :put-ns (:wat::i64::+ pn-acc take-put-ns))
                                          (:wat::core::conj box
                                            (:wat::service::Directed
                                              :conn-id (:queue::Waiter/conn-id w)
                                              :reply (:queue::Queue::Reply::Receive
                                                       (:queue::Queue::ReceiveResponse::Ok envs))))))))))
                              (:wat::core::Tuple
                                (:queue::TakeAcc
                                  :store store
                                  :keep (:wat::core::PersistentVector :- [:queue::Waiter])
                                  :calls 0
                                  :ns 0
                                  :scan-calls 0
                                  :scan-ns 0
                                  :put-calls 0
                                  :put-ns 0)
                                (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]))
                              (:queue::queue::State/waiters s'))
                      store2 (:queue::TakeAcc/store (:wat::core::first wpair))
                      keep (:queue::TakeAcc/keep (:wat::core::first wpair))
                      box  (:wat::core::second wpair)
                      s2 (:queue::queue::State
                           :durable (:queue::queue::State/durable s')
                           :store store2
                           :take (:queue::queue::State/take s')
                           :waiters keep
                           :outbox (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                           :receive-calls (:queue::queue::State/receive-calls s')
                           :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s') (:queue::TakeAcc/calls (:wat::core::first wpair))) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s') (:queue::TakeAcc/ns (:wat::core::first wpair)))
              :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s') (:queue::TakeAcc/put-calls (:wat::core::first wpair))) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s') (:queue::TakeAcc/put-ns (:wat::core::first wpair)))
              :delete-calls (:queue::queue::State/delete-calls s') :delete-ns (:queue::queue::State/delete-ns s')
              :count-calls (:queue::queue::State/count-calls s') :count-ns (:queue::queue::State/count-ns s')
              :scan-calls (:wat::i64::+ (:queue::queue::State/scan-calls s') (:queue::TakeAcc/scan-calls (:wat::core::first wpair))) :scan-ns (:wat::i64::+ (:queue::queue::State/scan-ns s') (:queue::TakeAcc/scan-ns (:wat::core::first wpair)))
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                           :depth (:queue::queue::State/depth s') :total (:queue::queue::State/total s')
                           :q-name (:queue::queue::State/q-name s')
                           :tick-armed? (:queue::queue::State/tick-armed? s')
                           :arm-tick (:queue::queue::State/arm-tick s')
              :counters (:queue::queue::State/counters s')
              :seen-ids (:queue::queue::State/seen-ids s'))
                      pair (:wat::core::apply (:queue::queue::State/arm-tick s2)
                              (:queue::queue::State/tick-armed? s2)
                              [(:wat::core::count (:queue::queue::State/waiters s2)) 1000000])
                      s3 (:queue::queue::State
                           :durable (:queue::queue::State/durable s2)
                           :store store2
                           :take (:queue::queue::State/take s2)
                           :waiters (:queue::queue::State/waiters s2)
                           :outbox (:queue::queue::State/outbox s2)
                           :receive-calls (:queue::queue::State/receive-calls s2) :store-calls (:queue::queue::State/store-calls s2) :store-ns (:queue::queue::State/store-ns s2)
              :put-calls (:queue::queue::State/put-calls s2) :put-ns (:queue::queue::State/put-ns s2)
              :delete-calls (:queue::queue::State/delete-calls s2) :delete-ns (:queue::queue::State/delete-ns s2)
              :count-calls (:queue::queue::State/count-calls s2) :count-ns (:queue::queue::State/count-ns s2)
              :scan-calls (:queue::queue::State/scan-calls s2) :scan-ns (:queue::queue::State/scan-ns s2)
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                           :depth (:queue::queue::State/depth s2) :total (:queue::queue::State/total s2)
                           :q-name (:queue::queue::State/q-name s2)
                           :tick-armed? (:wat::core::first pair)
                           :arm-tick (:queue::queue::State/arm-tick s2)
              :counters (:queue::queue::State/counters s2)
              :seen-ids (:queue::queue::State/seen-ids s2))
                      ok (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Accepted take)))]
                     (:wat::service::Outcome::Continue s3 ok box (:wat::core::second pair))))))
             ((:wat::query::Store::PutResponse::Transient _e)
               (:wat::core::let
                 [retry-acc (:queue::queue::retry-put store rows)
                  n-retry (:queue::RetryAcc/n retry-acc)
                  retry-ns (:queue::RetryAcc/ns retry-acc)
                  s-r (:queue::queue::State
                        :durable (:queue::queue::State/durable s)
                        :store store
                        :take (:queue::queue::State/take s)
                        :waiters (:queue::queue::State/waiters s)
                        :outbox (:queue::queue::State/outbox s)
                        :receive-calls (:queue::queue::State/receive-calls s)
                        :store-calls (:wat::i64::+ sc0 (:wat::i64::+ 1 n-retry)) :store-ns (:wat::i64::+ sn0 (:wat::i64::+ put-ns retry-ns))
                        ;; retry-put is puts only — RetryAcc/n extra calls, RetryAcc/ns.
                        :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s) (:wat::i64::+ 1 n-retry)) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s) (:wat::i64::+ put-ns retry-ns))
                        :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
                        :count-calls cc0 :count-ns cn0
                        :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                        :handler-ns (:queue::queue::State/handler-ns s)
                        :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                        :q-name q
                        :tick-armed? (:queue::queue::State/tick-armed? s)
                        :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
                 (:queue::queue::send-after-put s-r store q now-ns take start-ns)))
             ((:wat::query::Store::PutResponse::Constraint _e)
               (:wat::kernel::assertion-failed! "queue.send: store put Constraint" :wat::core::None :wat::core::None))
             ((:wat::query::Store::PutResponse::Fatal _e)
               (:wat::kernel::assertion-failed! "queue.send: store put Fatal" :wat::core::None :wat::core::None))
             ((:wat::query::Store::PutResponse::RequestTooLarge _b _c)
               (:wat::kernel::assertion-failed! "queue.send: store put RequestTooLarge" :wat::core::None :wat::core::None))
             ((:wat::query::Store::PutResponse::RequestMalformed _p _e _g)
               (:wat::kernel::assertion-failed! "queue.send: store put RequestMalformed" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost _cause)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:queue::queue::Record/store-addr (:queue::queue::State/durable s)))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])
              s' (:queue::queue::State
                    :durable (:queue::queue::State/durable s)
                    :store fresh
                    :take (:queue::queue::State/take s)
                    :waiters (:queue::queue::State/waiters s)
                    :outbox (:queue::queue::State/outbox s)
                    :receive-calls (:queue::queue::State/receive-calls s)
                    :store-calls (:wat::i64::+ sc0 1) :store-ns (:wat::i64::+ sn0 put-ns)
                       :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s) 1) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s) put-ns)
                       :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
                       :count-calls cc0 :count-ns cn0
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                    :q-name q
                    :tick-armed? (:queue::queue::State/tick-armed? s)
                    :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
             ;; Do not claim Accepted n — the put is unknowable. Accepted 0 is the caller's retry.
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:queue::Queue::Reply::Send
                 (:queue::Queue::SendResponse::Accepted 0)))
               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
               none-alarms)))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "queue.send: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:queue::queue::Record/store-addr (:queue::queue::State/durable s)))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])
              s' (:queue::queue::State
                    :durable (:queue::queue::State/durable s)
                    :store fresh
                    :take (:queue::queue::State/take s)
                    :waiters (:queue::queue::State/waiters s)
                    :outbox (:queue::queue::State/outbox s)
                    :receive-calls (:queue::queue::State/receive-calls s)
                    :store-calls (:wat::i64::+ sc0 1) :store-ns (:wat::i64::+ sn0 put-ns)
                       :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s) 1) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s) put-ns)
                       :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
                       :count-calls cc0 :count-ns cn0
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                    :q-name q
                    :tick-armed? (:queue::queue::State/tick-armed? s)
                    :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
             ;; Do not claim Accepted n — the put is unknowable. Accepted 0 is the caller's retry.
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:queue::Queue::Reply::Send
                 (:queue::Queue::SendResponse::Accepted 0)))
               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
               none-alarms))) (:wat::kernel::RecvOutcome::TimedOut (:wat::core::let [fresh (:wat::core::match (:wat::kernel::connect (:queue::queue::Record/store-addr (:queue::queue::State/durable s))) ((:wat::kernel::ConnectOutcome::Connected p) p) (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None))) none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]) s' (:queue::queue::State :durable (:queue::queue::State/durable s) :store fresh :take (:queue::queue::State/take s) :waiters (:queue::queue::State/waiters s) :outbox (:queue::queue::State/outbox s) :receive-calls (:queue::queue::State/receive-calls s) :store-calls (:wat::i64::+ sc0 1) :store-ns (:wat::i64::+ sn0 put-ns)
                       :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s) 1) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s) put-ns)
                       :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
                       :count-calls cc0 :count-ns cn0
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns)) :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s) :q-name q :tick-armed? (:queue::queue::State/tick-armed? s) :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))] (:wat::service::Outcome::Continue s' (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Accepted 0))) (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]) none-alarms))) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))))))

   (receive [s ctx req]
     (:wat::core::let
       [start-ns (:wat::service::Invocation/start-ns ctx)
        store0 (:queue::queue::State/store s)
        q      (:queue::Queue::ReceiveRequest/queue req)
        now-ns (:queue::Queue::ReceiveRequest/now-ns req)
        vis-ns (:queue::Queue::ReceiveRequest/visibility-ns req)
        lim    (:queue::Queue::ReceiveRequest/limit req)
        wait   (:queue::Queue::ReceiveRequest/wait req)
        rec    (:queue::queue::State/durable s)
        rate   (:queue::queue::Record/drop-recv-bp rec)
        pair   (:wat::core::if (:wat::i64::> rate 0)
                 (:wat::rand::int-from (:queue::queue::Record/drop-seed rec) 0 10000)
                 (:wat::core::Tuple (:queue::queue::Record/drop-seed rec) 0))
        seed1  (:wat::core::first pair)
        bp     (:wat::core::second pair)
        hit?   (:wat::core::and (:wat::i64::> rate 0) (:wat::i64::< bp rate))
        rec'   (:queue::queue::Record
                 :cap (:queue::queue::Record/cap rec)
                 :store-addr (:queue::queue::Record/store-addr rec)
                 :drop-recv-bp rate
                 :drop-ack-bp (:queue::queue::Record/drop-ack-bp rec)
                 :drop-seed seed1)
        calls  (:wat::i64::+ (:queue::queue::State/receive-calls s) 1)
        taken-pair (:wat::core::apply (:queue::queue::State/take s) store0 q [now-ns vis-ns lim])
        store  (:wat::core::first taken-pair)
        envs   (:wat::core::second taken-pair)
        take-split (:wat::core::third taken-pair)
        take-scan-ns (:wat::core::first take-split)
        take-put-ns (:wat::core::second take-split)
        take-ns (:wat::i64::+ take-scan-ns take-put-ns)
        ;; take is ALWAYS one scan-index; the re-put happens only when it found rows.
        take-sc (:wat::core::if (:wat::core::empty? envs) 1 2)
        take-pc (:wat::core::if (:wat::core::empty? envs) 0 1)
        s-n    (:queue::queue::State
                 :durable rec'
                 :store store
                 :take (:queue::queue::State/take s)
                 :waiters (:queue::queue::State/waiters s)
                 :outbox (:queue::queue::State/outbox s)
                 :receive-calls calls
                 :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) take-sc) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) take-ns)
                 :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s) take-pc) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s) take-put-ns)
                 :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
                 :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
                 :scan-calls (:wat::i64::+ (:queue::queue::State/scan-calls s) 1) :scan-ns (:wat::i64::+ (:queue::queue::State/scan-ns s) take-scan-ns)
                 :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                 :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                 :q-name q
                 :tick-armed? (:queue::queue::State/tick-armed? s)
                 :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
       (:wat::core::if (:wat::core::not (:wat::core::empty? envs))
         (:wat::core::let
           [cold (:queue::queue::State/counters s-n)
            rd-pair
              (:wat::core::foldl
                (:wat::core::fn
                  [acc <- (:wat::core::Tuple :- [(:wat::core::PersistentSet :- [:wat::core::String]) :wat::core::i64])
                   e   <- :queue::Envelope]
                  -> (:wat::core::Tuple :- [(:wat::core::PersistentSet :- [:wat::core::String]) :wat::core::i64])
                  (:wat::core::let
                    [seen (:wat::core::first acc)
                     rd   (:wat::core::second acc)
                     id   (:queue::Envelope/id e)]
                    (:wat::core::if (:wat::set::contains? seen id)
                      (:wat::core::Tuple seen (:wat::i64::+ rd 1))
                      (:wat::core::Tuple (:wat::set::conj seen id) rd))))
                (:wat::core::Tuple (:queue::queue::State/seen-ids s-n) (:queue::Counters/redeliveries cold))
                envs)
            pair (:wat::core::apply (:queue::queue::State/arm-tick s-n)
                    (:queue::queue::State/tick-armed? s-n)
                    [(:wat::core::count (:queue::queue::State/waiters s-n)) 1000000])
            s-a (:queue::queue::State
                  :durable (:queue::queue::State/durable s-n)
                  :store store
                  :take (:queue::queue::State/take s-n)
                  :waiters (:queue::queue::State/waiters s-n)
                  :outbox (:queue::queue::State/outbox s-n)
                  :receive-calls calls :store-calls (:queue::queue::State/store-calls s-n) :store-ns (:queue::queue::State/store-ns s-n)
                    :put-calls (:queue::queue::State/put-calls s-n) :put-ns (:queue::queue::State/put-ns s-n)
                    :delete-calls (:queue::queue::State/delete-calls s-n) :delete-ns (:queue::queue::State/delete-ns s-n)
                    :count-calls (:queue::queue::State/count-calls s-n) :count-ns (:queue::queue::State/count-ns s-n)
                    :scan-calls (:queue::queue::State/scan-calls s-n) :scan-ns (:queue::queue::State/scan-ns s-n)
                    :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                  :depth (:queue::queue::State/depth s-n) :total (:queue::queue::State/total s-n)
                  :q-name (:queue::queue::State/q-name s-n)
                  :tick-armed? (:wat::core::first pair)
                  :arm-tick (:queue::queue::State/arm-tick s-n)
                  ;; Changes a cold counter → the carrier is rebuilt here.
                  :counters (:queue::Counters
                              :ticks (:queue::Counters/ticks cold)
                              :acks (:queue::Counters/acks cold)
                              :sends-accepted (:queue::Counters/sends-accepted cold)
                              :sends-refused (:queue::Counters/sends-refused cold)
                              :redeliveries (:wat::core::second rd-pair)
                              :expired-waiters (:queue::Counters/expired-waiters cold)
                              ;; ⭑ THE INJECTOR COUNTS ITS OWN FIRE, here, at the site that
                              ;; suppresses the reply four lines below — not at the `hit?`
                              ;; computation, so the counter cannot drift from the behaviour.
                              :recv-drops (:wat::i64::+ (:queue::Counters/recv-drops cold)
                                            (:wat::core::if hit? 1 0))
                              :ack-drops (:queue::Counters/ack-drops cold)
                              :ack-calls (:queue::Counters/ack-calls cold)
                              ;; PATH 1 of 2 that reaches a reply: envelopes were taken.
                              :recv-replies (:wat::i64::+ (:queue::Counters/recv-replies cold) 1))
              :seen-ids (:wat::core::first rd-pair))]
           (:wat::service::Outcome::Continue s-a
             (:wat::core::if hit?
               :wat::core::None
               (:wat::core::Some (:queue::Queue::Reply::Receive (:queue::Queue::ReceiveResponse::Ok envs))))
             (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
             (:wat::core::second pair)))
         (:wat::core::match wait
           ((:queue::Queue::Wait::Immediate)
           (:wat::core::let
             [pair (:wat::core::apply (:queue::queue::State/arm-tick s-n)
                      (:queue::queue::State/tick-armed? s-n)
                      [(:wat::core::count (:queue::queue::State/waiters s-n)) 1000000])
              s-a (:queue::queue::State
                    :durable (:queue::queue::State/durable s-n)
                    :store store
                    :take (:queue::queue::State/take s-n)
                    :waiters (:queue::queue::State/waiters s-n)
                    :outbox (:queue::queue::State/outbox s-n)
                    :receive-calls calls :store-calls (:queue::queue::State/store-calls s-n) :store-ns (:queue::queue::State/store-ns s-n)
                    :put-calls (:queue::queue::State/put-calls s-n) :put-ns (:queue::queue::State/put-ns s-n)
                    :delete-calls (:queue::queue::State/delete-calls s-n) :delete-ns (:queue::queue::State/delete-ns s-n)
                    :count-calls (:queue::queue::State/count-calls s-n) :count-ns (:queue::queue::State/count-ns s-n)
                    :scan-calls (:queue::queue::State/scan-calls s-n) :scan-ns (:queue::queue::State/scan-ns s-n)
                    :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s-n) :total (:queue::queue::State/total s-n)
                    :q-name (:queue::queue::State/q-name s-n)
                    :tick-armed? (:wat::core::first pair)
                    :arm-tick (:queue::queue::State/arm-tick s-n)
              ;; ⛔ THIS SITE USED TO PASS THE CARRIER THROUGH UNCHANGED, and it is the
              ;; immediate-EMPTY reply — which `hit?` suppresses four lines below exactly as
              ;; the take path does. So a drop fired here and was invisible, and the inbox
              ;; (the tier that is polled empty most) read 1.87% against a 5% setting while
              ;; every sub read 5.0%. A counter that covers one of two paths that can do the
              ;; thing is not a slow counter, it is a wrong one.
              :counters (:queue::Counters
                          :ticks (:queue::Counters/ticks (:queue::queue::State/counters s-n))
                          :acks (:queue::Counters/acks (:queue::queue::State/counters s-n))
                          :sends-accepted (:queue::Counters/sends-accepted (:queue::queue::State/counters s-n))
                          :sends-refused (:queue::Counters/sends-refused (:queue::queue::State/counters s-n))
                          :redeliveries (:queue::Counters/redeliveries (:queue::queue::State/counters s-n))
                          :expired-waiters (:queue::Counters/expired-waiters (:queue::queue::State/counters s-n))
                          :recv-drops (:wat::i64::+ (:queue::Counters/recv-drops (:queue::queue::State/counters s-n))
                                        (:wat::core::if hit? 1 0))
                          :ack-drops (:queue::Counters/ack-drops (:queue::queue::State/counters s-n))
                          :ack-calls (:queue::Counters/ack-calls (:queue::queue::State/counters s-n))
                          ;; PATH 2 of 2 that reaches a reply: nothing was available, and an
                          ;; empty Ok is still a reply the injector can take away.
                          :recv-replies (:wat::i64::+ (:queue::Counters/recv-replies (:queue::queue::State/counters s-n)) 1))
              :seen-ids (:queue::queue::State/seen-ids s-n))]
             (:wat::service::Outcome::Continue s-a
               (:wat::core::if hit?
                 :wat::core::None
                 (:wat::core::Some (:queue::Queue::Reply::Receive (:queue::Queue::ReceiveResponse::Ok
                   (:wat::core::Vector :- [:queue::Envelope])))))
               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
               (:wat::core::second pair))))
           ((:queue::Queue::Wait::UpTo d)
           (:wat::core::let
             [w (:queue::Waiter
                  :conn-id (:wat::service::Invocation/conn-id ctx)
                  :queue q
                  :limit lim
                  :visibility-ns vis-ns
                  :deadline-ns (:wat::core::+ (:wat::service::Invocation/start-ns ctx)
                                 (:wat::time::nanoseconds d)))
              s-w (:queue::queue::State
                    :durable (:queue::queue::State/durable s-n)
                    :store store
                    :take (:queue::queue::State/take s-n)
                    :waiters (:wat::vector::conj (:queue::queue::State/waiters s-n) w)
                    :outbox (:queue::queue::State/outbox s-n)
                    :receive-calls calls :store-calls (:queue::queue::State/store-calls s-n) :store-ns (:queue::queue::State/store-ns s-n)
                    :put-calls (:queue::queue::State/put-calls s-n) :put-ns (:queue::queue::State/put-ns s-n)
                    :delete-calls (:queue::queue::State/delete-calls s-n) :delete-ns (:queue::queue::State/delete-ns s-n)
                    :count-calls (:queue::queue::State/count-calls s-n) :count-ns (:queue::queue::State/count-ns s-n)
                    :scan-calls (:queue::queue::State/scan-calls s-n) :scan-ns (:queue::queue::State/scan-ns s-n)
                    :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s-n) :total (:queue::queue::State/total s-n)
                    :q-name (:queue::queue::State/q-name s-n)
                    :tick-armed? (:queue::queue::State/tick-armed? s-n)
                    :arm-tick (:queue::queue::State/arm-tick s-n)
              :counters (:queue::queue::State/counters s-n)
              :seen-ids (:queue::queue::State/seen-ids s-n))
              pair (:wat::core::apply (:queue::queue::State/arm-tick s-w)
                      (:queue::queue::State/tick-armed? s-w)
                      [(:wat::core::count (:queue::queue::State/waiters s-w))
                       (:wat::time::nanoseconds d)])
              s-a (:queue::queue::State
                    :durable (:queue::queue::State/durable s-w)
                    :store store
                    :take (:queue::queue::State/take s-w)
                    :waiters (:queue::queue::State/waiters s-w)
                    :outbox (:queue::queue::State/outbox s-w)
                    :receive-calls calls :store-calls (:queue::queue::State/store-calls s-w) :store-ns (:queue::queue::State/store-ns s-w)
                    :put-calls (:queue::queue::State/put-calls s-w) :put-ns (:queue::queue::State/put-ns s-w)
                    :delete-calls (:queue::queue::State/delete-calls s-w) :delete-ns (:queue::queue::State/delete-ns s-w)
                    :count-calls (:queue::queue::State/count-calls s-w) :count-ns (:queue::queue::State/count-ns s-w)
                    :scan-calls (:queue::queue::State/scan-calls s-w) :scan-ns (:queue::queue::State/scan-ns s-w)
                    :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s-w) :total (:queue::queue::State/total s-w)
                    :q-name (:queue::queue::State/q-name s-w)
                    :tick-armed? (:wat::core::first pair)
                    :arm-tick (:queue::queue::State/arm-tick s-w)
              :counters (:queue::queue::State/counters s-w)
              :seen-ids (:queue::queue::State/seen-ids s-w))]
             (:wat::service::Outcome::Continue s-a
               :wat::core::None
               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
               (:wat::core::second pair))))))))

   (ack [s ctx req]
     (:wat::core::let
       [start-ns (:wat::service::Invocation/start-ns ctx)
        store (:queue::queue::State/store s)
        q     (:queue::Queue::AckRequest/queue req)
        ids   (:queue::Queue::AckRequest/ids req)
        rec   (:queue::queue::State/durable s)
        rate  (:queue::queue::Record/drop-ack-bp rec)
        pair  (:wat::core::if (:wat::i64::> rate 0)
                (:wat::rand::int-from (:queue::queue::Record/drop-seed rec) 0 10000)
                (:wat::core::Tuple (:queue::queue::Record/drop-seed rec) 0))
        seed1 (:wat::core::first pair)
        bp    (:wat::core::second pair)
        hit?  (:wat::core::and (:wat::i64::> rate 0) (:wat::i64::< bp rate))
        rec'  (:queue::queue::Record
                :cap (:queue::queue::Record/cap rec)
                :store-addr (:queue::queue::Record/store-addr rec)
                :drop-recv-bp (:queue::queue::Record/drop-recv-bp rec)
                :drop-ack-bp rate
                :drop-seed seed1)
        _cap (:wat::core::if (:wat::i64::> (:wat::core::count ids) 10)
                (:wat::kernel::assertion-failed! "queue.ack: batch larger than 10" :wat::core::None :wat::core::None)
                nil)
        keys (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::Key])  id <- :wat::core::String]
                 -> (:wat::core::Vector :- [:wat::query::Key])
                 (:wat::core::conj acc (:wat::query::Key :pk q :sk id)))
               (:wat::core::Vector :- [:wat::query::Key])
               ids)
        t-del (:wat::time::epoch-nanos (:wat::time::now))
        del   (:wat::query::Store/delete store
                (:wat::query::Store::DeleteRequest keys))
        del-ns (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t-del)
        ;; Both ack arms below bump `acks`, so both rebuild the carrier. Read it once.
        cold (:queue::queue::State/counters s)]
       (:wat::core::match del
         ((:wat::kernel::RecvOutcome::Message sresp)
           (:wat::core::match sresp
             ((:wat::query::Store::DeleteResponse::Success)
               (:wat::core::let
                 [s' (:queue::queue::State
                       :durable rec'
                       :store store
                       :take (:queue::queue::State/take s)
                       :waiters (:queue::queue::State/waiters s)
                       :outbox (:queue::queue::State/outbox s)
                       :receive-calls (:queue::queue::State/receive-calls s)
                       :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) 1) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) del-ns)
                       :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
                       :delete-calls (:wat::i64::+ (:queue::queue::State/delete-calls s) 1) :delete-ns (:wat::i64::+ (:queue::queue::State/delete-ns s) del-ns)
                       :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                       :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                       :q-name q
                       :tick-armed? (:queue::queue::State/tick-armed? s)
                       :arm-tick (:queue::queue::State/arm-tick s)
                       ;; Changes a cold counter → the carrier is rebuilt here.
                       :counters (:queue::Counters
                                   :ticks (:queue::Counters/ticks cold)
                                   :acks (:wat::i64::+ (:queue::Counters/acks cold) (:wat::core::count ids))
                                   :sends-accepted (:queue::Counters/sends-accepted cold)
                                   :sends-refused (:queue::Counters/sends-refused cold)
                                   :redeliveries (:queue::Counters/redeliveries cold)
                                   :expired-waiters (:queue::Counters/expired-waiters cold)
                                   :recv-drops (:queue::Counters/recv-drops cold)
                                   :ack-drops (:wat::i64::+ (:queue::Counters/ack-drops cold)
                                                (:wat::core::if hit? 1 0))
                                   :ack-calls (:wat::i64::+ (:queue::Counters/ack-calls cold) 1) :recv-replies (:queue::Counters/recv-replies cold))
                       :seen-ids (:queue::queue::State/seen-ids s))
                  pair (:wat::core::apply (:queue::queue::State/arm-tick s')
                          (:queue::queue::State/tick-armed? s')
                          [(:wat::core::count (:queue::queue::State/waiters s')) 1000000])
                  s-a (:queue::queue::State
                        :durable (:queue::queue::State/durable s')
                        :store store
                        :take (:queue::queue::State/take s')
                        :waiters (:queue::queue::State/waiters s')
                        :outbox (:queue::queue::State/outbox s')
                        :receive-calls (:queue::queue::State/receive-calls s') :store-calls (:queue::queue::State/store-calls s') :store-ns (:queue::queue::State/store-ns s')
              :put-calls (:queue::queue::State/put-calls s') :put-ns (:queue::queue::State/put-ns s')
              :delete-calls (:queue::queue::State/delete-calls s') :delete-ns (:queue::queue::State/delete-ns s')
              :count-calls (:queue::queue::State/count-calls s') :count-ns (:queue::queue::State/count-ns s')
              :scan-calls (:queue::queue::State/scan-calls s') :scan-ns (:queue::queue::State/scan-ns s')
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                        :depth (:queue::queue::State/depth s') :total (:queue::queue::State/total s')
                        :q-name (:queue::queue::State/q-name s')
                        :tick-armed? (:wat::core::first pair)
                        :arm-tick (:queue::queue::State/arm-tick s')
              :counters (:queue::queue::State/counters s')
              :seen-ids (:queue::queue::State/seen-ids s'))]
                 (:wat::service::Outcome::Continue s-a
                   (:wat::core::if hit?
                     :wat::core::None
                     (:wat::core::Some (:queue::Queue::Reply::Ack (:queue::Queue::AckResponse::Ok))))
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
                   (:wat::core::second pair))))
             ((:wat::query::Store::DeleteResponse::Transient _e)
               (:wat::core::let
                 [retry-acc (:queue::queue::retry-delete store keys)
                  n-retry (:queue::RetryAcc/n retry-acc)
                  retry-ns (:queue::RetryAcc/ns retry-acc)
                  s-r (:queue::queue::State
                        :durable rec'
                        :store store
                        :take (:queue::queue::State/take s)
                        :waiters (:queue::queue::State/waiters s)
                        :outbox (:queue::queue::State/outbox s)
                        :receive-calls (:queue::queue::State/receive-calls s)
                        :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s)
                                        (:wat::i64::+ 1 n-retry)) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) (:wat::i64::+ del-ns retry-ns))
                        ;; retry-delete is deletes only — RetryAcc/n extra calls, RetryAcc/ns.
                        :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
                        :delete-calls (:wat::i64::+ (:queue::queue::State/delete-calls s) (:wat::i64::+ 1 n-retry)) :delete-ns (:wat::i64::+ (:queue::queue::State/delete-ns s) (:wat::i64::+ del-ns retry-ns))
                        :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
                        :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                        :handler-ns (:queue::queue::State/handler-ns s)
                        :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                        :q-name q
                        :tick-armed? (:queue::queue::State/tick-armed? s)
                        :arm-tick (:queue::queue::State/arm-tick s)
                        ;; Changes a cold counter → the carrier is rebuilt here.
                        :counters (:queue::Counters
                                    :ticks (:queue::Counters/ticks cold)
                                    :acks (:wat::i64::+ (:queue::Counters/acks cold) (:wat::core::count ids))
                                    :sends-accepted (:queue::Counters/sends-accepted cold)
                                    :sends-refused (:queue::Counters/sends-refused cold)
                                    :redeliveries (:queue::Counters/redeliveries cold)
                                    :expired-waiters (:queue::Counters/expired-waiters cold)
                                    :recv-drops (:queue::Counters/recv-drops cold)
                                    :ack-drops (:wat::i64::+ (:queue::Counters/ack-drops cold)
                                                 (:wat::core::if hit? 1 0))
                                    :ack-calls (:wat::i64::+ (:queue::Counters/ack-calls cold) 1) :recv-replies (:queue::Counters/recv-replies cold))
                        :seen-ids (:queue::queue::State/seen-ids s))]
                 (:queue::queue::ack-after-delete s-r store q rec' hit? start-ns)))
             ((:wat::query::Store::DeleteResponse::Constraint _e)
               (:wat::kernel::assertion-failed! "queue.ack: store delete Constraint" :wat::core::None :wat::core::None))
             ((:wat::query::Store::DeleteResponse::Fatal _e)
               (:wat::kernel::assertion-failed! "queue.ack: store delete Fatal" :wat::core::None :wat::core::None))
             ((:wat::query::Store::DeleteResponse::RequestTooLarge _b _c)
               (:wat::kernel::assertion-failed! "queue.ack: store delete RequestTooLarge" :wat::core::None :wat::core::None))
             ((:wat::query::Store::DeleteResponse::RequestMalformed _p _e _g)
               (:wat::kernel::assertion-failed! "queue.ack: store delete RequestMalformed" :wat::core::None :wat::core::None))))
         ((:wat::kernel::RecvOutcome::Lost _cause)
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:queue::queue::Record/store-addr (:queue::queue::State/durable s)))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:queue::queue::State
                    :durable (:queue::queue::State/durable s)
                    :store fresh
                    :take (:queue::queue::State/take s)
                    :waiters (:queue::queue::State/waiters s)
                    :outbox (:queue::queue::State/outbox s)
                    :receive-calls (:queue::queue::State/receive-calls s)
                    :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) 1) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) del-ns)
                       :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
                       :delete-calls (:wat::i64::+ (:queue::queue::State/delete-calls s) 1) :delete-ns (:wat::i64::+ (:queue::queue::State/delete-ns s) del-ns)
                       :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                    :q-name q
                    :tick-armed? (:queue::queue::State/tick-armed? s)
                    :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
             ;; Do not delete. Reply Ok so the worker does not hang.
             ;; Visibility + Seen absorb a possible duplicate.
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:queue::Queue::Reply::Ack (:queue::Queue::AckResponse::Ok)))
               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
               (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::assertion-failed! "queue.ack: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:queue::queue::Record/store-addr (:queue::queue::State/durable s)))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:queue::queue::State
                    :durable (:queue::queue::State/durable s)
                    :store fresh
                    :take (:queue::queue::State/take s)
                    :waiters (:queue::queue::State/waiters s)
                    :outbox (:queue::queue::State/outbox s)
                    :receive-calls (:queue::queue::State/receive-calls s)
                    :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) 1) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) del-ns)
                       :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
                       :delete-calls (:wat::i64::+ (:queue::queue::State/delete-calls s) 1) :delete-ns (:wat::i64::+ (:queue::queue::State/delete-ns s) del-ns)
                       :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                    :q-name q
                    :tick-armed? (:queue::queue::State/tick-armed? s)
                    :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
             ;; Do not delete. Reply Ok so the worker does not hang.
             ;; Visibility + Seen absorb a possible duplicate.
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:queue::Queue::Reply::Ack (:queue::Queue::AckResponse::Ok)))
               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
               (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])]))))
         (:wat::kernel::RecvOutcome::TimedOut
           (:wat::core::let
             [fresh (:wat::core::match
                      (:wat::kernel::connect (:queue::queue::Record/store-addr (:queue::queue::State/durable s)))
                      ((:wat::kernel::ConnectOutcome::Connected p) p)
                      (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
              s' (:queue::queue::State
                    :durable (:queue::queue::State/durable s)
                    :store fresh
                    :take (:queue::queue::State/take s)
                    :waiters (:queue::queue::State/waiters s)
                    :outbox (:queue::queue::State/outbox s)
                    :receive-calls (:queue::queue::State/receive-calls s)
                    :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) 1) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) del-ns)
                       :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
                       :delete-calls (:wat::i64::+ (:queue::queue::State/delete-calls s) 1) :delete-ns (:wat::i64::+ (:queue::queue::State/delete-ns s) del-ns)
                       :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
                       :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
                       :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
                    :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
                    :q-name q
                    :tick-armed? (:queue::queue::State/tick-armed? s)
                    :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
             ;; Do not delete. Reply Ok so the worker does not hang.
             ;; Visibility + Seen absorb a possible duplicate.
             (:wat::service::Outcome::Continue s'
               (:wat::core::Some (:queue::Queue::Reply::Ack (:queue::Queue::AckResponse::Ok)))
               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
               (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])))) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))))

   (stats [s ctx req]
     (:wat::core::let
       [start-ns (:wat::service::Invocation/start-ns ctx)
        now-ns start-ns
        q      (:queue::queue::State/q-name s)
        cap    (:queue::queue::Record/cap (:queue::queue::State/durable s))
        lim    (:wat::i64::+ cap 1)
        vu     (:wat::core::apply (:queue::queue::State/depth s)
                 (:queue::queue::State/store s) q [now-ns lim])
        depth-ns (:wat::core::third vu)
        ;; The six cold counters come out of the carrier here and go into the
        ;; UNCHANGED flat `:queue::Stats`. ⛔ Stats does NOT gain a nested `counters`
        ;; field: it is built at this ONE site, once per `stats` call, while its
        ;; fields are read 32 times across 8 files. The carrier pays in `State`,
        ;; which is rebuilt 30× per message — not here.
        cold (:queue::queue::State/counters s)
        pair (:wat::core::apply (:queue::queue::State/arm-tick s)
                (:queue::queue::State/tick-armed? s)
                [(:wat::core::count (:queue::queue::State/waiters s)) 1000000])
        s-a (:queue::queue::State
              :durable (:queue::queue::State/durable s)
              :store (:queue::queue::State/store s)
              :take (:queue::queue::State/take s)
              :waiters (:queue::queue::State/waiters s)
              :outbox (:queue::queue::State/outbox s)
              :receive-calls (:queue::queue::State/receive-calls s)
              :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) 2) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) depth-ns)
              ;; `depth` is TWO count-index calls (count-hi at now-ns and at +inf);
              ;; depth-ns is their summed elapsed. Both belong to count.
              :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
              :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
              :count-calls (:wat::i64::+ (:queue::queue::State/count-calls s) 2) :count-ns (:wat::i64::+ (:queue::queue::State/count-ns s) depth-ns)
              :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
              :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
              :q-name q
              :tick-armed? (:wat::core::first pair)
              :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
       (:wat::service::Outcome::Continue s-a
         (:wat::core::Some (:queue::Queue::Reply::Stats (:queue::Queue::StatsResponse::Ok
           (:queue::Stats
             :receive-calls (:queue::queue::State/receive-calls s)
             :ticks (:queue::Counters/ticks cold)
             :visible (:wat::core::first vu)
             :unacked (:wat::core::second vu)
             :store-calls (:queue::queue::State/store-calls s-a)
             :store-ns (:queue::queue::State/store-ns s-a)
             :put-calls (:queue::queue::State/put-calls s-a)
             :put-ns (:queue::queue::State/put-ns s-a)
             :delete-calls (:queue::queue::State/delete-calls s-a)
             :delete-ns (:queue::queue::State/delete-ns s-a)
             :count-calls (:queue::queue::State/count-calls s-a)
             :count-ns (:queue::queue::State/count-ns s-a)
             :scan-calls (:queue::queue::State/scan-calls s-a)
             :scan-ns (:queue::queue::State/scan-ns s-a)
             :handler-ns (:queue::queue::State/handler-ns s-a)
             :sends-accepted (:queue::Counters/sends-accepted cold)
             :sends-refused (:queue::Counters/sends-refused cold)
             :acks (:queue::Counters/acks cold)
             :redeliveries (:queue::Counters/redeliveries cold)
             :expired-waiters (:queue::Counters/expired-waiters cold)
             :recv-drops (:queue::Counters/recv-drops cold)
             :ack-drops (:queue::Counters/ack-drops cold)
             :ack-calls (:queue::Counters/ack-calls cold) :recv-replies (:queue::Counters/recv-replies cold)))))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
         (:wat::core::second pair))))

   ;; Scanning tick: expire past-deadline waiters and try-receive the rest
   ;; (same take-visible the receive arm uses). Sends and re-arm compose in
   ;; one SelfOutcome — no extra arm, no 1 ms stand-in for "and".
   (-tick [s ctx]
     (:wat::core::let
       [start-ns (:wat::service::SelfInvocation/start-ns ctx)
        now   start-ns
        store (:queue::queue::State/store s)
        ;; The only site that changes TWO cold counters (`ticks` and
        ;; `expired-waiters`), so it is also the only one that must read the
        ;; carrier before it can rebuild it.
        cold  (:queue::queue::State/counters s)
        ticks (:wat::i64::+ (:queue::Counters/ticks cold) 1)
        pair  (:wat::core::foldl
                (:wat::core::fn [acc <- (:wat::core::Tuple :- [:queue::TakeAcc
                                                               (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])])
                                 w   <- :queue::Waiter]
                  -> (:wat::core::Tuple :- [:queue::TakeAcc
                                            (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])])
                  (:wat::core::let
                    [ta   (:wat::core::first acc)
                     box  (:wat::core::second acc)
                     st   (:queue::TakeAcc/store ta)
                     keep (:queue::TakeAcc/keep ta)
                     taken (:queue::TakeAcc/calls ta)
                      ns-acc (:queue::TakeAcc/ns ta)
                      sc-acc (:queue::TakeAcc/scan-calls ta)
                      sn-acc (:queue::TakeAcc/scan-ns ta)
                      pc-acc (:queue::TakeAcc/put-calls ta)
                      pn-acc (:queue::TakeAcc/put-ns ta)
                     empty-ok (:queue::Queue::Reply::Receive
                                (:queue::Queue::ReceiveResponse::Ok
                                  (:wat::core::Vector :- [:queue::Envelope])))]
                    (:wat::core::if (:wat::i64::<= (:queue::Waiter/deadline-ns w) now)
                      (:wat::core::Tuple
                        (:queue::TakeAcc :store st :keep keep :calls taken :ns ns-acc :scan-calls sc-acc :scan-ns sn-acc :put-calls pc-acc :put-ns pn-acc)
                        (:wat::core::conj box
                          (:wat::service::Directed :conn-id (:queue::Waiter/conn-id w) :reply empty-ok)))
                      (:wat::core::let
                        [taken-pair (:wat::core::apply (:queue::queue::State/take s)
                                       st
                                       (:queue::Waiter/queue w)
                                       [now
                                        (:queue::Waiter/visibility-ns w)
                                        (:queue::Waiter/limit w)])
                         st' (:wat::core::first taken-pair)
                         envs (:wat::core::second taken-pair)
                          take-split (:wat::core::third taken-pair)
                          take-scan-ns (:wat::core::first take-split)
                          take-put-ns (:wat::core::second take-split)
                          take-ns (:wat::i64::+ take-scan-ns take-put-ns)]
                        (:wat::core::if (:wat::core::empty? envs)
                          (:wat::core::Tuple
                            (:queue::TakeAcc :store st' :keep (:wat::vector::conj keep w) :calls (:wat::i64::+ taken 1) :ns (:wat::i64::+ ns-acc take-ns) :scan-calls (:wat::i64::+ sc-acc 1) :scan-ns (:wat::i64::+ sn-acc take-scan-ns) :put-calls pc-acc :put-ns pn-acc)
                            box)
                          (:wat::core::Tuple
                            (:queue::TakeAcc :store st' :keep keep :calls (:wat::i64::+ taken 2) :ns (:wat::i64::+ ns-acc take-ns) :scan-calls (:wat::i64::+ sc-acc 1) :scan-ns (:wat::i64::+ sn-acc take-scan-ns) :put-calls (:wat::i64::+ pc-acc 1) :put-ns (:wat::i64::+ pn-acc take-put-ns))
                            (:wat::core::conj box
                              (:wat::service::Directed
                                :conn-id (:queue::Waiter/conn-id w)
                                :reply (:queue::Queue::Reply::Receive
                                         (:queue::Queue::ReceiveResponse::Ok envs))))))))))
                (:wat::core::Tuple
                  (:queue::TakeAcc
                    :store store
                    :keep (:wat::core::PersistentVector :- [:queue::Waiter])
                    :calls 0
                    :ns 0
                    :scan-calls 0
                    :scan-ns 0
                    :put-calls 0
                    :put-ns 0)
                  (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]))
                (:queue::queue::State/waiters s))
        store2 (:queue::TakeAcc/store (:wat::core::first pair))
        keep (:queue::TakeAcc/keep (:wat::core::first pair))
        box  (:wat::core::second pair)
        ew (:wat::core::foldl
             (:wat::core::fn [n <- :wat::core::i64  w <- :queue::Waiter] -> :wat::core::i64
               (:wat::core::if (:wat::i64::<= (:queue::Waiter/deadline-ns w) now)
                 (:wat::i64::+ n 1)
                 n))
             0
             (:queue::queue::State/waiters s))
        ;; Tick consumed the outstanding alarm: flag is false before the helper.
        s' (:queue::queue::State
             :durable (:queue::queue::State/durable s)
             :store store2
             :take (:queue::queue::State/take s)
             :waiters keep
             :outbox (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
             :receive-calls (:queue::queue::State/receive-calls s)
             :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s) (:queue::TakeAcc/calls (:wat::core::first pair))) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s) (:queue::TakeAcc/ns (:wat::core::first pair)))
             :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s) (:queue::TakeAcc/put-calls (:wat::core::first pair))) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s) (:queue::TakeAcc/put-ns (:wat::core::first pair)))
             :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
             :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
             :scan-calls (:wat::i64::+ (:queue::queue::State/scan-calls s) (:queue::TakeAcc/scan-calls (:wat::core::first pair))) :scan-ns (:wat::i64::+ (:queue::queue::State/scan-ns s) (:queue::TakeAcc/scan-ns (:wat::core::first pair)))
             :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
             :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
             :q-name (:queue::queue::State/q-name s)
             :tick-armed? false
             :arm-tick (:queue::queue::State/arm-tick s)
             ;; Changes TWO cold counters → the carrier is rebuilt here.
             :counters (:queue::Counters
                         :ticks ticks
                         :acks (:queue::Counters/acks cold)
                         :sends-accepted (:queue::Counters/sends-accepted cold)
                         :sends-refused (:queue::Counters/sends-refused cold)
                         :redeliveries (:queue::Counters/redeliveries cold)
                         :expired-waiters (:wat::i64::+ (:queue::Counters/expired-waiters cold) ew)
                         :recv-drops (:queue::Counters/recv-drops cold)
                         :ack-drops (:queue::Counters/ack-drops cold)
                         :ack-calls (:queue::Counters/ack-calls cold) :recv-replies (:queue::Counters/recv-replies cold))
             :seen-ids (:queue::queue::State/seen-ids s))
        delay (:wat::core::foldl
                (:wat::core::fn [d <- :wat::core::i64  w <- :queue::Waiter] -> :wat::core::i64
                  (:wat::core::let [rem (:wat::core::- (:queue::Waiter/deadline-ns w) now)]
                    (:wat::core::if (:wat::i64::< rem d) rem d)))
                1000000000000000
                keep)
        ;; Tick-rate floor, not a zero guard. The fold above keeps only waiters
        ;; with deadline-ns > now, so delay >= 1 always. Without this, a 1 µs
        ;; remainder would arm a 1 µs alarm and tick the queue a thousand times
        ;; per millisecond. arm-tick builds (Nanosecond delay0) from a computed
        ;; i64 — after Stone A a zero there is LociDiedError/Panic at process
        ;; locus. Keep the floor; it is now also the panic boundary.
        delay0 (:wat::core::if (:wat::i64::< delay 1000000) 1000000 delay)
        pair (:wat::core::apply (:queue::queue::State/arm-tick s')
                false
                [(:wat::core::count keep) delay0])
        s-a (:queue::queue::State
              :durable (:queue::queue::State/durable s')
              :store store2
              :take (:queue::queue::State/take s')
              :waiters keep
              :outbox (:queue::queue::State/outbox s')
              :receive-calls (:queue::queue::State/receive-calls s') :store-calls (:queue::queue::State/store-calls s') :store-ns (:queue::queue::State/store-ns s')
              :put-calls (:queue::queue::State/put-calls s') :put-ns (:queue::queue::State/put-ns s')
              :delete-calls (:queue::queue::State/delete-calls s') :delete-ns (:queue::queue::State/delete-ns s')
              :count-calls (:queue::queue::State/count-calls s') :count-ns (:queue::queue::State/count-ns s')
              :scan-calls (:queue::queue::State/scan-calls s') :scan-ns (:queue::queue::State/scan-ns s')
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
              :depth (:queue::queue::State/depth s') :total (:queue::queue::State/total s')
              :q-name (:queue::queue::State/q-name s')
              :tick-armed? (:wat::core::first pair)
              :arm-tick (:queue::queue::State/arm-tick s')
              ;; `s'` already carries the incremented `ticks` — copy, do not rebuild.
              :counters (:queue::queue::State/counters s')
              :seen-ids (:queue::queue::State/seen-ids s'))]
       (:wat::service::SelfOutcome::Continue s-a box (:wat::core::second pair))))])

;; Retry a transient put. Bound once at load so the send match's Transient
;; arm stays a few lines — unused large arms are paid on every send.
(:wat::core::defn :queue::queue::retry-put
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   rows <- (:wat::core::Vector :- [:wat::query::StoredRow])]
  -> :queue::RetryAcc
  (:wat::core::let
    [nap (:wat::core::fn [] -> :wat::core::nil
            (:wat::core::match
              (:wat::kernel::recv
                (:wat::kernel::after :wat::program::PeerKind::thread
                  (:wat::time::Milliseconds 1) :done))
              ((:wat::kernel::RecvOutcome::Message _m) nil)
              ((:wat::kernel::RecvOutcome::Lost _c) nil)
              (:wat::kernel::RecvOutcome::Stopped nil)
              (:wat::kernel::RecvOutcome::Closed nil)
              (:wat::kernel::RecvOutcome::TimedOut nil) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None))))
     once-put (:wat::core::fn
                 [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
                 -> (:wat::core::Tuple :- [:wat::core::bool :wat::core::i64])
                 (:wat::core::let
                   [t0 (:wat::time::epoch-nanos (:wat::time::now))
                    resp (:wat::query::Store/put st (:wat::query::Store::PutRequest rows))
                    elapsed (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0)]
                   (:wat::core::match resp
                     ((:wat::kernel::RecvOutcome::Message r)
                       (:wat::core::match r
                         ((:wat::query::Store::PutResponse::Success) (:wat::core::Tuple false elapsed))
                         ((:wat::query::Store::PutResponse::Transient _e) (:wat::core::Tuple true elapsed))
                         ((:wat::query::Store::PutResponse::Constraint _e)
                           (:wat::kernel::assertion-failed! "queue.send: store put Constraint" :wat::core::None :wat::core::None))
                         ((:wat::query::Store::PutResponse::Fatal _e)
                           (:wat::kernel::assertion-failed! "queue.send: store put Fatal" :wat::core::None :wat::core::None))
                         ((:wat::query::Store::PutResponse::RequestTooLarge _b _c)
                           (:wat::kernel::assertion-failed! "queue.send: store put RequestTooLarge" :wat::core::None :wat::core::None))
                         ((:wat::query::Store::PutResponse::RequestMalformed _p _e _g)
                           (:wat::kernel::assertion-failed! "queue.send: store put RequestMalformed" :wat::core::None :wat::core::None))))
                     ((:wat::kernel::RecvOutcome::Lost _c)
                       (:wat::kernel::assertion-failed! "queue.send: store put transient, exhausted after 3" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Stopped
                       (:wat::kernel::assertion-failed! "queue.send: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Closed
                       (:wat::kernel::assertion-failed! "queue.send: store put transient, exhausted after 3" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::TimedOut
                       (:wat::kernel::assertion-failed! "queue.send: store put transient, exhausted after 3" :wat::core::None :wat::core::None)) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))))
     _n1 (nap)
     p1 (once-put store)
     t1 (:wat::core::first p1)
     ns1 (:wat::core::second p1)
     p2 (:wat::core::if t1
          (:wat::core::let [_n2 (nap)] (once-put store))
          (:wat::core::Tuple false 0))
     t2 (:wat::core::first p2)
     ns2 (:wat::core::second p2)]
    (:wat::core::if t2
      (:wat::kernel::assertion-failed! "queue.send: store put transient, exhausted after 3" :wat::core::None :wat::core::None)
      (:queue::RetryAcc :n (:wat::core::if t1 2 1) :ns (:wat::i64::+ ns1 ns2)))))

(:wat::core::defn :queue::queue::retry-delete
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   keys <- (:wat::core::Vector :- [:wat::query::Key])]
  -> :queue::RetryAcc
  (:wat::core::let
    [nap (:wat::core::fn [] -> :wat::core::nil
            (:wat::core::match
              (:wat::kernel::recv
                (:wat::kernel::after :wat::program::PeerKind::thread
                  (:wat::time::Milliseconds 1) :done))
              ((:wat::kernel::RecvOutcome::Message _m) nil)
              ((:wat::kernel::RecvOutcome::Lost _c) nil)
              (:wat::kernel::RecvOutcome::Stopped nil)
              (:wat::kernel::RecvOutcome::Closed nil)
              (:wat::kernel::RecvOutcome::TimedOut nil) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None))))
     once-del (:wat::core::fn
                 [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
                 -> (:wat::core::Tuple :- [:wat::core::bool :wat::core::i64])
                 (:wat::core::let
                   [t0 (:wat::time::epoch-nanos (:wat::time::now))
                    resp (:wat::query::Store/delete st (:wat::query::Store::DeleteRequest keys))
                    elapsed (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0)]
                   (:wat::core::match resp
                     ((:wat::kernel::RecvOutcome::Message r)
                       (:wat::core::match r
                         ((:wat::query::Store::DeleteResponse::Success) (:wat::core::Tuple false elapsed))
                         ((:wat::query::Store::DeleteResponse::Transient _e) (:wat::core::Tuple true elapsed))
                         ((:wat::query::Store::DeleteResponse::Constraint _e)
                           (:wat::kernel::assertion-failed! "queue.ack: store delete Constraint" :wat::core::None :wat::core::None))
                         ((:wat::query::Store::DeleteResponse::Fatal _e)
                           (:wat::kernel::assertion-failed! "queue.ack: store delete Fatal" :wat::core::None :wat::core::None))
                         ((:wat::query::Store::DeleteResponse::RequestTooLarge _b _c)
                           (:wat::kernel::assertion-failed! "queue.ack: store delete RequestTooLarge" :wat::core::None :wat::core::None))
                         ((:wat::query::Store::DeleteResponse::RequestMalformed _p _e _g)
                           (:wat::kernel::assertion-failed! "queue.ack: store delete RequestMalformed" :wat::core::None :wat::core::None))))
                     ((:wat::kernel::RecvOutcome::Lost _c)
                       (:wat::kernel::assertion-failed! "queue.ack: store delete transient, exhausted after 3" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Stopped
                       (:wat::kernel::assertion-failed! "queue.ack: stop requested — the store peer was ALIVE" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::Closed
                       (:wat::kernel::assertion-failed! "queue.ack: store delete transient, exhausted after 3" :wat::core::None :wat::core::None))
                     (:wat::kernel::RecvOutcome::TimedOut
                       (:wat::kernel::assertion-failed! "queue.ack: store delete transient, exhausted after 3" :wat::core::None :wat::core::None)) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None)))))
     _n1 (nap)
     p1 (once-del store)
     t1 (:wat::core::first p1)
     ns1 (:wat::core::second p1)
     p2 (:wat::core::if t1
          (:wat::core::let [_n2 (nap)] (once-del store))
          (:wat::core::Tuple false 0))
     t2 (:wat::core::first p2)
     ns2 (:wat::core::second p2)]
    (:wat::core::if t2
      (:wat::kernel::assertion-failed! "queue.ack: store delete transient, exhausted after 3" :wat::core::None :wat::core::None)
      (:queue::RetryAcc :n (:wat::core::if t1 2 1) :ns (:wat::i64::+ ns1 ns2)))))

(:wat::core::defn :queue::queue::ack-after-delete
  [s <- :queue::queue::State
   store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   q <- :wat::core::String
   rec' <- :queue::queue::Record
   hit? <- :wat::core::bool
   start-ns <- :wat::core::i64]
  -> (:wat::service::Outcome :- [:queue::queue::State :queue::Queue::Reply :queue::queue::Op])
  (:wat::core::let
    [s' (:queue::queue::State
          :durable rec'
          :store store
          :take (:queue::queue::State/take s)
          :waiters (:queue::queue::State/waiters s)
          :outbox (:queue::queue::State/outbox s)
          :receive-calls (:queue::queue::State/receive-calls s) :store-calls (:queue::queue::State/store-calls s) :store-ns (:queue::queue::State/store-ns s)
          :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
          :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
          :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
          :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
          :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
          :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
          :q-name q
          :tick-armed? (:queue::queue::State/tick-armed? s)
          :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))
     pair (:wat::core::apply (:queue::queue::State/arm-tick s')
             (:queue::queue::State/tick-armed? s')
             [(:wat::core::count (:queue::queue::State/waiters s')) 1000000])
     s-a (:queue::queue::State
           :durable (:queue::queue::State/durable s')
           :store store
           :take (:queue::queue::State/take s')
           :waiters (:queue::queue::State/waiters s')
           :outbox (:queue::queue::State/outbox s')
           :receive-calls (:queue::queue::State/receive-calls s') :store-calls (:queue::queue::State/store-calls s') :store-ns (:queue::queue::State/store-ns s')
              :put-calls (:queue::queue::State/put-calls s') :put-ns (:queue::queue::State/put-ns s')
              :delete-calls (:queue::queue::State/delete-calls s') :delete-ns (:queue::queue::State/delete-ns s')
              :count-calls (:queue::queue::State/count-calls s') :count-ns (:queue::queue::State/count-ns s')
              :scan-calls (:queue::queue::State/scan-calls s') :scan-ns (:queue::queue::State/scan-ns s')
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
           :depth (:queue::queue::State/depth s') :total (:queue::queue::State/total s')
           :q-name (:queue::queue::State/q-name s')
           :tick-armed? (:wat::core::first pair)
           :arm-tick (:queue::queue::State/arm-tick s')
              :counters (:queue::queue::State/counters s')
              :seen-ids (:queue::queue::State/seen-ids s'))]
    (:wat::service::Outcome::Continue s-a
      (:wat::core::if hit?
        :wat::core::None
        (:wat::core::Some (:queue::Queue::Reply::Ack (:queue::Queue::AckResponse::Ok))))
      (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
      (:wat::core::second pair))))

;; Send-success continuation. Bound once at load so the Transient arm can
;; call it without carrying the continuation as an unused match-arm body
;; (that shape cost +1.7 s on publish — see SCORE-find-the-934-milliseconds).
(:wat::core::defn :queue::queue::send-after-put
  [s <- :queue::queue::State
   store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   q <- :wat::core::String
   now-ns <- :wat::core::i64
   n-ok <- :wat::core::i64
   start-ns <- :wat::core::i64]
  -> (:wat::service::Outcome :- [:queue::queue::State :queue::Queue::Reply :queue::queue::Op])
  (:wat::core::let
    [s' (:queue::queue::State
          :durable (:queue::queue::State/durable s)
          :store store
          :take (:queue::queue::State/take s)
          :waiters (:queue::queue::State/waiters s)
          :outbox (:queue::queue::State/outbox s)
          :receive-calls (:queue::queue::State/receive-calls s) :store-calls (:queue::queue::State/store-calls s) :store-ns (:queue::queue::State/store-ns s)
          :put-calls (:queue::queue::State/put-calls s) :put-ns (:queue::queue::State/put-ns s)
          :delete-calls (:queue::queue::State/delete-calls s) :delete-ns (:queue::queue::State/delete-ns s)
          :count-calls (:queue::queue::State/count-calls s) :count-ns (:queue::queue::State/count-ns s)
          :scan-calls (:queue::queue::State/scan-calls s) :scan-ns (:queue::queue::State/scan-ns s)
          :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
          :depth (:queue::queue::State/depth s) :total (:queue::queue::State/total s)
          :q-name q
          :tick-armed? (:queue::queue::State/tick-armed? s)
          :arm-tick (:queue::queue::State/arm-tick s)
              :counters (:queue::queue::State/counters s)
              :seen-ids (:queue::queue::State/seen-ids s))]
    (:wat::core::if (:wat::core::empty? (:queue::queue::State/waiters s'))
      (:wat::core::let
        [pair (:wat::core::apply (:queue::queue::State/arm-tick s')
                 (:queue::queue::State/tick-armed? s')
                 [(:wat::core::count (:queue::queue::State/waiters s')) 1000000])
         s2 (:queue::queue::State
              :durable (:queue::queue::State/durable s')
              :store store
              :take (:queue::queue::State/take s')
              :waiters (:queue::queue::State/waiters s')
              :outbox (:queue::queue::State/outbox s')
              :receive-calls (:queue::queue::State/receive-calls s') :store-calls (:queue::queue::State/store-calls s') :store-ns (:queue::queue::State/store-ns s')
              :put-calls (:queue::queue::State/put-calls s') :put-ns (:queue::queue::State/put-ns s')
              :delete-calls (:queue::queue::State/delete-calls s') :delete-ns (:queue::queue::State/delete-ns s')
              :count-calls (:queue::queue::State/count-calls s') :count-ns (:queue::queue::State/count-ns s')
              :scan-calls (:queue::queue::State/scan-calls s') :scan-ns (:queue::queue::State/scan-ns s')
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
              :depth (:queue::queue::State/depth s') :total (:queue::queue::State/total s')
              :q-name (:queue::queue::State/q-name s')
              :tick-armed? (:wat::core::first pair)
              :arm-tick (:queue::queue::State/arm-tick s')
              :counters (:queue::queue::State/counters s')
              :seen-ids (:queue::queue::State/seen-ids s'))]
        (:wat::service::Outcome::Continue s2
          (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Accepted n-ok)))
          (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
          (:wat::core::second pair)))
      (:wat::core::let
        [wpair (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Tuple :- [:queue::TakeAcc
                                                                (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])])
                                  w   <- :queue::Waiter]
                   -> (:wat::core::Tuple :- [:queue::TakeAcc
                                             (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])])
                   (:wat::core::let
                     [ta   (:wat::core::first acc)
                      box  (:wat::core::second acc)
                      st   (:queue::TakeAcc/store ta)
                      keep (:queue::TakeAcc/keep ta)
                      taken (:queue::TakeAcc/calls ta)
                      ns-acc (:queue::TakeAcc/ns ta)
                      sc-acc (:queue::TakeAcc/scan-calls ta)
                      sn-acc (:queue::TakeAcc/scan-ns ta)
                      pc-acc (:queue::TakeAcc/put-calls ta)
                      pn-acc (:queue::TakeAcc/put-ns ta)
                      empty-ok (:queue::Queue::Reply::Receive
                                 (:queue::Queue::ReceiveResponse::Ok
                                   (:wat::core::Vector :- [:queue::Envelope])))]
                     (:wat::core::if (:wat::i64::<= (:queue::Waiter/deadline-ns w) now-ns)
                       (:wat::core::Tuple
                         (:queue::TakeAcc :store st :keep keep :calls taken :ns ns-acc :scan-calls sc-acc :scan-ns sn-acc :put-calls pc-acc :put-ns pn-acc)
                         (:wat::core::conj box
                           (:wat::service::Directed :conn-id (:queue::Waiter/conn-id w) :reply empty-ok)))
                       (:wat::core::let
                         [taken-pair (:wat::core::apply (:queue::queue::State/take s')
                                        st
                                        (:queue::Waiter/queue w)
                                        [now-ns
                                         (:queue::Waiter/visibility-ns w)
                                         (:queue::Waiter/limit w)])
                          st' (:wat::core::first taken-pair)
                          envs (:wat::core::second taken-pair)
                          take-split (:wat::core::third taken-pair)
                          take-scan-ns (:wat::core::first take-split)
                          take-put-ns (:wat::core::second take-split)
                          take-ns (:wat::i64::+ take-scan-ns take-put-ns)]
                         (:wat::core::if (:wat::core::empty? envs)
                           (:wat::core::Tuple
                             (:queue::TakeAcc :store st' :keep (:wat::vector::conj keep w) :calls (:wat::i64::+ taken 1) :ns (:wat::i64::+ ns-acc take-ns) :scan-calls (:wat::i64::+ sc-acc 1) :scan-ns (:wat::i64::+ sn-acc take-scan-ns) :put-calls pc-acc :put-ns pn-acc)
                             box)
                           (:wat::core::Tuple
                             (:queue::TakeAcc :store st' :keep keep :calls (:wat::i64::+ taken 2) :ns (:wat::i64::+ ns-acc take-ns) :scan-calls (:wat::i64::+ sc-acc 1) :scan-ns (:wat::i64::+ sn-acc take-scan-ns) :put-calls (:wat::i64::+ pc-acc 1) :put-ns (:wat::i64::+ pn-acc take-put-ns))
                             (:wat::core::conj box
                               (:wat::service::Directed
                                 :conn-id (:queue::Waiter/conn-id w)
                                 :reply (:queue::Queue::Reply::Receive
                                          (:queue::Queue::ReceiveResponse::Ok envs))))))))))
                 (:wat::core::Tuple
                   (:queue::TakeAcc
                     :store store
                     :keep (:wat::core::PersistentVector :- [:queue::Waiter])
                     :calls 0
                     :ns 0
                     :scan-calls 0
                     :scan-ns 0
                     :put-calls 0
                     :put-ns 0)
                   (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])]))
                 (:queue::queue::State/waiters s'))
         store2 (:queue::TakeAcc/store (:wat::core::first wpair))
         keep (:queue::TakeAcc/keep (:wat::core::first wpair))
         box  (:wat::core::second wpair)
         s2 (:queue::queue::State
              :durable (:queue::queue::State/durable s')
              :store store2
              :take (:queue::queue::State/take s')
              :waiters keep
              :outbox (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
              :receive-calls (:queue::queue::State/receive-calls s')
              :store-calls (:wat::i64::+ (:queue::queue::State/store-calls s') (:queue::TakeAcc/calls (:wat::core::first wpair))) :store-ns (:wat::i64::+ (:queue::queue::State/store-ns s') (:queue::TakeAcc/ns (:wat::core::first wpair)))
              :put-calls (:wat::i64::+ (:queue::queue::State/put-calls s') (:queue::TakeAcc/put-calls (:wat::core::first wpair))) :put-ns (:wat::i64::+ (:queue::queue::State/put-ns s') (:queue::TakeAcc/put-ns (:wat::core::first wpair)))
              :delete-calls (:queue::queue::State/delete-calls s') :delete-ns (:queue::queue::State/delete-ns s')
              :count-calls (:queue::queue::State/count-calls s') :count-ns (:queue::queue::State/count-ns s')
              :scan-calls (:wat::i64::+ (:queue::queue::State/scan-calls s') (:queue::TakeAcc/scan-calls (:wat::core::first wpair))) :scan-ns (:wat::i64::+ (:queue::queue::State/scan-ns s') (:queue::TakeAcc/scan-ns (:wat::core::first wpair)))
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
              :depth (:queue::queue::State/depth s') :total (:queue::queue::State/total s')
              :q-name (:queue::queue::State/q-name s')
              :tick-armed? (:queue::queue::State/tick-armed? s')
              :arm-tick (:queue::queue::State/arm-tick s')
              :counters (:queue::queue::State/counters s')
              :seen-ids (:queue::queue::State/seen-ids s'))
         pair (:wat::core::apply (:queue::queue::State/arm-tick s2)
                 (:queue::queue::State/tick-armed? s2)
                 [(:wat::core::count (:queue::queue::State/waiters s2)) 1000000])
         s3 (:queue::queue::State
              :durable (:queue::queue::State/durable s2)
              :store store2
              :take (:queue::queue::State/take s2)
              :waiters (:queue::queue::State/waiters s2)
              :outbox (:queue::queue::State/outbox s2)
              :receive-calls (:queue::queue::State/receive-calls s2) :store-calls (:queue::queue::State/store-calls s2) :store-ns (:queue::queue::State/store-ns s2)
              :put-calls (:queue::queue::State/put-calls s2) :put-ns (:queue::queue::State/put-ns s2)
              :delete-calls (:queue::queue::State/delete-calls s2) :delete-ns (:queue::queue::State/delete-ns s2)
              :count-calls (:queue::queue::State/count-calls s2) :count-ns (:queue::queue::State/count-ns s2)
              :scan-calls (:queue::queue::State/scan-calls s2) :scan-ns (:queue::queue::State/scan-ns s2)
              :handler-ns (:wat::i64::+ (:queue::queue::State/handler-ns s) (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) start-ns))
              :depth (:queue::queue::State/depth s2) :total (:queue::queue::State/total s2)
              :q-name (:queue::queue::State/q-name s2)
              :tick-armed? (:wat::core::first pair)
              :arm-tick (:queue::queue::State/arm-tick s2)
              :counters (:queue::queue::State/counters s2)
              :seen-ids (:queue::queue::State/seen-ids s2))
         ok (:wat::core::Some (:queue::Queue::Reply::Send (:queue::Queue::SendResponse::Accepted n-ok)))]
        (:wat::service::Outcome::Continue s3 ok box (:wat::core::second pair))))))

;; ── client helpers (the gate; Handle stays in the same let as the ops) ──────────
(:wat::core::defn :user::dial-queue
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])]
  -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :user::send
  [q <- :queue::Queue  name <- :wat::core::String  body <- :wat::core::String  now-ns <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:queue::Queue/send q (:queue::Queue::SendRequest :queue name :bodies (:wat::core::Vector :- [:wat::core::String] body) :now-ns now-ns))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Accepted n)
          (:wat::core::if (:wat::core::= n 1) nil
            (:wat::kernel::assertion-failed! "send not fully accepted" :wat::core::None :wat::core::None)))
        (_ (:wat::kernel::assertion-failed! "send not Accepted" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "send: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::receive
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64]
  -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::receive-wait
  [q <- :queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64  wait <- :queue::Queue::Wait]
  -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait wait))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "receive-wait not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive-wait: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::read-call-counters
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok st)
          (:wat::core::Tuple (:queue::Stats/receive-calls st) (:queue::Stats/ticks st)))
        (_ (:wat::kernel::assertion-failed! "stats not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "stats: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::read-queue-counts
  [q <- :queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok st)
          (:wat::core::Tuple (:queue::Stats/visible st) (:queue::Stats/unacked st)))
        (_ (:wat::kernel::assertion-failed! "depth not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "depth: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::ack
  [q <- :queue::Queue  name <- :wat::core::String  id <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::match (:queue::Queue/ack q (:queue::Queue::AckRequest :queue name
                                             :ids (:wat::core::Vector :- [:wat::core::String] id)))
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
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64  wait <- :queue::Queue::Wait]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:user::send-ok!
         (:wat::kernel::send c
           (:queue::Queue::Op::Receive
             (:queue::Queue::ReceiveRequest
               :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait wait))))
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
    ((:wat::kernel::RecvOutcome::Lost _cause)
      (:wat::core::Vector :- [:queue::Envelope]))
    (_ (:wat::kernel::assertion-failed! "recv-envelopes: recv failed" :wat::core::None :wat::core::None))))

;; Timer-channel recv, not a sleep — legal where mora forbids sleeping.
(:wat::core::defn :user::await-timer-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None))))

;; lifecycle against ONE store. Handle lives in this let (same-ns lesson).
(:wat::core::defn :user::lifecycle
  [store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::String
  (:wat::core::let
    [qh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 1024 :store-addr store-addr :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     vis 100
     Ta  1000000001
     Tb  1000000002
     Tr  1000000002
     Tw  1000000102
     ;; STOP-3: a message whose isk equals now must be returned (inclusive hi).
     _bx (:user::send q "bound-q" "x" T0)
     bound (:user::receive q "bound-q" T0 vis 10)
     ;; send 3, staggered by 1ns so isk order is a,b,c (limit-2 among equal isk is unspecified).
     _sa (:user::send q "q" "a" T0)
     _sb (:user::send q "q" "b" Ta)
     _sc (:user::send q "q" "c" Tb)
     r1 (:user::receive q "q" Tr vis 2)
     r2 (:user::receive q "q" Tr vis 2)
     ;; ack one of the first receive (a) and the third (c), leaving b unacked.
     ;; c is acked so redelivery is exactly the unacked one — and so equal-isk
     ;; order between b and c cannot make the backends disagree on the summary.
     _a1 (:wat::core::if (:wat::core::empty? r1) nil
            (:user::ack q "q" (:queue::Envelope/id (:wat::core::first r1))))
     _a2 (:wat::core::if (:wat::core::empty? r2) nil
            (:user::ack q "q" (:queue::Envelope/id (:wat::core::first r2))))
     r3 (:user::receive q "q" Tr vis 10)
     re (:user::receive q "q" Tw vis 10)]
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
            :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     a   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     vis 100
     _   (:user::park-receive! a "q" T0 vis 1 (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 200)))
     _   (:user::send b "q" "hello" T0)
     got (:user::recv-envelopes! a)
     again (:user::receive b "q" T0 vis 10)]
    (:wat::core::format "got={got};hidden={hidden}"
      :got (:user::join-bodies got)
      :hidden (:wat::core::if (:wat::core::empty? again) "yes" (:user::join-bodies again)))))

;; ★ row 2: parked receive times out empty; queue keeps serving.
(:wat::core::defn :user::lp-timeout [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     a   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::park-receive! a "q" T0 100 1 (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 5)))
     got (:user::recv-envelopes! a)
     ping (:user::receive b "q" T0 100 10)]
    (:wat::core::format "empty={empty};serving={serving}"
      :empty (:wat::core::if (:wat::core::empty? got) "yes" "no")
      :serving (:wat::core::if (:wat::core::empty? ping) "yes" "no"))))

;; row 8: FIFO — first parked is served.
(:wat::core::defn :user::lp-fifo [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     a   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     c   (:user::dial-queue-peer (:queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::park-receive! a "q" T0 100 1 (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 200)))
     _   (:user::park-receive! c "q" T0 100 1 (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 200)))
     _   (:user::send b "q" "first" T0)
     ga  (:user::recv-envelopes! a)
     _   (:user::send b "q" "second" T0)
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
            :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::send q "q" "a" T0)
     _   (:user::send q "q" "b" T0)
     _   (:user::send q "q" "c" T0)
     got (:user::receive-wait q "q" T0 1000000000 10 (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 20)))
     _   (:user::receive-wait q "q" T0 1000000000 10 (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 5)))
     st  (:user::read-call-counters q)]
    (:wat::core::format "n={n};calls={calls}"
      :n (:wat::core::count got)
      :calls (:wat::core::first st))))

;; row 6: idle queue never ticks.
(:wat::core::defn :user::lp-idle [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:user::dial-queue (:queue::queue::Handle/addr qh))
     _   (:user::await-timer-ms 20)
     st  (:user::read-call-counters q)]
    (:wat::core::format "ticks={ticks}"
      :ticks (:wat::core::second st))))

;; depth is derived: visible = |isk <= now|, unacked = total - visible.
(:wat::core::defn :user::depth [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:user::dial-queue (:queue::queue::Handle/addr qh))
     T0  (:wat::time::epoch-nanos (:wat::time::now))
     vis 1000000000000
     _   (:user::send q "q" "a" T0)
     _   (:user::send q "q" "b" T0)
     _   (:user::send q "q" "c" T0)
     d0  (:user::read-queue-counts q)
     r   (:user::receive q "q" T0 vis 2)
     d1  (:user::read-queue-counts q)
     _   (:user::ack q "q" (:queue::Envelope/id (:wat::core::first r)))
     d2  (:user::read-queue-counts q)]
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
