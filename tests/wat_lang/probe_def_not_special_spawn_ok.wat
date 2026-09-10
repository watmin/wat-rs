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
      [:wat::kernel::RecvOutcome.Message {:msg m} m]
      [:wat::kernel::RecvOutcome.Lost {:cause cause}
        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "launch: stop requested before the child sent its value — the child was alive")]
      [:wat::kernel::RecvOutcome.Closed {}
        (:wat::kernel::assertion-failed! :message "launch: child closed before sending its value")])))
