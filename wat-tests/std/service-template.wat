;; ARC 114 MANUAL — needs type-design review
;; This template's driver returns the final State via join (R-via-join
;; ferrying); arc 114 retires that contract. The state must travel out
;; via a channel — either Get-on-shutdown or a dedicated final-state
;; reply pipe. Pattern + the test exercising it are bigger than an
;; auto-sweep can address; surfacing for human design.
;;
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
;; Driver pattern:
;;   - select over Vec<ReqRx>; one rx per provisioned client handle
;;   - on Some(req): per-variant handler returns NEW state
;;   - on :None: prune that rx, recurse with state unchanged
;;   - exit when Vec is empty; return final state through join
;;
;; Caller pattern:
;;   - outer scope holds the driver ProgramHandle
;;   - inner scope owns popped sender(s) + per-call reply channels
;;   - inner exits → all client Senders drop → driver's last rx
;;     disconnects → loop returns final state → outer join unblocks
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
   (:wat::core::typealias :svc::AckReplyTx :wat::kernel::QueueSender<()>)
   (:wat::core::typealias :svc::AckReplyRx :wat::kernel::QueueReceiver<()>)

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
   (:wat::core::typealias :svc::Spawn
     :(svc::ReqTxPool,wat::kernel::ProgramHandle<svc::State>))


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
           (((_ack :())
             (:wat::core::result::expect -> :()
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
           (((_send :())
             (:wat::core::result::expect -> :()
               (:wat::kernel::send reply-tx state)
               "Service/handle Get: reply-tx disconnected — caller died?")))
           state))))


   ;; ─── Service driver loop ─────────────────────────────────────
   ;;
   ;; select over Vec<ReqRx>; on Some(req) dispatch via /handle and
   ;; carry the new state forward; on :None for any rx, prune that
   ;; channel and recurse with state unchanged; exit when Vec is empty
   ;; (all client scopes have exited). Return the final state — it
   ;; rides through the spawn-thread's return value to join-result.
   (:wat::core::define
     (:svc::Service/loop
       (req-rxs :Vec<svc::ReqRx>)
       (state :svc::State)
       -> :svc::State)
     (:wat::core::if (:wat::core::empty? req-rxs) -> :svc::State
       state
       (:wat::core::let*
         (((chosen :wat::kernel::Chosen<svc::Request>) (:wat::kernel::select req-rxs))
          ((idx :wat::core::i64) (:wat::core::first chosen))
          ((maybe :wat::kernel::CommResult<svc::Request>) (:wat::core::second chosen)))
         (:wat::core::match maybe -> :svc::State
           ((Ok (Some req))
             (:wat::core::let*
               (((next :svc::State) (:svc::Service/handle req state)))
               (:svc::Service/loop req-rxs next)))
           ((Ok :None)
             (:svc::Service/loop (:wat::std::list::remove-at req-rxs idx) state))
           ((Err _died)
             (:svc::Service/loop (:wat::std::list::remove-at req-rxs idx) state))))))


   ;; ─── Service constructor ─────────────────────────────────────
   ;;
   ;; Build N request channels, pool the senders (orphan detector at
   ;; construction), spawn the driver with the receivers Vec and a
   ;; fresh state, return (pool, driver).
   ;;
   ;; Caller does:
   ;;   ((spawn :svc::Spawn) (:svc::Service N))
   ;;   ((pool ...) (:wat::core::first spawn))
   ;;   ((driver ...) (:wat::core::second spawn))
   ;;   <inner scope: pop N handles, finish, do work, exit>
   ;;   (:wat::kernel::join driver)
   (:wat::core::define
     (:svc::Service (count :wat::core::i64) -> :svc::Spawn)
     (:wat::core::let*
       (((pairs :Vec<wat::kernel::QueuePair<svc::Request>>)
         (:wat::core::map
           (:wat::core::range 0 count)
           (:wat::core::lambda ((_i :wat::core::i64) -> :wat::kernel::QueuePair<svc::Request>)
             (:wat::kernel::make-bounded-queue :svc::Request 1))))

        ((req-txs :Vec<svc::ReqTx>)
         (:wat::core::map pairs
           (:wat::core::lambda ((p :wat::kernel::QueuePair<svc::Request>) -> :svc::ReqTx)
             (:wat::core::first p))))

        ((req-rxs :Vec<svc::ReqRx>)
         (:wat::core::map pairs
           (:wat::core::lambda ((p :wat::kernel::QueuePair<svc::Request>) -> :svc::ReqRx)
             (:wat::core::second p))))

        ((pool :svc::ReqTxPool)
         (:wat::kernel::HandlePool::new "svc-template" req-txs))

        ((driver :wat::kernel::ProgramHandle<svc::State>)
         (:wat::kernel::spawn :svc::Service/loop req-rxs (:svc::State::fresh))))
       (:wat::core::tuple pool driver)))))


;; ─── Test — exercise all three reply shapes + state survives ──
;;
;; Drive a known sequence (2 Pushes + 1 Ack + 1 Get + 1 Push + 1 Get),
;; verify each Get reads LIVE state, verify the final state via
;; join-result. One deftest that demonstrates the full template.

(:deftest :svc::test-template-end-to-end
  (:wat::core::let*
    (((spawn :svc::Spawn) (:svc::Service 1))
     ((pool :svc::ReqTxPool) (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<svc::State>) (:wat::core::second spawn))

     ((_inner :())
      (:wat::core::let*
        (((req-tx :svc::ReqTx) (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))

         ;; Ack reply channel.
         ((ack-pair :wat::kernel::QueuePair<()>)
          (:wat::kernel::make-bounded-queue :() 1))
         ((ack-tx :svc::AckReplyTx) (:wat::core::first ack-pair))
         ((ack-rx :svc::AckReplyRx) (:wat::core::second ack-pair))

         ;; Get reply channel — carries the full State.
         ((get-pair :wat::kernel::QueuePair<svc::State>)
          (:wat::kernel::make-bounded-queue :svc::State 1))
         ((get-tx :wat::kernel::QueueSender<svc::State>) (:wat::core::first get-pair))
         ((get-rx :wat::kernel::QueueReceiver<svc::State>) (:wat::core::second get-pair))

         ;; 2 Pushes — fire and forget. Per arc 110, every send wraps
         ;; in option::expect: in-memory peer-death is catastrophic;
         ;; the service driver dying mid-test must surface, not hang.
         ((_p1 :())
          (:wat::core::result::expect -> :()
            (:wat::kernel::send req-tx (:svc::Request::Push 100))
            "test send Push 100: req-tx disconnected — driver died?"))
         ((_p2 :())
          (:wat::core::result::expect -> :()
            (:wat::kernel::send req-tx (:svc::Request::Push 200))
            "test send Push 200: req-tx disconnected — driver died?"))

         ;; 1 Ack — confirm-receipt round-trip.
         ((_a :())
          (:wat::core::result::expect -> :()
            (:wat::kernel::send req-tx (:svc::Request::Ack ack-tx))
            "test send Ack: req-tx disconnected — driver died?"))
         ((_r :())
          (:wat::core::option::expect -> :()
            (:wat::core::result::expect -> :Option<()>
              (:wat::kernel::recv ack-rx)
              "test recv ack: ack-rx peer thread died")
            "test recv ack: ack-rx clean disconnect — driver died mid-Ack?"))

         ;; 1 Get — expect (push=2, ack=1). expect on the recv unwraps
         ;; Option<State> directly into State; :None panics with a
         ;; meaningful diagnostic rather than masking as a "no snap".
         ((_g1 :())
          (:wat::core::result::expect -> :()
            (:wat::kernel::send req-tx (:svc::Request::Get get-tx))
            "test send Get #1: req-tx disconnected — driver died?"))
         ((snap1 :svc::State)
          (:wat::core::option::expect -> :svc::State
            (:wat::core::result::expect -> :Option<svc::State>
              (:wat::kernel::recv get-rx)
              "test recv get #1: peer thread died")
            "test recv get #1: clean disconnect — driver died mid-Get?"))
         ((_check1a :())
          (:wat::core::if (:wat::core::= (:svc::State/push-count snap1) 2) -> :()
            ()
            (:wat::test::assert-eq "snap1 push != 2" "")))
         ((_check1b :())
          (:wat::core::if (:wat::core::= (:svc::State/ack-count snap1) 1) -> :()
            ()
            (:wat::test::assert-eq "snap1 ack != 1" "")))

         ;; 1 more Push, then Get — expect (push=3, ack=1).
         ;; Proves Get reads LIVE state, not a frozen capture.
         ((_p3 :())
          (:wat::core::result::expect -> :()
            (:wat::kernel::send req-tx (:svc::Request::Push 300))
            "test send Push 300: req-tx disconnected — driver died?"))
         ((_g2 :())
          (:wat::core::result::expect -> :()
            (:wat::kernel::send req-tx (:svc::Request::Get get-tx))
            "test send Get #2: req-tx disconnected — driver died?"))
         ((snap2 :svc::State)
          (:wat::core::option::expect -> :svc::State
            (:wat::core::result::expect -> :Option<svc::State>
              (:wat::kernel::recv get-rx)
              "test recv get #2: peer thread died")
            "test recv get #2: clean disconnect — driver died mid-Get?"))
         ((_check2a :())
          (:wat::core::if (:wat::core::= (:svc::State/push-count snap2) 3) -> :()
            ()
            (:wat::test::assert-eq "snap2 push != 3" "")))
         ((_check2b :())
          (:wat::core::if (:wat::core::= (:svc::State/ack-count snap2) 1) -> :()
            ()
            (:wat::test::assert-eq "snap2 ack != 1" ""))))
        ()))

     ;; Final state via join-result. Should match the last snapshot.
     ((result :Result<svc::State,Vec<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::join-result driver)))
    (:wat::core::match result -> :()
      ((Ok s)
        (:wat::core::let*
          (((pc :wat::core::i64) (:svc::State/push-count s))
           ((ac :wat::core::i64) (:svc::State/ack-count s))
           ((_ :())
            (:wat::core::if (:wat::core::= pc 3) -> :()
              ()
              (:wat::test::assert-eq "final push != 3" ""))))
          (:wat::core::if (:wat::core::= ac 1) -> :()
            ()
            (:wat::test::assert-eq "final ack != 1" ""))))
      ((Err _) (:wat::test::assert-eq "driver-died" "")))))
