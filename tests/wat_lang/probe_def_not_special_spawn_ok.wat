;; Fixture: probe 1 — def at fn body do-prefix lifts to prologue end-to-end.
;;
;; Arc 278 IPC de-prime — driver migrated to `spawn-program' (process)` + `recv'`; the
;; declaration under test (`:wat::core::def :h::local-answer` at the child program's top
;; level) is unchanged. The child now `println`s the value it read back from the def, so the
;; assertion proves the def registered AND resolved — stronger than exit-0.
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::def :h::local-answer 42)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [v    :h::local-answer
                _out (:wat::kernel::println v)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
