;; 293.4c collision fixture — two `extend-type` impls for the same `:<T>/<method>`.
;;
;; Expected: startup fails with DuplicateDefine.
;; The second `extend-type` for `:wat::core::String/:t::DupTagged/tag` must be rejected
;; because `:<wat::core::String>/tag` was already registered by the first.

(:wat::core::defsurface :t::DupTagged
  :holder :wat::core::Struct
  :features [(tag [self <- :t::DupTagged] -> :wat::core::i64)])

;; First registration — should succeed.
(:wat::core::extend-type :wat::core::String :t::DupTagged
  (tag [self] -> :wat::core::i64 1))

;; Second registration for the SAME :<T>/<method> — must fail with DuplicateDefine.
(:wat::core::extend-type :wat::core::String :t::DupTagged
  (tag [self] -> :wat::core::i64 2))
