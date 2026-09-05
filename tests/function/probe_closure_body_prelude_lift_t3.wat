;; tests/function/probe_closure_body_prelude_lift_t3.wat — enum in fn body do-prefix lifts to prologue.
;;
;; Arc 278 IPC de-prime — driver migrated to `spawn-program' (process)` + `recv'`; the
;; declaration under test (`:h::LocalDir` at the child program's top level) is unchanged.
;; The child now MATCHES the variant it constructed and `println`s the mapped i64, so the
;; assertion proves the enum was registered AND its variants construct AND match — stronger
;; than exit-0. Both variants are named (full-enum matching is mandatory; no wildcard arm).
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defenum :h::LocalDir :wat::enum::Pure
             :North
             :South)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [d    :h::LocalDir::North
                n    (:wat::core::match d
                       (:h::LocalDir::North 1)
                       (:h::LocalDir::South 2))
                _out (:wat::kernel::println n)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch: stop requested before child sent its value — child was ALIVE, channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
