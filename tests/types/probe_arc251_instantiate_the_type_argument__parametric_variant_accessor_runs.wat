;; RUNTIME twin — :Enum.Variant/field must actually read the Enum value.
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [d (:u::Demo.Has {:has 42})]
    (:wat::kernel::println (:u::Demo.Has/has d))))
