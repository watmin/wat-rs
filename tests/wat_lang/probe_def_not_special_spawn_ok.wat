;; Fixture: probe 1 — def at fn body do-prefix lifts to prologue end-to-end.
;; The child process registers :h::local-answer = 42 and references it; exits 0.

(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::def :h::local-answer 42)
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
          [v :h::local-answer]
          nil)))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
