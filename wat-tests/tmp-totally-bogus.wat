;; Deliberately reference a name that exists nowhere — does resolve catch it?

(:wat::test::deftest :wat-tests::tmp::totally-bogus
  ()
  (:wat::test::assert-eq (:totally::made::up::name 42) 42))
