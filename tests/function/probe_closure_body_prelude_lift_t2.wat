;; tests/function/probe_closure_body_prelude_lift_t2.wat — struct in fn body do-prefix lifts to prologue.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defstruct :h::LocalPoint
                  [x <- :wat::core::i64
                   y <- :wat::core::i64])
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let [p (:h::LocalPoint :x 3 :y 4)] nil)))))

