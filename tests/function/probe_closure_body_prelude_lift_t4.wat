;; tests/function/probe_closure_body_prelude_lift_t4.wat — mixed prelude (struct+enum+define) all lift.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defstruct :h::LocalItem
                  [value <- :wat::core::i64])
                (:wat::core::defenum :h::LocalKind
                  :A
                  :B)
                (:wat::core::defn :h::make-item [] -> :h::LocalItem
                  (:h::LocalItem/new 99))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [item (:h::make-item)
                     kind :h::LocalKind::A]
                    nil)))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
