;; wat-tests/service-template.wat — the canonical in-memory
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
;;
;; Arc 130 — complectēns rewrite. Top-down dependency graph in ONE file.
;;
;; ─── Layers ──────────────────────────────────────────────────────────
;;
;;   Layer 0  :test::svc-spawn-and-shutdown
;;              ; spawn service (1 client), pop handle, finish, join
;;
;;   Layer 1  :test::svc-send-push
;;              ; send Push(value) to req-tx — fire and forget
;;
;;   Layer 2  :test::svc-assert-state
;;              ; assert push-count and ack-count match expected values
;;
;;   Layer 3  :test::svc-full-sequence-and-verify
;;              ; spawn, drive full sequence (2 Push + Ack + 2×Get + Push),
;;              ; check mid-sequence snapshots, recv final, join, assert final.
;;              ; Keeps make-bounded-channel + send/recv inline (not via
;;              ; helpers with both halves) so arc 126 does not fire here.
;;
;;   Final    :svc::test-template-end-to-end   (1 line — composes Layer 3)
;;
;; Note on arc 126 and send/recv vs helper verbs:
;;   The test drives the service via Request::Ack(ack-tx) and
;;   Request::Get(get-tx) by embedding only ONE channel half in the
;;   request payload and recving on the other half in a separate call.
;;   Arc 126 traces function calls that receive BOTH halves of the same
;;   channel pair as arguments simultaneously. The send/recv pattern here
;;   never passes both halves to a single function call — so arc 126 does
;;   not fire, and all deftests pass cleanly without :should-panic.

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
   (:wat::core::typealias :svc::AckReplyTx :wat::kernel::Sender<wat::core::unit>)
   (:wat::core::typealias :svc::AckReplyRx :wat::kernel::Receiver<wat::core::unit>)

   ;; Get's reply channel carries the full State struct. Inlined in
   ;; the variant declaration below — domain-payload reply types are
   ;; per-verb and don't tend to repeat outside the variant they belong
   ;; to (one alias per such variant adds noise, not signal).

   ;; Request — three reply shapes side by side. Every in-memory
   ;; request/reply service is some combination of these.
   (:wat::core::enum :svc::Request
     (Push (value :wat::core::i64))
     (Ack  (reply-tx :svc::AckReplyTx))
     (Get  (reply-tx :wat::kernel::Sender<svc::State>)))

   ;; Per-broker request channel typealiases. Idiomatic: every service
   ;; has these four (Tx / Rx / TxPool / Spawn) — they describe the
   ;; service's wire shape independent of state or verbs.
   (:wat::core::typealias :svc::ReqTx :wat::kernel::Sender<svc::Request>)
   (:wat::core::typealias :svc::ReqRx :wat::kernel::Receiver<svc::Request>)
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
             (:wat::core::Result/expect -> :wat::core::unit
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
             (:wat::core::Result/expect -> :wat::core::unit
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
       (:wat::core::Result/expect -> :wat::core::unit
         (:wat::kernel::send out state)
         "Service/loop: out disconnected — parent dropped Thread/output before recv?")
       (:wat::core::let*
         (((chosen :wat::kernel::Chosen<svc::Request>) (:wat::kernel::select req-rxs))
          ((idx :wat::core::i64) (:wat::core::first chosen))
          ((maybe :wat::kernel::CommResult<svc::Request>) (:wat::core::second chosen)))
         (:wat::core::match maybe -> :wat::core::unit
           ((:wat::core::Ok (:wat::core::Some req))
             (:wat::core::let*
               (((next :svc::State) (:svc::Service/handle req state)))
               (:svc::Service/loop req-rxs next out)))
           ((:wat::core::Ok :wat::core::None)
             (:svc::Service/loop (:wat::std::list::remove-at req-rxs idx) state out))
           ((:wat::core::Err _died)
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
       (((pairs :wat::core::Vector<wat::kernel::Channel<svc::Request>>)
         (:wat::core::map
           (:wat::core::range 0 count)
           (:wat::core::lambda ((_i :wat::core::i64) -> :wat::kernel::Channel<svc::Request>)
             (:wat::kernel::make-bounded-channel :svc::Request 1))))

        ((req-txs :wat::core::Vector<svc::ReqTx>)
         (:wat::core::map pairs
           (:wat::core::lambda ((p :wat::kernel::Channel<svc::Request>) -> :svc::ReqTx)
             (:wat::core::first p))))

        ((req-rxs :wat::core::Vector<svc::ReqRx>)
         (:wat::core::map pairs
           (:wat::core::lambda ((p :wat::kernel::Channel<svc::Request>) -> :svc::ReqRx)
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
       (:wat::core::Tuple pool thr)))


   ;; ─── Layer 0 — lifecycle ─────────────────────────────────────
   ;;
   ;; Spawn the service (1 client), pop the handle so the pool has no
   ;; orphaned handles, finish the pool, drop the req-tx at inner scope
   ;; exit. The driver delivers final State on Thread/output — recv it
   ;; (even though unused) so the driver's send doesn't fail. Join.
   ;; No requests sent — the narrowest possible lifecycle proof.
   ;; Inner-let* lockstep per SERVICE-PROGRAMS.md § "The lockstep" and
   ;; arc 131. Pop-before-finish per arc 130 edge-case guidance.
   (:wat::core::define
     (:test::svc-spawn-and-shutdown -> :wat::core::unit)
     (:wat::core::let*
       (((driver :wat::kernel::Thread<wat::core::unit,svc::State>)
         (:wat::core::let*
           (((spawn :svc::Spawn) (:svc::Service 1))
            ((pool :svc::ReqTxPool) (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,svc::State>) (:wat::core::second spawn))
            ((_req-tx :svc::ReqTx) (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool)))
           d))
        ((final-rx :rust::crossbeam_channel::Receiver<svc::State>)
         (:wat::kernel::Thread/output driver))
        ((_final-state :svc::State)
         (:wat::core::Option/expect -> :svc::State
           (:wat::core::Result/expect -> :wat::core::Option<svc::State>
             (:wat::kernel::recv final-rx)
             "svc-spawn-and-shutdown: thread died before delivering final state")
           "svc-spawn-and-shutdown: thread output closed without delivering final state"))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))


   ;; ─── Layer 1 — fire-and-forget send ──────────────────────────
   ;;
   ;; :test::svc-send-push — send Push(value) to req-tx. No reply
   ;; channel — single Sender argument, so arc 126 does not apply.
   ;; Panics if req-tx is disconnected (driver died unexpectedly).
   (:wat::core::define
     (:test::svc-send-push
       (req-tx :svc::ReqTx)
       (v :wat::core::i64)
       -> :wat::core::unit)
     (:wat::core::Result/expect -> :wat::core::unit
       (:wat::kernel::send req-tx (:svc::Request::Push v))
       "svc-send-push: req-tx disconnected — driver died?"))


   ;; ─── Layer 2 — assertion helper ──────────────────────────────
   ;;
   ;; :test::svc-assert-state — assert that state's push-count and
   ;; ack-count equal the expected values. Fails with a labelled
   ;; assert-eq on mismatch. Pure function — no channels or threads.
   (:wat::core::define
     (:test::svc-assert-state
       (state :svc::State)
       (push-expected :wat::core::i64)
       (ack-expected :wat::core::i64)
       -> :wat::core::unit)
     (:wat::core::let*
       (((_pc :wat::core::unit)
         (:wat::core::if (:wat::core::= (:svc::State/push-count state) push-expected)
           -> :wat::core::unit
           ()
           (:wat::test::assert-eq "push-count mismatch" ""))))
       (:wat::core::if (:wat::core::= (:svc::State/ack-count state) ack-expected)
         -> :wat::core::unit
         ()
         (:wat::test::assert-eq "ack-count mismatch" ""))))


   ;; ─── Layer 3 — full scenario ──────────────────────────────────
   ;;
   ;; :test::svc-full-sequence-and-verify — spawn service (1 client),
   ;; allocate reply channels, drive the full sequence:
   ;;   Push 100, Push 200, Ack, Get→snap1 (assert 2 push/1 ack),
   ;;   Push 300, Get→snap2 (assert 3 push/1 ack).
   ;; Drops inner scope (req-tx + reply channels) → driver delivers
   ;; final state on Thread/output → recv + join → assert final counters.
   ;; Returns unit; all assertions internal.
   ;;
   ;; Arc 126 note: Ack and Get channels are allocated inline here.
   ;; ack-tx moves INTO the Request::Ack payload via send (not passed
   ;; alongside ack-rx to a single function). get-tx similarly moves
   ;; INTO Request::Get. Each recv is a separate call on just one half.
   ;; Arc 126's deadlock checker never sees both halves at one call site
   ;; → no channel-pair-deadlock fires.
   (:wat::core::define
     (:test::svc-full-sequence-and-verify -> :wat::core::unit)
     (:wat::core::let*
       (((thr :wat::kernel::Thread<wat::core::unit,svc::State>)
         (:wat::core::let*
           (((spawn :svc::Spawn) (:svc::Service 1))
            ((pool :svc::ReqTxPool) (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,svc::State>) (:wat::core::second spawn))
            ((req-tx :svc::ReqTx) (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
            ;; Ack reply channel — tx embedded in Request payload; rx recvd separately.
            ((ack-pair :wat::kernel::Channel<wat::core::unit>)
             (:wat::kernel::make-bounded-channel :wat::core::unit 1))
            ((ack-tx :svc::AckReplyTx) (:wat::core::first ack-pair))
            ((ack-rx :svc::AckReplyRx) (:wat::core::second ack-pair))
            ;; Get reply channel — tx embedded in Request payload; rx recvd separately.
            ((get-pair :wat::kernel::Channel<svc::State>)
             (:wat::kernel::make-bounded-channel :svc::State 1))
            ((get-tx :wat::kernel::Sender<svc::State>) (:wat::core::first get-pair))
            ((get-rx :wat::kernel::Receiver<svc::State>) (:wat::core::second get-pair))
            ;; Drive: 2 Pushes, 1 Ack, 1 Get, check snap1, 1 Push, 1 Get, check snap2.
            ((_ :wat::core::unit) (:test::svc-send-push req-tx 100))
            ((_ :wat::core::unit) (:test::svc-send-push req-tx 200))
            ((_ :wat::core::unit)
             (:wat::core::Result/expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Ack ack-tx))
               "svc-full-sequence: send Ack: req-tx disconnected"))
            ((_ :wat::core::unit)
             (:wat::core::Option/expect -> :wat::core::unit
               (:wat::core::Result/expect -> :wat::core::Option<wat::core::unit>
                 (:wat::kernel::recv ack-rx)
                 "svc-full-sequence: recv ack: peer died")
               "svc-full-sequence: recv ack: clean disconnect"))
            ((_ :wat::core::unit)
             (:wat::core::Result/expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Get get-tx))
               "svc-full-sequence: send Get #1: req-tx disconnected"))
            ((snap1 :svc::State)
             (:wat::core::Option/expect -> :svc::State
               (:wat::core::Result/expect -> :wat::core::Option<svc::State>
                 (:wat::kernel::recv get-rx)
                 "svc-full-sequence: recv get #1: peer died")
               "svc-full-sequence: recv get #1: clean disconnect"))
            ((_ :wat::core::unit) (:test::svc-assert-state snap1 2 1))
            ((_ :wat::core::unit) (:test::svc-send-push req-tx 300))
            ((_ :wat::core::unit)
             (:wat::core::Result/expect -> :wat::core::unit
               (:wat::kernel::send req-tx (:svc::Request::Get get-tx))
               "svc-full-sequence: send Get #2: req-tx disconnected"))
            ((snap2 :svc::State)
             (:wat::core::Option/expect -> :svc::State
               (:wat::core::Result/expect -> :wat::core::Option<svc::State>
                 (:wat::kernel::recv get-rx)
                 "svc-full-sequence: recv get #2: peer died")
               "svc-full-sequence: recv get #2: clean disconnect"))
            ((_ :wat::core::unit) (:test::svc-assert-state snap2 3 1)))
           d))
        ((final-rx :rust::crossbeam_channel::Receiver<svc::State>)
         (:wat::kernel::Thread/output thr))
        ((final-state :svc::State)
         (:wat::core::Option/expect -> :svc::State
           (:wat::core::Result/expect -> :wat::core::Option<svc::State>
             (:wat::kernel::recv final-rx)
             "svc-full-sequence: thread died before delivering final state")
           "svc-full-sequence: thread output closed without delivering final state"))
        ((join-result :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result thr)))
       (:wat::core::match join-result -> :wat::core::unit
         ((:wat::core::Ok _)
           (:test::svc-assert-state final-state 3 1))
         ((:wat::core::Err _) (:wat::test::assert-eq "driver-died" "")))))

   ))


;; ─── Per-layer deftests ────────────────────────────────────────────────────
;;
;; Each layer carries its own proof. cargo test shows the tree.
;; All pass cleanly: no arc-126 trigger in the prelude because
;; send/recv calls here never pass both channel halves to a single function.

;; Layer 0 — lifecycle proof.
(:deftest :svc::test-svc-spawn-and-shutdown
  (:test::svc-spawn-and-shutdown))


;; Layer 1 — send-push proof.
;; Spawns service, sends one Push, drops inner scope → driver delivers
;; final state on Thread/output → recv (discarded) → join.
(:deftest :svc::test-svc-send-push
  (:wat::core::let*
    (((thr :wat::kernel::Thread<wat::core::unit,svc::State>)
      (:wat::core::let*
        (((spawn :svc::Spawn) (:svc::Service 1))
         ((pool :svc::ReqTxPool) (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,svc::State>) (:wat::core::second spawn))
         ((req-tx :svc::ReqTx) (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
         ((_ :wat::core::unit) (:test::svc-send-push req-tx 42)))
        d))
     ((final-rx :rust::crossbeam_channel::Receiver<svc::State>)
      (:wat::kernel::Thread/output thr))
     ((_final-state :svc::State)
      (:wat::core::Option/expect -> :svc::State
        (:wat::core::Result/expect -> :wat::core::Option<svc::State>
          (:wat::kernel::recv final-rx)
          "test-svc-send-push: thread died before delivering final state")
        "test-svc-send-push: thread output closed without delivering final state"))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result thr)))
    ()))


;; Layer 2 — assert-state proof (pure: no threading, no channels).
(:deftest :svc::test-svc-assert-state
  (:test::svc-assert-state (:svc::State/new 3 1) 3 1))


;; Layer 3 — full-sequence proof.
(:deftest :svc::test-svc-full-sequence-and-verify
  (:test::svc-full-sequence-and-verify))


;; ─── Final — the named scenario ────────────────────────────────────────────
;;
;; Body is 1 line BECAUSE the layers exist. The scenario is named and
;; proven; the deftest is just the invocation.
(:deftest :svc::test-template-end-to-end
  (:test::svc-full-sequence-and-verify))
