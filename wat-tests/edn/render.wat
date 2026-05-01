;; wat-tests/edn/render.wat — arc 079 smoke tests.
;;
;; Verify :wat::edn::write / write-pretty / write-json render every
;; common wat value variant to the expected EDN/JSON string. The
;; renderer's contract is exact-string equality — wat-edn's writer
;; is deterministic, so the tests assert on full output.

;; ─── Primitives ──────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::edn::test-write-bool
  ()
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write true)))
    (:wat::test::assert-eq s "true")))

(:wat::test::deftest :wat-tests::edn::test-write-i64
  ()
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write 42)))
    (:wat::test::assert-eq s "42")))

(:wat::test::deftest :wat-tests::edn::test-write-string
  ()
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write "hello")))
    (:wat::test::assert-eq s "\"hello\"")))

(:wat::test::deftest :wat-tests::edn::test-write-unit
  ()
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write ())))
    (:wat::test::assert-eq s "nil")))

;; ─── Vec ─────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::edn::test-write-vec-i64
  ()
  (:wat::core::let*
    (((v :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 1 2 3))
     ((s :wat::core::String) (:wat::edn::write v)))
    (:wat::test::assert-eq s "[1 2 3]")))

(:wat::test::deftest :wat-tests::edn::test-write-vec-string
  ()
  (:wat::core::let*
    (((v :wat::core::Vector<wat::core::String>) (:wat::core::Vector :wat::core::String "a" "b"))
     ((s :wat::core::String) (:wat::edn::write v)))
    (:wat::test::assert-eq s "[\"a\" \"b\"]")))

;; ─── Tuple ───────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::edn::test-write-tuple
  ()
  (:wat::core::let*
    (((t :(wat::core::i64,wat::core::String)) (:wat::core::Tuple 7 "x"))
     ((s :wat::core::String) (:wat::edn::write t)))
    (:wat::test::assert-eq s "[7 \"x\"]")))

;; ─── JSON path ───────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::edn::test-write-json-vec
  ()
  (:wat::core::let*
    (((v :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 1 2 3))
     ((s :wat::core::String) (:wat::edn::write-json v)))
    (:wat::test::assert-eq s "[1,2,3]")))

(:wat::test::deftest :wat-tests::edn::test-write-json-string
  ()
  (:wat::core::let*
    (((s :wat::core::String) (:wat::edn::write-json "hi")))
    (:wat::test::assert-eq s "\"hi\"")))

;; ─── Pretty path — multi-line for nested vec ─────────────────────
;;
;; A flat vec of small scalars stays inline (write_pretty's
;; "all-scalar and len <= 8" rule). A nested vec breaks across
;; lines. The pretty renderer is deterministic; assert exact text.

(:wat::test::deftest :wat-tests::edn::test-write-pretty-flat
  ()
  (:wat::core::let*
    (((v :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 1 2 3))
     ((s :wat::core::String) (:wat::edn::write-pretty v)))
    (:wat::test::assert-eq s "[1 2 3]")))
