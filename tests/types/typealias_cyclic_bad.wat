;; typealias_cyclic_bad.wat — cyclic alias must halt at startup. Expect StartupError::Type.
(:wat::core::typealias :my::A :my::B)
(:wat::core::typealias :my::B :my::A)
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
