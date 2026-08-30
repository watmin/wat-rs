;; wat-tests/edn/write-opts.wat — excursus 001 stone WRITE-OPTS.
;;
;; WriteOpts is a VALUE the caller passes. Default is 9 (nanos). JSON verbs
;; take it. Digits clamp to [0, 9] like `:wat::time::to-iso8601`. The 1-arg
;; EDN `write` is not under test here — it is a correctness invariant.
;;
;; Equality is `contains?` of the independently-formatted ISO string, not
;; interpolate-of-a-JSON-object: interpolate treats `{` as a placeholder.

(:wat::test::deftest :wat-tests::edn::opts-default-is-nine
  (:wat::test::assert-eq
    (:wat::edn::WriteOpts/inst-digits (:wat::edn::opts))
    9))

(:wat::test::deftest :wat-tests::edn::write-json-default-is-nine-digits
  (:wat::core::let
    [inst (:wat::time::at-nanos 1200000000)
     json (:wat::edn::write-json inst (:wat::edn::opts))
     iso  (:wat::time::to-iso8601 inst 9)]
    (:wat::core::do
      (:wat::test::assert-true (:wat::string::contains? json "#inst"))
      (:wat::test::assert-true (:wat::string::contains? json iso)))))

(:wat::test::deftest :wat-tests::edn::write-json-digits-zero-has-no-fraction
  (:wat::core::let
    [inst (:wat::time::at-nanos 1200000000)
     json (:wat::edn::write-json inst (:wat::edn::opts/inst-digits 0))
     iso  (:wat::time::to-iso8601 inst 0)]
    (:wat::core::do
      (:wat::test::assert-true (:wat::string::contains? json iso))
      (:wat::test::assert-true (:wat::core::not (:wat::string::contains? json "."))))))

(:wat::test::deftest :wat-tests::edn::write-json-digits-three
  (:wat::core::let
    [inst (:wat::time::at-nanos 1200000000)
     json (:wat::edn::write-json inst (:wat::edn::opts/inst-digits 3))
     iso  (:wat::time::to-iso8601 inst 3)]
    (:wat::test::assert-true (:wat::string::contains? json iso))))

(:wat::test::deftest :wat-tests::edn::write-json-clamps-below-zero-to-zero
  (:wat::core::let
    [inst (:wat::time::at-nanos 1200000000)
     lo   (:wat::edn::write-json inst (:wat::edn::opts/inst-digits -1))
     zero (:wat::edn::write-json inst (:wat::edn::opts/inst-digits 0))]
    (:wat::test::assert-eq lo zero)))

(:wat::test::deftest :wat-tests::edn::write-json-clamps-above-nine-to-nine
  (:wat::core::let
    [inst (:wat::time::at-nanos 1200000000)
     hi   (:wat::edn::write-json inst (:wat::edn::opts/inst-digits 99))
     nine (:wat::edn::write-json inst (:wat::edn::opts/inst-digits 9))]
    (:wat::test::assert-eq hi nine)))

(:wat::test::deftest :wat-tests::edn::write-json-natural-default-is-nine-digits
  (:wat::core::let
    [inst (:wat::time::at-nanos 1200000000)
     json (:wat::edn::write-json-natural inst (:wat::edn::opts))
     iso  (:wat::time::to-iso8601 inst 9)]
    (:wat::test::assert-true (:wat::string::contains? json iso))))
