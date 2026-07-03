;; typealias_self_ref_bad.wat — self-referential alias must halt at startup. Expect StartupError::Type.
(:wat::core::typealias :my::A :my::A)
