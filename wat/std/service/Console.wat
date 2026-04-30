;; :wat::std::service::Console — the sole gateway to the world's
;; stdio. User direction 2026-04-19: "the console /should/ be the
;; only way to print to the world... anyone who wants console access
;; /must/ be provisioned a pair of pipes and invoke console through
;; the pipes."
;;
;; Model:
;;   - Console owns BOTH stdout and stderr (the real crossbeam
;;     senders the wat passes to :user::main).
;;   - Each client gets ONE queue carrying tagged messages
;;     `(tag :wat::core::i64, msg :wat::core::String)` — tag 0 = stdout, tag 1 = stderr.
;;   - Users call the thin wrappers `Console/out` / `Console/err`
;;     which encode the tag; the Console driver decodes and forwards.
;;   - One select loop, one thread, N fan-in sources. Clean.
;;
;; The good wat program:
;;   (define (:user::main stdin stdout stderr -> :())
;;     (let* ((pool console-driver) (Console stdout stderr N))
;;       ...hand out handles, use them, drop them...
;;       (join console-driver)))
;;
;; After passing stdout and stderr to Console, the program should
;; IGNORE those bindings — every print from every thread should go
;; through a Console-provisioned handle.

;; --- Tag constants ---
;;
;; Ints inline in Console/out and Console/err below; named here for
;; reader clarity. 0 = stdout, 1 = stderr. No enum yet; tuples suffice.

;; --- Message typealias ---
;;
;; A Console message is (tag :wat::core::i64, msg :wat::core::String). The ack address
;; isn't carried in the payload — the driver pairs each rx with
;; its matching ack-tx by index when select fires (see the loop
;; below). One write pipe + one ack pipe per producer scope; the
;; two pipes mutually block each other through bounded(1).
(:wat::core::typealias :wat::std::service::Console::Message
  :(i64,String))
(:wat::core::typealias :wat::std::service::Console::Tx
  :wat::kernel::QueueSender<wat::std::service::Console::Message>)
(:wat::core::typealias :wat::std::service::Console::Rx
  :wat::kernel::QueueReceiver<wat::std::service::Console::Message>)


;; --- Ack channel + handle typealiases (arc 089 slice 5) ---
;;
;; In-memory TCP: producer writes on Console::Tx, blocks on
;; Console::AckRx until the driver finishes the corresponding
;; IOWriter/write-string. Each producer scope creates ONE pair of
;; pipes — one for each direction — and the bounded(1) on both
;; gives mutual blocking without any extra plumbing.
;;
;; Console::Handle = (Tx, AckRx). Pop one of these from the pool
;; at the producer's scope; pass it into Console/out and
;; Console/err. The driver's internal pairs hold the matching
;; (Rx, AckTx) — paired by index inside Console/spawn.
(:wat::core::typealias :wat::std::service::Console::AckTx
  :wat::kernel::QueueSender<()>)
(:wat::core::typealias :wat::std::service::Console::AckRx
  :wat::kernel::QueueReceiver<()>)
(:wat::core::typealias :wat::std::service::Console::Handle
  :(wat::std::service::Console::Tx,wat::std::service::Console::AckRx))
(:wat::core::typealias :wat::std::service::Console::DriverPair
  :(wat::std::service::Console::Rx,wat::std::service::Console::AckTx))

;; --- Spawn return shape ---
;;
;; What `:wat::std::service::Console/spawn` returns: the HandlePool of
;; per-producer Handles ((Tx, AckRx) pairs) + the driver's
;; ProgramHandle. Caller pops N Handles, finishes the pool,
;; scoped-drops at end → driver exits.
(:wat::core::typealias :wat::std::service::Console::Spawn
  :(wat::kernel::HandlePool<wat::std::service::Console::Handle>,wat::kernel::ProgramHandle<()>))

;; --- Driver loop ---
;;
;; Select across N producers' request receivers; on a select fire
;; at index i, write the (tag, msg) to the matching IOWriter and
;; send () on the ack-tx paired with that receiver at index i.
;; The pair index IS the routing — the substrate's `select`
;; already told us WHICH producer fired; the matching ack-tx
;; lives at the same index in the driver's pairs vec. No payload
;; bloat for ack routing.
;;
;; Ack discipline (arc 089 slice 5): producer is blocked on its
;; ack-rx; we send () AFTER IOWriter/write-string returns, so the
;; producer's unblock happens after the bytes are durable.
;; Bounded(1) on both pipes gives mutual blocking — the producer
;; can't queue another message until the driver acked the
;; previous one. The `select` IS the mutex — only one producer
;; can be processed at a time; the ack is the release.
;;
;; Removes disconnected pairs (producer scope exited → req-tx
;; dropped → req-rx returns :None on select). Exits when no
;; pairs remain.
(:wat::core::define
  (:wat::std::service::Console/loop
    (pairs :Vec<wat::std::service::Console::DriverPair>)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::if (:wat::core::empty? pairs) -> :()
    ()
    (:wat::core::let*
      (((rxs :Vec<wat::std::service::Console::Rx>)
        (:wat::core::map pairs
          (:wat::core::lambda
            ((p :wat::std::service::Console::DriverPair)
             -> :wat::std::service::Console::Rx)
            (:wat::core::first p))))
       ((chosen :(i64,Option<wat::std::service::Console::Message>))
        (:wat::kernel::select rxs))
       ((idx :wat::core::i64) (:wat::core::first chosen))
       ((maybe :Option<wat::std::service::Console::Message>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :()
        ((Some tagged)
          (:wat::core::let*
            (((tag :wat::core::i64) (:wat::core::first tagged))
             ((msg :wat::core::String) (:wat::core::second tagged))
             ((_ :wat::core::i64) (:wat::core::if (:wat::core::= tag 0) -> :wat::core::i64
                        (:wat::io::IOWriter/write-string stdout msg)
                        (:wat::io::IOWriter/write-string stderr msg)))
             ((_ack :())
              (:wat::std::service::Console/ack-at pairs idx)))
            (:wat::std::service::Console/loop pairs stdout stderr)))
        (:None
          (:wat::std::service::Console/loop
            (:wat::std::list::remove-at pairs idx)
            stdout
            stderr))))))


;; --- Helper — send () on the ack-tx paired with rx[idx]. -----
;;
;; Lifted out of the loop body so the loop's outer let* stays
;; one-let-deep per `feedback_simple_forms_per_func`. `(get pairs
;; idx)` returns Option per arc 047; an out-of-bounds idx (which
;; can't happen here since `select` returned a valid index over
;; the same vec we mapped) collapses to a no-op.
(:wat::core::define
  (:wat::std::service::Console/ack-at
    (pairs :Vec<wat::std::service::Console::DriverPair>)
    (idx :wat::core::i64)
    -> :())
  (:wat::core::match (:wat::core::get pairs idx) -> :()
    ((Some pair)
      (:wat::core::let*
        (((ack-tx :wat::std::service::Console::AckTx)
          (:wat::core::second pair)))
        (:wat::core::option::expect -> :()
          (:wat::kernel::send ack-tx ())
          "Console/ack-at: ack-tx disconnected — producer scope died mid-write?")))
    (:None ())))

;; --- Client helpers ---
;;
;; Each producer pops one Console::Handle from the pool — that's
;; their (Tx, AckRx) pair. The helpers below destructure the
;; handle, send the (tag, msg) tuple on the request channel, then
;; block on the ack channel until the driver writes and acks.
;;
;; `(tag, msg)` is the entire payload — no per-call ack address
;; bundled in. The driver pairs the producer's ack-tx with the
;; producer's req-rx by index inside `Console/loop`, so the
;; routing falls out of `select` for free.
;;
;; If the Console driver has already shut down: send → :None (req
;; channel disconnected), recv → :None (ack-tx the driver held
;; dropped). Per arc 110: in-memory peer-death is catastrophic.
;; Either disconnect panics here so the caller's program tree
;; learns its sink is dead instead of silently dropping prints.
(:wat::core::define
  (:wat::std::service::Console/out
    (handle :wat::std::service::Console::Handle)
    (msg :wat::core::String)
    -> :())
  (:wat::core::let*
    (((tx :wat::std::service::Console::Tx) (:wat::core::first handle))
     ((ack-rx :wat::std::service::Console::AckRx) (:wat::core::second handle))
     ((_send :())
      (:wat::core::option::expect -> :()
        (:wat::kernel::send tx (:wat::core::tuple 0 msg))
        "Console/out: tx disconnected — Console driver died?")))
    (:wat::core::option::expect -> :()
      (:wat::kernel::recv ack-rx)
      "Console/out: ack-rx disconnected — Console driver died mid-write?")))

(:wat::core::define
  (:wat::std::service::Console/err
    (handle :wat::std::service::Console::Handle)
    (msg :wat::core::String)
    -> :())
  (:wat::core::let*
    (((tx :wat::std::service::Console::Tx) (:wat::core::first handle))
     ((ack-rx :wat::std::service::Console::AckRx) (:wat::core::second handle))
     ((_send :())
      (:wat::core::option::expect -> :()
        (:wat::kernel::send tx (:wat::core::tuple 1 msg))
        "Console/err: tx disconnected — Console driver died?")))
    (:wat::core::option::expect -> :()
      (:wat::kernel::recv ack-rx)
      "Console/err: ack-rx disconnected — Console driver died mid-write?")))

;; --- Console setup ---
;;
;; Builds N bounded(1) queues carrying tagged messages, wraps the
;; senders in a HandlePool, spawns one driver thread that fans in
;; all receivers and dispatches to stdout / stderr by tag, returns
;; (pool, driver-handle).
;;
;; The returned tuple is the honest shutdown contract: caller pops
;; N handles, distributes, calls HandlePool::finish, does its work,
;; drops all handles (end of their scope), then calls
;; `(join driver)`. The drop cascade triggers the loop's clean exit.
(:wat::core::define
  (:wat::std::service::Console/spawn
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    (count :wat::core::i64)
    -> :wat::std::service::Console::Spawn)
  (:wat::core::let*
    ;; Build N request pairs and N ack pairs in lock-step. The
    ;; index of the request pair matches the index of the ack
    ;; pair — this is what makes pair-by-index ack routing
    ;; possible inside Console/loop.
    (((req-pairs :Vec<(wat::std::service::Console::Tx,wat::std::service::Console::Rx)>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :wat::core::i64)
           -> :(wat::std::service::Console::Tx,wat::std::service::Console::Rx))
          (:wat::kernel::make-bounded-queue
            :wat::std::service::Console::Message 1))))
     ((ack-pairs :Vec<(wat::std::service::Console::AckTx,wat::std::service::Console::AckRx)>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda
          ((_i :wat::core::i64)
           -> :(wat::std::service::Console::AckTx,wat::std::service::Console::AckRx))
          (:wat::kernel::make-bounded-queue :() 1))))
     ;; Producer-side: pop a Handle = (req-Tx, ack-Rx).
     ((handles :Vec<wat::std::service::Console::Handle>)
      (:wat::std::list::zip
        (:wat::core::map req-pairs
          (:wat::core::lambda
            ((p :(wat::std::service::Console::Tx,wat::std::service::Console::Rx))
             -> :wat::std::service::Console::Tx)
            (:wat::core::first p)))
        (:wat::core::map ack-pairs
          (:wat::core::lambda
            ((p :(wat::std::service::Console::AckTx,wat::std::service::Console::AckRx))
             -> :wat::std::service::Console::AckRx)
            (:wat::core::second p)))))
     ;; Driver-side: Vec<DriverPair> = (req-Rx, ack-Tx) at the
     ;; matching index. select fires for idx i; pairs[i].second
     ;; is the ack-Tx the driver writes back on.
     ((driver-pairs :Vec<wat::std::service::Console::DriverPair>)
      (:wat::std::list::zip
        (:wat::core::map req-pairs
          (:wat::core::lambda
            ((p :(wat::std::service::Console::Tx,wat::std::service::Console::Rx))
             -> :wat::std::service::Console::Rx)
            (:wat::core::second p)))
        (:wat::core::map ack-pairs
          (:wat::core::lambda
            ((p :(wat::std::service::Console::AckTx,wat::std::service::Console::AckRx))
             -> :wat::std::service::Console::AckTx)
            (:wat::core::first p)))))
     ((pool :wat::kernel::HandlePool<wat::std::service::Console::Handle>)
      (:wat::kernel::HandlePool::new "Console" handles))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::service::Console/loop
        driver-pairs stdout stderr)))
    (:wat::core::tuple pool driver)))
