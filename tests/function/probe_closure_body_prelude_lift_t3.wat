;; tests/function/probe_closure_body_prelude_lift_t3.wat — enum in fn body do-prefix lifts to prologue.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defenum :h::LocalDir
                  :North
                  :South)
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let [d :h::LocalDir::North] nil)))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
