;; tests/function/probe_closure_body_prelude_lift_t2.wat — struct in fn body do-prefix lifts to prologue.
;;
;; Arc 278 IPC de-prime — driver migrated to `spawn-program' (process)` + `recv'`; the
;; declaration under test (`:h::LocalPoint` at the child program's top level) is unchanged.
;; The child now sums both fields and `println`s the result, so the assertion proves the
;; struct was registered AND constructible AND its accessors resolve — stronger than exit-0.
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defstruct :h::LocalPoint
             [x <- :wat::core::i64
              y <- :wat::core::i64])
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [pt   (:h::LocalPoint :x 3 :y 4)
                n    (:wat::i64::+ (:h::LocalPoint/x pt) (:h::LocalPoint/y pt))
                _out (:wat::kernel::println n)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch: stop requested before child sent its value — child was ALIVE, channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch: child closed before sending its value" :wat::core::None :wat::core::None)))))
