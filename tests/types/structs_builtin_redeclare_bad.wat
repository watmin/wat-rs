;; structs_builtin_redeclare_bad.wat — user cannot redeclare :wat::holon::CapacityExceeded. Must FAIL.
(:wat::core::defstruct :wat::holon::CapacityExceeded
  [boom <- :wat::core::bool])
(:wat::core::defn :user::main [] -> :() ())
