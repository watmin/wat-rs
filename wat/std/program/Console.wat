;; :wat::std::program::Console — the sole gateway to the world's
;; stdio. User direction 2026-04-19: "the console /should/ be the
;; only way to print to the world... anyone who wants console access
;; /must/ be provisioned a pair of pipes and invoke console through
;; the pipes."
;;
;; Model:
;;   - Console owns BOTH stdout and stderr (the real crossbeam
;;     senders the wat-vm passes to :user::main).
;;   - Each client gets ONE queue carrying tagged messages
;;     `(tag :i64, msg :String)` — tag 0 = stdout, tag 1 = stderr.
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
;; A Console message is (tag :i64, msg :String). This alias makes
;; every signature in the file name the shape once; `reduce` walks
;; through at unify + shape-inspection sites, so Message and its
;; tuple expansion are interchangeable everywhere.
(:wat::core::typealias :wat::std::program::Console::Message
  :(i64,String))
(:wat::core::typealias :wat::std::program::Console::Tx
  :rust::crossbeam_channel::Sender<wat::std::program::Console::Message>)
(:wat::core::typealias :wat::std::program::Console::Rx
  :rust::crossbeam_channel::Receiver<wat::std::program::Console::Message>)

;; --- Driver loop ---
;;
;; Select across N receivers, decode each message's tag, write to
;; the matching real IO handle. Removes disconnected receivers.
;; Exits when no receivers remain.
(:wat::core::define
  (:wat::std::program::Console/loop
    (rxs :Vec<wat::std::program::Console::Rx>)
    (stdout :rust::std::io::Stdout)
    (stderr :rust::std::io::Stderr)
    -> :())
  (:wat::core::if (:wat::core::empty? rxs) -> :()
    ()
    (:wat::core::let*
      (((chosen :(i64,Option<wat::std::program::Console::Message>))
        (:wat::kernel::select rxs))
       ((idx :i64) (:wat::core::first chosen))
       ((maybe :Option<wat::std::program::Console::Message>)
        (:wat::core::second chosen)))
      (:wat::core::match maybe -> :()
        ((Some tagged)
          (:wat::core::let*
            (((tag :i64) (:wat::core::first tagged))
             ((msg :String) (:wat::core::second tagged))
             ((_ :()) (:wat::core::if (:wat::core::= tag 0) -> :()
                        (:wat::io::write stdout msg)
                        (:wat::io::write stderr msg))))
            (:wat::std::program::Console/loop rxs stdout stderr)))
        (:None
          (:wat::std::program::Console/loop
            (:wat::std::list::remove-at rxs idx)
            stdout
            stderr))))))

;; --- Client helpers ---
;;
;; Each handle is a Console::Tx; callers don't build the tuple
;; themselves, they use Console/out or Console/err.
;; These are fire-and-forget from the client's perspective: if the
;; Console driver has already shut down, the write has nowhere to
;; go and we swallow silently rather than surface a late-lifecycle
;; error. `send` returns :Option<()> after the 2026-04-20
;; symmetrization; both arms of the match collapse to :(). A program
;; that WANTS disconnect awareness uses the primitive `send` directly
;; on its console handle.
(:wat::core::define
  (:wat::std::program::Console/out
    (handle :wat::std::program::Console::Tx)
    (msg :String)
    -> :())
  (:wat::core::match
    (:wat::kernel::send handle (:wat::core::tuple 0 msg))
    -> :()
    ((Some _) ())
    (:None ())))

(:wat::core::define
  (:wat::std::program::Console/err
    (handle :wat::std::program::Console::Tx)
    (msg :String)
    -> :())
  (:wat::core::match
    (:wat::kernel::send handle (:wat::core::tuple 1 msg))
    -> :()
    ((Some _) ())
    (:None ())))

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
  (:wat::std::program::Console
    (stdout :rust::std::io::Stdout)
    (stderr :rust::std::io::Stderr)
    (count :i64)
    -> :(wat::kernel::HandlePool<wat::std::program::Console::Tx>,wat::kernel::ProgramHandle<()>))
  (:wat::core::let*
    (((pairs :Vec<(wat::std::program::Console::Tx,wat::std::program::Console::Rx)>)
      (:wat::core::map
        (:wat::core::range 0 count)
        (:wat::core::lambda ((_i :i64) -> :(wat::std::program::Console::Tx,wat::std::program::Console::Rx))
          (:wat::kernel::make-bounded-queue :wat::std::program::Console::Message 1))))
     ((txs :Vec<wat::std::program::Console::Tx>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :(wat::std::program::Console::Tx,wat::std::program::Console::Rx))
                            -> :wat::std::program::Console::Tx)
          (:wat::core::first p))))
     ((rxs :Vec<wat::std::program::Console::Rx>)
      (:wat::core::map pairs
        (:wat::core::lambda ((p :(wat::std::program::Console::Tx,wat::std::program::Console::Rx))
                            -> :wat::std::program::Console::Rx)
          (:wat::core::second p))))
     ((pool :wat::kernel::HandlePool<wat::std::program::Console::Tx>)
      (:wat::kernel::HandlePool::new "Console" txs))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::program::Console/loop rxs stdout stderr)))
    (:wat::core::tuple pool driver)))
