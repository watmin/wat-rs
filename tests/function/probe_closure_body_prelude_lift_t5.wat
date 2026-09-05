;; tests/function/probe_closure_body_prelude_lift_t5.wat — prefix-termination semantics.
;;
;; Arc 278 IPC de-prime — driver migrated to `spawn-program' (process)` + `recv'`; the
;; declaration under test (`:h::counted-helper` at the child program's top level) is unchanged.
;; The child now `println`s the helper's value, so the assertion proves it was registered AND
;; callable AND returned 7 — stronger than exit-0.
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :h::counted-helper [] -> :wat::core::i64 7)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [v    (:h::counted-helper)
                _out (:wat::kernel::println v)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch: stop requested before child sent its value — child was ALIVE, channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
