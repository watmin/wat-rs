;; tests/function/probe_closure_body_prelude_lift_t4.wat — mixed prelude (struct+enum+defn) all lift.
;;
;; Arc 278 IPC de-prime — driver migrated to `spawn-program' (process)` + `recv'`; the three
;; declarations under test (`:h::LocalItem`, `:h::LocalKind`, `:h::make-item`, all at the child
;; program's top level, in that order) are unchanged. The child now exercises ALL THREE and
;; `println`s the combined result (99 from the struct via the factory + 1 from the enum match),
;; so a single value proves every declaration registered in order — stronger than exit-0.
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defstruct :h::LocalItem
             [value <- :wat::core::i64])
           (:wat::core::defenum :h::LocalKind :wat::enum::Pure
             :A
             :B)
           (:wat::core::defn :h::make-item [] -> :h::LocalItem
             (:h::LocalItem 99))
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [item (:h::make-item)
                kind :h::LocalKind::A
                k    (:wat::core::match kind
                       (:h::LocalKind::A 1)
                       (:h::LocalKind::B 2))
                n    (:wat::i64::+ (:h::LocalItem/value item) k)
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
