;; tests/process/arc112_scheme_probe.wat — co-located fixture for arc112_scheme_probe.rs
;; startup_beside(file!()) world — verifies phantom type params on (Process :- [I O]) survive
;; instantiation and unify against a user-annotated binding.

(:wat::core::defn :my::worker
  []
  -> :wat::core::nil
  nil)

(:wat::core::defn :my::launch [] -> (:wat::kernel::Process :- [:wat::core::i64 :wat::core::i64])
  (:wat::test::spawn-peer (:wat::spawn::process)
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:my::worker)))))

