;; the-rope-can-be-looked-at — non-consuming lineage-status from wat.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-lineage-status.wat

(:wat::core::defn :probe::render
  [st <- (:wat::core::Option :- [:wat::kernel::CloseOutcome])]
  -> :wat::core::String
  (:wat::core::match st
    (:wat::core::None "None")
    ((:wat::core::Some o)
      (:wat::core::match o
        ((:wat::kernel::CloseOutcome::Closed exit)
          (:wat::core::match exit
            (:wat::core::None "Some(Closed None)")
            ((:wat::core::Some code)
              (:wat::core::format "Some(Closed Some({c}))"
                :c (:wat::i64::to-string code)))))
        ((:wat::kernel::CloseOutcome::Signaled sig)
          (:wat::core::format "Some(Signaled {s})"
            :s (:wat::i64::to-string sig)))
        ((:wat::kernel::CloseOutcome::Failed cause)
          (:wat::core::format "Some(Failed {m})"
            :m (:wat::kernel::Failure/message cause)))))))

(:wat::core::defn :probe::pause [] -> :wat::core::nil
  (:wat::core::let
    [tmr (:wat::kernel::after :wat::program::PeerKind::thread
            (:wat::time::Milliseconds 25) 0)]
    (:wat::core::match (:wat::kernel::recv tmr)
      (_ nil))))

(:wat::core::defn :probe::until-some
  [again <- [:-> (:wat::core::Option :- [:wat::kernel::CloseOutcome])]
   n <- :wat::core::i64]
  -> (:wat::core::Option :- [:wat::kernel::CloseOutcome])
  (:wat::core::match (again)
    ((:wat::core::Some o) (:wat::core::Some o))
    (:wat::core::None
      (:wat::core::if (:wat::i64::<= n 0)
        :wat::core::None
        (:wat::core::let [_ (:probe::pause)]
          (:probe::until-some again (:wat::i64::- n 1)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ── still running (process) ──────────────────────────────────────────
     live (:wat::test::spawn-peer (:wat::spawn::process)
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil
                 (:wat::core::let
                   [tmr (:wat::kernel::after :wat::program::PeerKind::process
                           (:wat::time::Milliseconds 3600000) 0)]
                   (:wat::core::match (:wat::kernel::recv tmr)
                     (_ nil))))))
     running (:probe::render (:wat::kernel::lineage-status live))
     running2 (:probe::render (:wat::kernel::lineage-status live))

     ;; ── Closed, process (clean exit) ─────────────────────────────────────
     p-exit (:wat::test::spawn-peer (:wat::spawn::process)
               (:wat::core::forms
                 (:wat::core::defn :user::main [] -> :wat::core::nil nil)))
     p-closed (:probe::render
                (:probe::until-some
                  (:wat::core::fn [] -> (:wat::core::Option :- [:wat::kernel::CloseOutcome])
                    (:wat::kernel::lineage-status p-exit))
                  80))
     p-closed2 (:probe::render (:wat::kernel::lineage-status p-exit))

     ;; ── Closed, thread (clean exit) ──────────────────────────────────────
     t-exit (:wat::test::spawn-peer (:wat::spawn::thread)
               (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
                 nil))
     t-closed (:probe::render
                (:probe::until-some
                  (:wat::core::fn [] -> (:wat::core::Option :- [:wat::kernel::CloseOutcome])
                    (:wat::kernel::lineage-status t-exit))
                  80))

     ;; ── Signaled 9 (Kill) ────────────────────────────────────────────────
     p-kill (:wat::test::spawn-peer (:wat::spawn::process)
               (:wat::core::forms
                 (:wat::core::defn :user::main [] -> :wat::core::nil
                   (:wat::core::let
                     [tmr (:wat::kernel::after :wat::program::PeerKind::process
                             (:wat::time::Milliseconds 3600000) 0)]
                     (:wat::core::match (:wat::kernel::recv tmr)
                       (_ nil))))))
     _kill (:wat::core::match (:wat::kernel::signal p-kill :wat::kernel::Signal::Kill)
             (:wat::kernel::SignalOutcome::Delivered nil)
             ((:wat::kernel::SignalOutcome::Failed cause)
               (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause)
                 :wat::core::None :wat::core::None)))
     signaled (:probe::render
                (:probe::until-some
                  (:wat::core::fn [] -> (:wat::core::Option :- [:wat::kernel::CloseOutcome])
                    (:wat::kernel::lineage-status p-kill))
                  80))
     signaled2 (:probe::render (:wat::kernel::lineage-status p-kill))

     ;; ── Failed via SIGSTOP (not Signaled) ────────────────────────────────
     p-stop (:wat::test::spawn-peer (:wat::spawn::process)
               (:wat::core::forms
                 (:wat::core::defn :user::main [] -> :wat::core::nil
                   (:wat::core::let
                     [tmr (:wat::kernel::after :wat::program::PeerKind::process
                             (:wat::time::Milliseconds 3600000) 0)]
                     (:wat::core::match (:wat::kernel::recv tmr)
                       (_ nil))))))
     _stop (:wat::core::match (:wat::kernel::signal p-stop :wat::kernel::Signal::Stop)
             (:wat::kernel::SignalOutcome::Delivered nil)
             ((:wat::kernel::SignalOutcome::Failed cause)
               (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause)
                 :wat::core::None :wat::core::None)))
     stopped (:probe::render
               (:probe::until-some
                 (:wat::core::fn [] -> (:wat::core::Option :- [:wat::kernel::CloseOutcome])
                   (:wat::kernel::lineage-status p-stop))
                 80))
     ;; Reap so RAII Drop does not hang on a frozen child (close waits WEXITED).
     _kill-stop (:wat::core::match (:wat::kernel::signal p-stop :wat::kernel::Signal::Kill)
                  (:wat::kernel::SignalOutcome::Delivered nil)
                  ((:wat::kernel::SignalOutcome::Failed cause)
                    (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause)
                      :wat::core::None :wat::core::None)))

     ;; ── panicking thread (disk: join is Ok after catch_unwind) ───────────
     t-panic (:wat::test::spawn-peer (:wat::spawn::thread)
               (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
                 (:wat::kernel::assertion-failed! "probe panic"
                   :wat::core::None :wat::core::None)))
     panicked (:probe::render
                (:probe::until-some
                  (:wat::core::fn [] -> (:wat::core::Option :- [:wat::kernel::CloseOutcome])
                    (:wat::kernel::lineage-status t-panic))
                  80))
     ;; Reap the still-running live child so RAII Drop does not wait out the 1h park.
     _kill-live (:wat::core::match (:wat::kernel::signal live :wat::kernel::Signal::Kill)
                  (:wat::kernel::SignalOutcome::Delivered nil)
                  ((:wat::kernel::SignalOutcome::Failed cause)
                    (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause)
                      :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      (:wat::core::format
        "running={r};running2={r2};process-closed={pc};process-closed2={pc2};thread-closed={tc};signaled={sg};signaled2={sg2};stopped={st};panicked={pn}"
        :r running :r2 running2
        :pc p-closed :pc2 p-closed2
        :tc t-closed
        :sg signaled :sg2 signaled2
        :st stopped
        :pn panicked))))
