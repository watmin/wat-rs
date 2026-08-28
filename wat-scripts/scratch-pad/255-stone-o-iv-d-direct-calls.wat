;; wat-scripts/scratch-pad/255-stone-o-iv-d-direct-calls.wat — arc 255 Stone O-iv-d,
;; acceptance row 4. Direct calls (not through apply) to every verb this rider migrated to
;; ALGEBRA, plus a couple of arity-error controls. `time::now` and `uuid::v4` are
;; nondeterministic — their exact printed value differs run to run, so instead of comparing
;; the RESULT verbatim, the type tag / shape is checked by eye; the BEFORE/AFTER diff for
;; those two lines is expected to show a different sampled value, not a different SHAPE. All
;; other lines must diff byte-identical.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "core::List 1 2 3: "
      (:wat::edn::write (:wat::core::List 1 2 3))))
    (:wat::kernel::println (:wat::string::concat "core::List (empty): "
      (:wat::edn::write (:wat::core::List))))
    (:wat::kernel::println (:wat::string::concat "math::pi: "
      (:wat::edn::write (:wat::math::pi))))
    (:wat::kernel::println (:wat::string::concat "uuid::nil: "
      (:wat::edn::write (:wat::uuid::nil))))
    (:wat::kernel::println (:wat::string::concat "uuid::v4 (nondeterministic; shape only): "
      (:wat::edn::write (:wat::uuid::version (:wat::uuid::v4)))))
    (:wat::kernel::println (:wat::string::concat "time::now (nondeterministic; shape only): "
      (:wat::edn::write (:wat::time::now))))
    (:wat::kernel::println (:wat::string::concat "kernel::stopped?: "
      (:wat::edn::write (:wat::kernel::stopped?))))
    (:wat::kernel::println (:wat::string::concat "kernel::sigusr1? (pre-reset): "
      (:wat::edn::write (:wat::kernel::sigusr1?))))
    (:wat::kernel::println (:wat::string::concat "kernel::sigusr2? (pre-reset): "
      (:wat::edn::write (:wat::kernel::sigusr2?))))
    (:wat::kernel::println (:wat::string::concat "kernel::sighup? (pre-reset): "
      (:wat::edn::write (:wat::kernel::sighup?))))
    (:wat::kernel::println (:wat::string::concat "kernel::reset-sigusr1!: "
      (:wat::edn::write (:wat::kernel::reset-sigusr1!))))
    (:wat::kernel::println (:wat::string::concat "kernel::reset-sigusr2!: "
      (:wat::edn::write (:wat::kernel::reset-sigusr2!))))
    (:wat::kernel::println (:wat::string::concat "kernel::reset-sighup!: "
      (:wat::edn::write (:wat::kernel::reset-sighup!))))
    (:wat::kernel::println (:wat::string::concat "kernel::sigusr1? (post-reset): "
      (:wat::edn::write (:wat::kernel::sigusr1?))))
    ;; arity-error controls: passing an arg to a 0-arg verb must still error the same way
    (:wat::kernel::println (:wat::string::concat "math::pi(1) arity error: "
      (:wat::edn::write (:wat::eval-ast! (:wat::core::quote (:wat::math::pi 1))))))
    (:wat::kernel::println (:wat::string::concat "uuid::v4(1) arity error: "
      (:wat::edn::write (:wat::eval-ast! (:wat::core::quote (:wat::uuid::v4 1))))))))
