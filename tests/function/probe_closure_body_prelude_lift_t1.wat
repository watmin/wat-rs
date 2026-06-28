;; tests/function/probe_closure_body_prelude_lift_t1.wat — define in fn body do-prefix lifts to prologue.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defn :h::helper [] -> :wat::core::i64 42)
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let [v (:h::helper)] nil)))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
