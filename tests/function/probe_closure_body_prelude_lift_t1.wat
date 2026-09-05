;; tests/function/probe_closure_body_prelude_lift_t1.wat — define in fn body do-prefix lifts to prologue.
;;
;; Arc 278 IPC de-prime — migrated off the non-prime `:wat::kernel::spawn-process` onto the
;; composed primes (`spawn-program' (process)` + `recv'`). The DECLARATIONS under test are
;; unchanged: `:h::helper` sits at the child program's TOP LEVEL alongside `:user::main`, and
;; the child's `startup_from_forms` must register it before the body runs. Only the DRIVER and
;; the OBSERVATION flipped: the child now `println`s the value it computes from the declaration,
;; and the parent reads it back off the peer as a `recv'` `RecvOutcome::Message`. That is a
;; STRONGER assertion than the old exit-code check — it proves the declaration was registered
;; AND callable AND returned the right value, not merely that the child did not crash.
;; Exemplar: wat_arc170_program_contracts_t5_launch_lambda.wat (same shape, minus the send').
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :h::helper [] -> :wat::core::i64 42)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [v    (:h::helper)
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
