;; wat-scripts/queue/sqs.wat — the DRIVER half of wat-queue: the differential lifecycle,
;; the five long-poll rows, the depth rows, and the ergonomic client gate they call.
;;
;; ⛔ THE LIBRARY IS GONE FROM HERE. The `:wat::queue::Queue` surface and the
;;    `:wat::queue::queue` service were promoted to the stdlib at wat/queue.wat (manifest
;;    position 50) by excursus 001's "the queue matures into the stdlib". They arrive with
;;    the binary now, so NO program needs a `load-file!` of this file to reach the queue —
;;    16 such lines were deleted by wat-scripts/fixes/drop-sqs-load-file.wat. The only two
;;    files that still load this one want `:user::dial-queue` below, which is a different
;;    thing entirely. See wat/queue.wat's header for the design.
;;
;; WHAT STAYED, AND WHY. All 23 `:user::` names stayed. Each is one of three things, and none of
;; the three is library. ⚠ The DESIGN called 13 of them CLIENT API; that reading is rejected here,
;; with reasons — see the SCORE's row 9.
;;
;;   THE PANIC GATE  dial-queue · dial-queue-peer · send · receive · receive-wait · ack ·
;;                   send-ok! · park-receive! · recv-envelopes! · read-call-counters ·
;;                   read-queue-counts
;;     Every non-happy arm of each of these is `assertion-failed!`. That is a test fixture,
;;     not a client API: the client API is the surface's own generated methods
;;     (`:wat::queue::Queue/send` · `/receive` · `/ack` · `/stats`), which hand back a
;;     `RecvOutcome` the caller must face. The measurement that settles it: the two real
;;     consumers — wat-scripts/fanout/circuit.wat (189 `:wat::queue::` references) and
;;     wat-scripts/topic/sns-fanout.wat (194) — call ZERO of these helpers. They wrote their
;;     own outcome-facing versions instead. Promoting a panicking gate into the stdlib would
;;     enshrine, at manifest position 50, exactly the ungraceful failure this excursus is a
;;     crusade against.
;;
;;   NOT LIBRARY, FOR A DIFFERENT REASON  await-timer-ms (a generic timer-channel recv — the word
;;     "queue" does not appear in it; if it earns promotion it belongs in wat/time.wat, not here) ·
;;     join-bodies (does take Envelopes, but what it produces is an ASSERTION STRING — the
;;     comma-joined bodies the rows below compare against; it is the test's formatter, not a queue
;;     verb). Both are called only by a driver in this file.
;;
;;   THE DRIVERS  lp-send-wakes · lp-timeout · lp-fifo · lp-fewer-receives · lp-idle ·
;;     lifecycle · depth · long-poll · compute · main. These pick a concrete store backend
;;     (`:wat::query::mem-store`, manifest 51; `:wat::query::sqlite-store`, 52), which a file
;;     at position 50 may not name even if it wanted to.
;;
;; ★ THE CLOCK IS AN ARGUMENT — see wat/queue.wat. `now-ns` is a value, so the rows below
;;   drive the visibility window without a wall-clock wait, and `mora`'s no-sleep rule holds.
;;
;; `:user::compute` runs the full lifecycle against mem-store AND sqlite-store
;; (:memory:, :index-names ["by-visible-at"]) and returns the agreed summary
;; (or DIFFERENTIAL-MISMATCH). `:user::main` prints it. Shape copied from
;; tests/services/probe_ex001_journal_same_ns.wat and wat-scripts/topic/sns-fanout.wat.
;;
;; The five rust probes that rendezvous with this half:
;;   probe_ex001_queue.rs      startup_from_file(this file)          -> :user::compute
;;   probe_queue_long_poll.rs  startup_from_file(this file)          -> :user::long-poll
;;   probe_queue_depth.rs      startup_from_file(this file)          -> :user::depth
;;   probe_queue_visibility.rs scratch-pad/probe-visibility-redelivers.wat -> its OWN :user::compute
;;   probe_async_publish.rs    topic/sns-fanout.wat + fanout/circuit.wat   -> their OWN gates
;; The last two reach the QUEUE, not this file's drivers; they needed the FsLoader only
;; because their entry program used to `load-file!` this file for the library. It does not
;; any more — the library is in the binary.

;; ── client helpers (the gate; Handle stays in the same let as the ops) ──────────
(:wat::core::defn :user::dial-queue
  [a <- (:wat::kernel::Address :- [:wat::queue::Queue::Op :wat::queue::Queue::Reply])]
  -> :wat::queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :user::send
  [q <- :wat::queue::Queue  name <- :wat::core::String  body <- :wat::core::String  now-ns <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:wat::queue::Queue/send q (:wat::queue::Queue::SendRequest :queue name :bodies (:wat::core::Vector :- [:wat::core::String] body) :now-ns now-ns))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::queue::Queue::SendResponse::Accepted n)
          (:wat::core::if (:wat::core::= n 1) nil
            (:wat::kernel::assertion-failed! "send not fully accepted" :wat::core::None :wat::core::None)))
        (_ (:wat::kernel::assertion-failed! "send not Accepted" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "send: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::receive
  [q <- :wat::queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::queue::Envelope])
  (:wat::core::match
    (:wat::queue::Queue/receive q
      (:wat::queue::Queue::ReceiveRequest :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait (:wat::queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::receive-wait
  [q <- :wat::queue::Queue  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64  wait <- :wat::queue::Queue::Wait]
  -> (:wat::core::Vector :- [:wat::queue::Envelope])
  (:wat::core::match
    (:wat::queue::Queue/receive q
      (:wat::queue::Queue::ReceiveRequest :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait wait))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "receive-wait not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "receive-wait: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::read-call-counters
  [q <- :wat::queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:wat::queue::Queue/stats q (:wat::queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::queue::Queue::StatsResponse::Ok st)
          (:wat::core::Tuple (:wat::queue::Stats/receive-calls st) (:wat::queue::Stats/ticks st)))
        (_ (:wat::kernel::assertion-failed! "stats not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "stats: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::read-queue-counts
  [q <- :wat::queue::Queue] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:wat::queue::Queue/stats q (:wat::queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::queue::Queue::StatsResponse::Ok st)
          (:wat::core::Tuple (:wat::queue::Stats/visible st) (:wat::queue::Stats/unacked st)))
        (_ (:wat::kernel::assertion-failed! "depth not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "depth: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::ack
  [q <- :wat::queue::Queue  name <- :wat::core::String  id <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::match (:wat::queue::Queue/ack q (:wat::queue::Queue::AckRequest :queue name
                                             :ids (:wat::core::Vector :- [:wat::core::String] id)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::queue::Queue::AckResponse::Ok) nil)
        (_ (:wat::kernel::assertion-failed! "ack not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "ack: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::join-bodies
  [envs <- (:wat::core::Vector :- [:wat::queue::Envelope])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String e <- :wat::queue::Envelope] -> :wat::core::String
      (:wat::core::let [b (:wat::queue::Envelope/body e)]
        (:wat::core::if (:wat::core::= acc "")
          b
          (:wat::string::concat acc (:wat::string::concat "," b)))))
    ""
    envs))

(:wat::core::defn :user::dial-queue-peer
  [a <- (:wat::kernel::Address :- [:wat::queue::Queue::Op :wat::queue::Queue::Reply])]
  -> (:wat::kernel::Peer :- [:wat::queue::Queue::Op :wat::queue::Queue::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; ⛔ ALL FOUR ARMS NAMED, AND THREE WORLDS SAY THREE THINGS. This was
;; `(_ (assertion-failed! "send-ok: not Sent" …))`: one wildcard standing for Closed,
;; Lost and Stopped, so the message could not tell a caller which had happened — and a
;; variant ADDED to `SendOutcome` later (the `TimedOut` of
;; `a-send-cannot-say-it-is-blocked`) would have been absorbed here too, at the only
;; send-facing helper in this file (both of `park-receive!`'s sends go through it).
;; `Lost` keeps its cause; the other two name themselves.
(:wat::core::defn :user::send-ok!
  [st <- :wat::kernel::SendOutcome] -> :wat::core::nil
  (:wat::core::match st
    (:wat::kernel::SendOutcome::Sent nil)
    (:wat::kernel::SendOutcome::Closed
      (:wat::kernel::assertion-failed! "send-ok: the service peer closed — the send never landed" :wat::core::None :wat::core::None))
    (:wat::kernel::SendOutcome::Stopped
      (:wat::kernel::assertion-failed! "send-ok: a stop was in force — the send never landed" :wat::core::None :wat::core::None))
    ((:wat::kernel::SendOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))))

(:wat::core::defn :user::park-receive!
  [c <- (:wat::kernel::Peer :- [:wat::queue::Queue::Op :wat::queue::Queue::Reply])  name <- :wat::core::String  now-ns <- :wat::core::i64
   vis-ns <- :wat::core::i64  lim <- :wat::core::i64  wait <- :wat::queue::Queue::Wait]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:user::send-ok!
         (:wat::kernel::send c
           (:wat::queue::Queue::Op::Receive
             (:wat::queue::Queue::ReceiveRequest
               :queue name :now-ns now-ns :visibility-ns vis-ns :limit lim :wait wait))))
     _st (:user::send-ok!
           (:wat::kernel::send c (:wat::queue::Queue::Op::Stats (:wat::queue::Queue::StatsRequest))))
     _   (:wat::core::match (:wat::kernel::recv c)
           ((:wat::kernel::RecvOutcome::Message recvd)
             (:wat::core::match recvd
               ((:wat::queue::Queue::Reply::Stats _s) nil)
               (_ (:wat::kernel::assertion-failed! "park-receive: expected Stats reply as barrier" :wat::core::None :wat::core::None))))
           (_ (:wat::kernel::assertion-failed! "park-receive: stats barrier recv failed" :wat::core::None :wat::core::None)))]
    nil))

(:wat::core::defn :user::recv-envelopes!
  [c <- (:wat::kernel::Peer :- [:wat::queue::Queue::Op :wat::queue::Queue::Reply])] -> (:wat::core::Vector :- [:wat::queue::Envelope])
  (:wat::core::match (:wat::kernel::recv c)
    ((:wat::kernel::RecvOutcome::Message recvd)
      (:wat::core::match recvd
        ((:wat::queue::Queue::Reply::Receive resp)
          (:wat::core::match resp
            ((:wat::queue::Queue::ReceiveResponse::Ok envs) envs)
            (_ (:wat::kernel::assertion-failed! "recv-envelopes: not Ok" :wat::core::None :wat::core::None))))
        (_ (:wat::kernel::assertion-failed! "recv-envelopes: expected Reply::Receive" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost _cause)
      (:wat::core::Vector :- [:wat::queue::Envelope]))
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
    [qh (:wat::queue::queue/start :locus (:wat::spawn::thread)
           :record (:wat::queue::queue::Record :cap 1024 :store-addr store-addr :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:user::dial-queue (:wat::queue::queue::Handle/addr qh))
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
            (:user::ack q "q" (:wat::queue::Envelope/id (:wat::core::first r1))))
     _a2 (:wat::core::if (:wat::core::empty? r2) nil
            (:user::ack q "q" (:wat::queue::Envelope/id (:wat::core::first r2))))
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
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
            :record (:wat::queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     a   (:user::dial-queue-peer (:wat::queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:wat::queue::queue::Handle/addr qh))
     T0  1000000000
     vis 100
     _   (:user::park-receive! a "q" T0 vis 1 (:wat::queue::Queue::Wait::UpTo (:wat::time::Milliseconds 200)))
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
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
            :record (:wat::queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     a   (:user::dial-queue-peer (:wat::queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:wat::queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::park-receive! a "q" T0 100 1 (:wat::queue::Queue::Wait::UpTo (:wat::time::Milliseconds 5)))
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
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
            :record (:wat::queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     a   (:user::dial-queue-peer (:wat::queue::queue::Handle/addr qh))
     c   (:user::dial-queue-peer (:wat::queue::queue::Handle/addr qh))
     b   (:user::dial-queue (:wat::queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::park-receive! a "q" T0 100 1 (:wat::queue::Queue::Wait::UpTo (:wat::time::Milliseconds 200)))
     _   (:user::park-receive! c "q" T0 100 1 (:wat::queue::Queue::Wait::UpTo (:wat::time::Milliseconds 200)))
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
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
            :record (:wat::queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:user::dial-queue (:wat::queue::queue::Handle/addr qh))
     T0  1000000000
     _   (:user::send q "q" "a" T0)
     _   (:user::send q "q" "b" T0)
     _   (:user::send q "q" "c" T0)
     got (:user::receive-wait q "q" T0 1000000000 10 (:wat::queue::Queue::Wait::UpTo (:wat::time::Milliseconds 20)))
     _   (:user::receive-wait q "q" T0 1000000000 10 (:wat::queue::Queue::Wait::UpTo (:wat::time::Milliseconds 5)))
     st  (:user::read-call-counters q)]
    (:wat::core::format "n={n};calls={calls}"
      :n (:wat::core::count got)
      :calls (:wat::core::first st))))

;; row 6: idle queue never ticks.
(:wat::core::defn :user::lp-idle [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
            :record (:wat::queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:user::dial-queue (:wat::queue::queue::Handle/addr qh))
     _   (:user::await-timer-ms 20)
     st  (:user::read-call-counters q)]
    (:wat::core::format "ticks={ticks}"
      :ticks (:wat::core::second st))))

;; depth is derived: visible = |isk <= now|, unacked = total - visible.
(:wat::core::defn :user::depth [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
            :record (:wat::queue::queue::Record :cap 1024 :store-addr (:wat::query::mem-store::Handle/addr msh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:user::dial-queue (:wat::queue::queue::Handle/addr qh))
     T0  (:wat::time::epoch-nanos (:wat::time::now))
     vis 1000000000000
     _   (:user::send q "q" "a" T0)
     _   (:user::send q "q" "b" T0)
     _   (:user::send q "q" "c" T0)
     d0  (:user::read-queue-counts q)
     r   (:user::receive q "q" T0 vis 2)
     d1  (:user::read-queue-counts q)
     _   (:user::ack q "q" (:wat::queue::Envelope/id (:wat::core::first r)))
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
