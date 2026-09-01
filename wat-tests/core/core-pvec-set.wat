;; wat-tests/core/core-pvec-set.wat — perf-3: PersistentVector indexed set + drop-last.

(:wat::test::deftest :wat-tests::core::core-pvec-set::set-replaces-at-index
  (:wat::test::assert-eq
    (:wat::core::nth
      (:wat::vector::set (:wat::core::PersistentVector 1 2 3) 1 9)
      1)
    9))

(:wat::test::deftest :wat-tests::core::core-pvec-set::set-does-not-mutate
  (:wat::core::let [v (:wat::core::PersistentVector 1 2 3)
                    _ (:wat::vector::set v 0 0)]
    (:wat::test::assert-eq (:wat::core::nth v 0) 1)))

(:wat::test::deftest :wat-tests::core::core-pvec-set::drop-last-shortens
  (:wat::test::assert-eq
    (:wat::vector::length (:wat::vector::drop-last (:wat::core::PersistentVector 1 2 3)))
    2))

(:wat::test::deftest-hermetic :wat-tests::core::core-pvec-set::set-past-end-is-located
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::vector::set (:wat::core::PersistentVector 1 2 3) 9 0)))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message _m)
             (:wat::kernel::assertion-failed! "expected Lost, got Message" :wat::core::None :wat::core::None))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::edn::write cause))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "expected Lost, got Stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "expected Lost, got Closed" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-true (:wat::regex::matches? "index 9 out of range \\(length 3\\)" msg))))

(:wat::test::deftest-hermetic :wat-tests::core::core-pvec-set::drop-last-empty-is-located
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               (:wat::vector::drop-last (:wat::core::PersistentVector))))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message _m)
             (:wat::kernel::assertion-failed! "expected Lost, got Message" :wat::core::None :wat::core::None))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::edn::write cause))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "expected Lost, got Stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "expected Lost, got Closed" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-true (:wat::regex::matches? "drop-last on empty vector \\(length 0\\)" msg))))
