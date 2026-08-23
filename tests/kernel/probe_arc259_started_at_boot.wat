;; Co-located fixture for probe_arc259_started_at_boot.rs — slurped via startup_beside(file!()).
;; Two named fns; the test injects a boot instant via set_process_boot_instant before calling.

(:wat::core::defn :my::assert-started-at [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq
      (:wat::time::epoch-seconds
        (:wat::program::Env/started-at (:wat::program::env)))
      1000)
    nil))

(:wat::core::defn :my::assert-boot-gap [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-true
      (:wat::core::>
        (:wat::time::seconds
          (:wat::time::-
            (:wat::program::Env/peer-started-at (:wat::program::env))
            (:wat::program::Env/started-at (:wat::program::env))))
        0))
    nil))

