;; wat-tests/std/service-template.wat — the canonical in-memory
;; request/reply service program in wat.
;;
;; This file is a runnable reference. It exercises every pattern that
;; a service program built on the substrate's channel + spawn primitives
;; will reach for. Lift directly when starting your own service; the
;; only thing that should change is the State struct (your domain) and
;; the Request enum's verbs (your operations).
;;
;; SERVICE-PROGRAMS.md § "The complete pattern" walks this file as
;; the canonical synthesis of the eight building-block steps.
;;
;; Three reply shapes — every request is one of these:
;;   Push(value)    — fire-and-forget; no reply channel
;;   Ack(reply-tx)  — confirm-receipt; reply is unit
;;   Get(reply-tx)  — read-only query; reply is the domain state
;;
;; Driver pattern (post-arc-114 mini-TCP):
;;   - select over Vec<ReqRx>; one rx per provisioned client handle
;;   - on Some(req): per-variant handler returns NEW state
;;   - on :None: prune that rx, recurse with state unchanged
;;   - exit when Vec is empty; SEND final state on the substrate's
;;     `out` Sender; return unit
;;
;; Caller pattern:
;;   - outer scope holds the Thread<(),State>
;;   - inner scope owns popped sender(s) + per-call reply channels
;;   - inner exits → all client Senders drop → driver's last rx
;;     disconnects → loop sends final state on `out` → returns unit
;;   - outer recvs final state via Thread/output thr
;;   - outer Thread/join-result thr — confirms clean exit
;;
;; Why this is mini-TCP-shaped: arc 114 retired R-via-join. The thread
;; contract is "input pipe in, output pipe out, error via join chain."
;; Service drivers have N request channels (per-client) + N per-request
;; reply channels — those are mini-TCP at the per-REQUEST level. The
;; substrate-allocated `out` carries the per-THREAD output (final
;; state on graceful exit); the substrate-allocated `_in` is currently
;; unused (a future arc may wire it to a shutdown signal).
;;
;; Namespace `:svc::*` is generic — when you fork this, swap the
;; namespace for your domain (`:my::accountant::*`, `:my::registry::*`)
;; and rename the verbs to your operations.

(:wat::test::make-deftest :deftest
  (;; State — the domain accumulator the loop carries between
   ;; iterations. Each handler returns a NEW state (values discipline;
   ;; never mutate in place). Two counter fields here demonstrate the
   ;; pattern; in your service, these are your real domain fields
   ;; (an LRU map, a treasury record, a registry table, etc.).
   (:wat::core::struct :svc::State
     (push-count :wat::core::i64)
     (ack-count  :wat::core::i64))

   (:wat::core::define
     (:svc::State::fresh -> :svc::State)
     (:svc::State/new 0 0))

   ;; Reply channel for the Ack verb — unit reply. Aliased because it
   ;; recurs at every Ack call site (request body + caller's reply
   ;; channel both reference it).
   (:wat::core::typealias :svc::AckReplyTx :wat::kernel::QueueSender<wat::core::unit>)
   (:wat::core::typealias :svc::AckReplyRx :wat::kernel::QueueReceiver<wat::core::unit>)

   ;; Get's reply channel carries the full State struct. Inlined in
   ;; the variant declaration below — domain-payload reply types are
   ;; per-verb and don't tend to repeat outside the variant they belong
   ;; to (one alias per such variant adds noise, not signal).

   ;; Request — three reply shapes side by side. Every in-memory
   ;; request/reply service is some combination of these.
   (:wat::core::enum :svc::Request
     (Push (value :wat::core::i64))
     (Ack  (reply-tx :svc::AckReplyTx))
     (Get  (reply-tx :wat::kernel::QueueSender<svc::State>)))

   ;; Per-broker request channel typealiases. Idiomatic: every service
   ;; has these four (Tx / Rx / TxPool / Spawn) — they describe the
   ;; service's wire shape independent of state or verbs.
   (:wat::core::typealias :svc::ReqTx :wat::kernel::QueueSender<svc::Request>)
   (:wat::core::typealias :svc::ReqRx :wat::kernel::QueueReceiver<svc::Request>)
   (:wat::core::typealias :svc::ReqTxPool :wat::kernel::HandlePool<svc::ReqTx>)
   ;; Arc 114 — the spawn shape's second member is now Thread<(),State>
   ;; instead of ProgramHandle<State>. Final-state delivery is via the
   ;; thread's output Sender; the Thread/join-result confirms exit.
   (:wat::core::typealias :svc::Spawn
     :(svc::ReqTxPool,wat::kernel::Thread<wat::core::unit,svc::State>))

   ;; The substrate-allocated `_in` Receiver is unused by the service
   ;; driver (the service's request channels are the real inputs).
   ;; Aliased for clarity at the spawn-thread call site.
   (:wat::core::typealias :svc::DriverIn  :rust::crossbeam_channel::Receiver<wat::core::unit>)
   (:wat::core::typealias :svc::DriverOut :rust::crossbeam_channel::Sender<svc::State>)


   ;; ─── Per-variant dispatch ─────────────────────────────────────
   ;;
   ;; Each arm: read state via accessors, do the verb's work, return
   ;; a new state (or the same state for read-only verbs).
   (:wat::core::define
     (:svc::Service/handle
       (req :svc::Request)
       (state :svc::State)
       -> :svc::State)
     (:wat::core::match req -> :svc::State
       ;; Push — fire-and-forget. Bump push-count, no reply.
       ;; The _value param ignored here; in your service it'd feed
       ;; into state computation.
       ((:svc::Request::Push _value)
         (:svc::State/new
           (:wat::core::+ (:svc::State/push-count state) 1)
           (:svc::State/ack-count state)))

       ;; Ack — confirm-receipt. Bump ack-count, send unit reply.
       ;; Per arc 110: client dropping its reply-rx mid-protocol is a
       ;; protocol violation; expect makes the disconnect a panic so
       ;; the program tree learns the breakage instead of silent drop.
       ((:svc::Request::Ack reply-tx)
         (:wat::core::let*
           (((_ack :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send reply-tx ())
               "Service/handle Ack: reply-tx disconnected — caller died?")))
           (:svc::State/new
             (:svc::State/push-count state)
             (:wat::core::+ (:svc::State/ack-count state) 1))))

       ;; Get — read-only query. Send current state through reply-tx,
       ;; return state UNCHANGED. No counters bumped (a read should
       ;; not look like a mutation).
       ((:svc::Request::Get reply-tx)
         (:wat::core::let*
           (((_send :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send reply-tx state)
               "Service/handle Get: reply-tx disconnected — caller died?")))
           state))))


   ;; ─── Service driver loop ─────────────────────────────────────
   ;;
   ;; select over Vec<ReqRx>; on Some(req) dispatch via /handle and
   ;; carry the new state forward; on :None for any rx, prune that
   ;; channel and recurse with state unchanged; exit when Vec is empty
   ;; (all client scopes have exited). On exit, send the final state
   ;; on the thread's output Sender — that's the post-arc-114
   ;; final-state-delivery channel; the parent recv's it before
   ;; calling Thread/join-result.
   (:wat::core::define
     (:svc::Service/loop
       (req-rxs :wat::core::Vector<svc::ReqRx>)
       (state :svc::State)
       (out :svc::DriverOut)
       -> :wat::core::unit)
     (:wat::core::if (:wat::core::empty? req-rxs) -> :wat::core::unit
       ;; Empty — every client gone. Deliver final state via `out`.
       ;; `expect` panics if the parent dropped its Receiver — that's
       ;; a substrate-tree breakage worth surfacing, not silently
       ;; eating.
       (:wat::core::result::expect -> :wat::core::unit
         (:wat::kernel::send out state)
         "Service/loop: out disconnected — parent dropped Thread/output before recv?")
       (:wat::core::let*
         (((chosen :wat::kernel::Chosen<svc::Request>) (:wat::kernel::select req-rxs))
          ((idx :wat::core::i64) (:wat::core::first chosen))
          ((maybe :wat::kernel::CommResult<svc::Request>) (:wat::core::second chosen)))
         (:wat::core::match maybe -> :wat::core::unit
           ((Ok (:wat::core::Some req))
             (:wat::core::let*
               (((next :svc::State) (:svc::Service/handle req state)))
               (:svc::Service/loop req-rxs next out)))
           ((Ok :wat::core::None)
             (:svc::Service/loop (:wat::std::list::remove-at req-rxs idx) state out))
           ((Err _died)
             (:svc::Service/loop (:wat::std::list::remove-at req-rxs idx) state out))))))


   ;; ─── Service constructor ─────────────────────────────────────
   ;;
   ;; Build N request channels, pool the senders (orphan detector at
   ;; construction), spawn the driver lambda — the lambda closes over
   ;; req-rxs + initial state and forwards the substrate's `out`
   ;; Sender to Service/loop. Returns (pool, thread).
   ;;
   ;; Caller does:
   ;;   ((spawn :svc::Spawn) (:svc::Service N))
   ;;   ((pool ...) (:wat::core::first spawn))
   ;;   ((thr ...) (:wat::core::second spawn))
   ;;   <inner scope: pop N handles, finish, do work, exit>
   ;;   ((final :svc::State) recv on Thread/output thr)
   ;;   (:wat::kernel::Thread/join-result thr)  ; confirms clean exit
   (:wat::core::define
     (:svc::Service (count :wat::core::i64) -> :svc::Spawn)
     (:wat::core::let*
       (((pairs :wat::core::Vector<wat::kernel::QueuePair<svc::Request>>)
         (:wat::core::map
           (:wat::core::range 0 count)
           (:wat::core::lambda ((_i :wat::core::i64) -> :wat::kernel::QueuePair<svc::Request>)
             (:wat::kernel::make-bounded-queue :svc::Request 1))))

        ((req-txs :wat::core::Vector<svc::ReqTx>)
         (:wat::core::map pairs
           (:wat::core::lambda ((p :wat::kernel::QueuePair<svc::Request>) -> :svc::ReqTx)
             (:wat::core::first p))))

        ((req-rxs :wat::core::Vector<svc::ReqRx>)
         (:wat::core::map pairs
           (:wat::core::lambda ((p :wat::kernel::QueuePair<svc::Request>) -> :svc::ReqRx)
             (:wat::core::second p))))

        ((pool :svc::ReqTxPool)
         (:wat::kernel::HandlePool::new "svc-template" req-txs))

        ((thr :wat::kernel::Thread<wat::core::unit,svc::State>)
         (:wat::kernel::spawn-thread
           (:wat::core::lambda
             ((_in :svc::DriverIn)
              (out :svc::DriverOut)
              -> :wat::core::unit)
             (:svc::Service/loop req-rxs (:svc::State::fresh) out)))))
       (:wat::core::Tuple pool thr)))))


;; ─── Test — exercise all three reply shapes + state survives ──
;;
;; Drives a known sequence (2 Pushes + 1 Ack + 1 Get + 1 Push + 1
;; Get), verifies each Get reads LIVE state, then verifies the
;; FINAL state via the thread's output channel (post-arc-114
;; pattern — final state delivered on `out`, not via join's R).
(:deftest :svc::test-template-end-to-end
     (:wat::core::let*
       ;; Outer holds only the Thread (and the final-rx Receiver
       ;; cloned from it). Inner owns the spawn-tuple + pool + every
       ;; per-request channel; inner returns the Thread; pool drops
       ;; at inner exit. Arc 117 enforces this nesting.
       (((thr :wat::kernel::Thread<wat::core::unit,svc::State>)
         (:wat::core::let*
           (((spawn :svc::Spawn) (:svc::Service 1))
            ((pool :svc::ReqTxPool) (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,svc::State>) (:wat::core::second spawn))
            ((req-tx :svc::ReqTx) (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
            ((ack-pair :wat::kernel::QueuePair<wat::core::unit>)
             (:wat::kernel::make-bounded-queue :wat::core::unit 1))
            ((ack-tx :svc::AckReplyTx) (:wat::core::first ack-pair))
            ((ack-rx :svc::AckReplyRx) (:wat::core::second ack-pair))
            ((get-pair :wat::kernel::QueuePair<svc::State>)
             (:wat::kernel::make-bounded-queue :svc::State 1))
            ((get-tx :wat::kernel::QueueSender<svc::State>) (:wat::core::first get-pair))
            ((get-rx :wat::kernel::QueueReceiver<svc::State>) (:wat::core::second get-pair))
            ((_p1 :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Push 100))
               "test send Push 100: req-tx disconnected — driver died?"))
            ((_p2 :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Push 200))
               "test send Push 200: req-tx disconnected — driver died?"))
            ((_a :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Ack ack-tx))
               "test send Ack: req-tx disconnected — driver died?"))
            ((_r :wat::core::unit)
             (:wat::core::option::expect -> :wat::core::unit
               (:wat::core::result::expect -> :wat::core::Option<wat::core::unit>
                 (:wat::kernel::recv ack-rx)
                 "test recv ack: ack-rx peer thread died")
               "test recv ack: ack-rx clean disconnect — driver died mid-Ack?"))
            ((_g1 :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Get get-tx))
               "test send Get #1: req-tx disconnected — driver died?"))
            ((snap1 :svc::State)
             (:wat::core::option::expect -> :svc::State
               (:wat::core::result::expect -> :wat::core::Option<svc::State>
                 (:wat::kernel::recv get-rx)
                 "test recv get #1: peer thread died")
               "test recv get #1: clean disconnect — driver died mid-Get?"))
            ((_check1a :wat::core::unit)
             (:wat::core::if (:wat::core::= (:svc::State/push-count snap1) 2) -> :wat::core::unit
               ()
               (:wat::test::assert-eq "snap1 push != 2" "")))
            ((_check1b :wat::core::unit)
             (:wat::core::if (:wat::core::= (:svc::State/ack-count snap1) 1) -> :wat::core::unit
               ()
               (:wat::test::assert-eq "snap1 ack != 1" "")))
            ((_p3 :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Push 300))
               "test send Push 300: req-tx disconnected — driver died?"))
            ((_g2 :wat::core::unit)
             (:wat::core::result::expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Get get-tx))
               "test send Get #2: req-tx disconnected — driver died?"))
            ((snap2 :svc::State)
             (:wat::core::option::expect -> :svc::State
               (:wat::core::result::expect -> :wat::core::Option<svc::State>
                 (:wat::kernel::recv get-rx)
                 "test recv get #2: peer thread died")
               "test recv get #2: clean disconnect — driver died mid-Get?"))
            ((_check2a :wat::core::unit)
             (:wat::core::if (:wat::core::= (:svc::State/push-count snap2) 3) -> :wat::core::unit
               ()
               (:wat::test::assert-eq "snap2 push != 3" "")))
            ((_check2b :wat::core::unit)
             (:wat::core::if (:wat::core::= (:svc::State/ack-count snap2) 1) -> :wat::core::unit
               ()
               (:wat::test::assert-eq "snap2 ack != 1" ""))))
           d))
        ((final-rx :rust::crossbeam_channel::Receiver<svc::State>)
         (:wat::kernel::Thread/output thr))
        ;; inner scope dropped pool + req-tx + reply channels; driver's
        ;; last rx disconnects; Service/loop's empty-Vec arm sends final
        ;; state on `out`. Recv it here.
        ((final-state :svc::State)
         (:wat::core::option::expect -> :svc::State
           (:wat::core::result::expect -> :wat::core::Option<svc::State>
             (:wat::kernel::recv final-rx)
             "test recv final-state: thread died before delivering final state")
           "test recv final-state: thread output closed without delivering final state"))
        ((join-result :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result thr)))
       (:wat::core::match join-result -> :wat::core::unit
         ((Ok _)
           (:wat::core::let*
             (((pc :wat::core::i64) (:svc::State/push-count final-state))
              ((ac :wat::core::i64) (:svc::State/ack-count final-state))
              ((_check-pc :wat::core::unit)
               (:wat::core::if (:wat::core::= pc 3) -> :wat::core::unit
                 ()
                 (:wat::test::assert-eq "final push != 3" ""))))
             (:wat::core::if (:wat::core::= ac 1) -> :wat::core::unit
               ()
               (:wat::test::assert-eq "final ack != 1" ""))))
         ((Err _) (:wat::test::assert-eq "driver-died" "")))))
