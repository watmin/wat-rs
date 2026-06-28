;; tests/function/probe_closure_body_prelude_lift_t5.wat — prefix-termination semantics.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defn :h::counted-helper [] -> :wat::core::i64 7)
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let [_v (:h::counted-helper)] nil)))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
